use configparser::ini::Ini;
use std::fs::*;
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf, MAIN_SEPARATOR};

pub mod WyagError;
use WyagError::*;

pub struct GitRepository {
    worktree: PathBuf,
    gitdir: PathBuf,
    config: Ini,
}

fn default_config() -> Ini {
    let mut config = Ini::new();
    config.set("core", "repositoryformatversion", Some("0".to_owned()));
    config.set("core", "filemode", Some("false".to_owned()));
    config.set("core", "bare", Some("false".to_owned()));
    config
}

//TODO this should live somehwere else
fn clean_unc(path:PathBuf) -> PathBuf {
    let str_path = path.to_string_lossy();
    if str_path.starts_with(r"\\?\") {
        //Unwrap safe because of check???
        PathBuf::from(str_path.strip_prefix(r"\\?\").unwrap())
    } else {
        path
    }
}

fn find_repo_dir(path:&str) -> Result<PathBuf, InternalError> {
    println!("{}",path);
    let path = PathBuf::from(path).canonicalize()?;
    let path = clean_unc(path);
    println!("{}",path.to_str().unwrap());
    let git_dir = path.join(".git");
        
        if git_dir.exists() {
            Ok(path)
        } else {
            if let Some(parent) = path.parent() {
                find_repo_dir(parent.to_str().unwrap())
            } else {
                Err(InternalError::NotAGitDirectory())
            }
        }
}

impl GitRepository {
    pub fn init(path: &str) -> Result<GitRepository, InternalError> {
        let path = PathBuf::from(path);
        let git_dir = path.join(".git");
        if git_dir.exists() {
            Err(InternalError::AlreadyGitDirectory())
        } else {
            std::fs::create_dir_all(&git_dir)?;
            let config = default_config();
            config.write(git_dir.join("config").to_str().unwrap());
            GitRepository::at_path(path.to_str().unwrap(), false)
        }
    }

    fn at_path(path: &str, force: bool) -> Result<GitRepository, InternalError> {
        let worktree = PathBuf::from(path);
        let gitdir = worktree.join(".git");
        let mut config = Ini::new();
        let config_path = gitdir.join("config");

        if !force && !gitdir.exists() {
            return Err(InternalError::NotAGitDirectory());
        }

        if config_path.is_file() {
            if config.load(config_path.to_str().unwrap()).is_err() {
                return Err(InternalError::Parse());
            }
        } else if !force {
            return Err(InternalError::MissingConfig());
        }

        if !force {
            if let Ok(Some(ver)) = config.getint("core", "repositoryformatversion") {
                if ver != 0 {
                    return Err(InternalError::UnsupportedVersion());
                }
            } else {
                return Err(InternalError::Parse());
            }
        }

        Ok(GitRepository {
            worktree,
            gitdir,
            config,
        })
    }

    //TODO get rid of unwraps here
    fn find_repo(path:&str) -> Result<GitRepository, InternalError> {
        let git_dir = find_repo_dir(path)?;
        GitRepository::at_path(path, false)
    }

    fn repo_path(&self, path: &Path) -> PathBuf {
        self.gitdir.join(path)
    }

    fn repo_dir(&self, path: &Path, mkdir: bool) -> Result<PathBuf, std::io::Error> {
        let path = self.repo_path(path);

        if path.exists() {
            return Ok(path);
        } else if mkdir {
            std::fs::create_dir_all(path.to_str().unwrap_or(""))?;
            return Ok(path);
        } else {
            Err(std::io::Error::new(
                ErrorKind::NotFound,
                "Path does not exist",
            ))
        }
    }

    fn repo_file(&self, path: &Path, mkdir: bool) -> Result<PathBuf, std::io::Error> {
        let path = self.repo_path(path);
        if path.is_file() {
            Ok(path)
        } else if mkdir {
            let mut dir = path.clone();
            dir.pop();
            self.repo_dir(&dir, mkdir)?;
            std::fs::File::create(&path)?;
            Ok(path)
        } else {
            Err(std::io::Error::new(
                ErrorKind::NotFound,
                "File does not exist",
            ))
        }
    }
} //impl GitRepo

#[cfg(test)]
mod tests {
    use super::*;

    fn init_test_repo(temp_dir: &str) -> GitRepository {
        let worktree = get_test_dir(temp_dir);
        let gitdir = worktree.join(".git");
        let config = Ini::new();
        GitRepository {
            worktree,
            gitdir,
            config,
        }
    }

    fn get_test_dir(sub_dir: &str) -> PathBuf {
        [
            "C:\\", "users", "gameo", "appdata", "local", "temp", "testing", sub_dir,
        ]
        .iter()
        .collect::<PathBuf>()
    }

    #[test]
    fn create_repo_path() {
        let test_repo = init_test_repo("create_repo_path");
        let rel_head: PathBuf = ["refs", "head"].iter().collect();
        let head = test_repo.repo_path(&rel_head);
        let res_path = get_test_dir("create_repo_path")
            .join([".git", "refs", "head"].iter().collect::<PathBuf>());
        assert!(head == res_path, "was {}", head.to_string_lossy());
    }

    #[test]
    fn create_repo_dir() {
        //setup
        let test_repo = init_test_repo("create_repo_dir");
        let rel_path: PathBuf = ["refs", "head"].iter().collect();
        let res_dir = get_test_dir("create_repo_dir")
            .join([".git", "refs", "head"].iter().collect::<PathBuf>());
        //clean
        if Path::exists(&res_dir) {
            std::fs::remove_dir_all(&res_dir).expect("unable to clean directory");
        }
        //test
        let repo_dir = test_repo
            .repo_dir(rel_path.as_ref(), true)
            .expect("Error with repo_dir");

        assert!(res_dir == repo_dir, "was {}", repo_dir.to_str().unwrap());
        assert!(res_dir.exists());
    }

    #[test]
    fn create_repo_file() {
        let test_repo = init_test_repo("create_repo_file");
        let rel_path = ["objects", "tags", "test"].iter().collect::<PathBuf>();
        let res_file = get_test_dir("create_repo_file").join(PathBuf::from(".git").join(&rel_path));

        if res_file.exists() {
            std::fs::remove_file(&res_file).expect("unable to clean directory");
        }

        let repo_file = test_repo
            .repo_file(rel_path.as_ref(), true)
            .expect(&format!(
                "Error with repo_file at {}",
                res_file.to_string_lossy()
            ));
        assert!(
            repo_file == res_file,
            "{} does not match {}",
            repo_file.to_string_lossy(),
            res_file.to_string_lossy()
        );
        assert!(repo_file.exists());
    }

    #[test]
    fn create_default_repo() {
        let test_dir = get_test_dir("create_default_repo");
        if test_dir.join(".git").exists() {
            std::fs::remove_dir_all(&test_dir.join(".git")).expect("Error cleaning directory");
        }
        match GitRepository::init(test_dir.to_str().unwrap()) {
            Ok(repo) => {
                assert!(repo.worktree == test_dir);
                assert!(test_dir.join(".git").exists());
            }
            Err(err) => assert!(false, "Error initializing repo: {}",err)
        }
        
    }

    #[test]
    fn find_repo_in_path() {
        let test_dir = get_test_dir("find_repo_in_path");
        let deep_dir = test_dir.join(
            ["A","B","C"].iter().collect::<PathBuf>());
        let git_dir = test_dir.join(
            ["A",".git"].iter().collect::<PathBuf>());
        if !deep_dir.exists() {std::fs::create_dir_all(&deep_dir).expect("Problem creating directory structure");}
        if !git_dir.exists() {std::fs::create_dir(&git_dir).expect("Problem creating git directory");} 

        match find_repo_dir(deep_dir.to_str().unwrap()) {
            //There is weird case stuff here that only works on Windows, that's why there is the to_lower
            Ok(repo_dir) => assert!(
             repo_dir.to_str().unwrap().to_lowercase() ==
             git_dir.parent().unwrap().to_str().unwrap().to_lowercase(),
             "found git dir at: {}, was at {}",
            repo_dir.to_str().unwrap(),
            git_dir.parent().unwrap().to_str().unwrap()),

            Err(error) => assert!(false,"Problem finding repo: {}",error),
        }
    }
}
