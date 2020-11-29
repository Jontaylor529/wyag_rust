use std::path::{PathBuf,Path};
use std::io::{ErrorKind};
use configparser::ini::Ini;
use super::super::error::git::GitError;
use super::super::error::object::ObjectParseError;
use crate::lib::{clean_unc,decompress_file_to_bytes};
use super::git_object::{GitObject,factory};

pub struct GitRepository {
    worktree: PathBuf,
    gitdir: PathBuf,
    config: Ini,
}

impl GitRepository {
    pub fn new(worktree:PathBuf, gitdir:PathBuf, config:Ini) -> GitRepository {
        GitRepository {
            worktree,
            gitdir,
            config,
        }
    }

    pub fn worktree(&self) -> &Path {
        self.worktree.as_ref()
    }

    pub fn gitdir(&self) -> &Path {
        self.gitdir.as_ref()
    }

    pub fn config(&self) -> &Ini {
        &self.config
    }

    pub fn init(path: &str) -> Result<GitRepository, GitError> {
        let path = PathBuf::from(path);
        let git_dir = path.join(".git");
        if git_dir.exists() {
            Err(GitError::AlreadyGitDirectory())
        } else {
            std::fs::create_dir_all(&git_dir)?;
            let config = default_config();
            config.write(git_dir.join("config").to_str().unwrap())?;
            GitRepository::at_path(path.to_str().unwrap(), false)
        }
    }

    pub fn at_path(path: &str, force: bool) -> Result<GitRepository, GitError> {
        let worktree = PathBuf::from(path);
        let gitdir = worktree.join(".git");
        let mut config = Ini::new();
        let config_path = gitdir.join("config");

        if !force && !gitdir.exists() {
            return Err(GitError::NotAGitDirectory());
        }

        if config_path.is_file() {
            if config.load(config_path.to_str().unwrap()).is_err() {
                return Err(GitError::Parse());
            }
        } else if !force {
            return Err(GitError::MissingConfig());
        }

        if !force {
            if let Ok(Some(ver)) = config.getint("core", "repositoryformatversion") {
                if ver != 0 {
                    return Err(GitError::UnsupportedVersion());
                }
            } else {
                return Err(GitError::Parse());
            }
        }

        Ok(GitRepository {
            worktree,
            gitdir,
            config,
        })
    }

    //TODO get rid of unwraps here
    pub fn find_repo(path: &str) -> Result<GitRepository, GitError> {
        let git_dir = find_repo_dir(path)?;
        let git_dir = git_dir.to_str().ok_or(GitError::Parse())?;
        GitRepository::at_path(git_dir, false)
    }

    pub fn repo_path(&self, path: &Path) -> PathBuf {
        self.gitdir.join(path)
    }

    pub fn repo_dir(&self, path: &Path, mkdir: bool) -> Result<PathBuf, std::io::Error> {
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

    pub fn repo_file(&self, path: &Path, mkdir: bool) -> Result<PathBuf, std::io::Error> {
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

    pub fn read_object(&self, sha: &str) -> Result<Box<dyn GitObject>, ObjectParseError> {
        let rel_path: PathBuf = ["objects", &sha[..1], &sha[2..]].iter().collect();
        let path = self.repo_file(&rel_path, false)?;
        let raw = decompress_file_to_bytes(&path)?;
        let is_ascii_space = |b: &u8| *b == 0x20;
        let is_ascii_null = |b: &u8| *b == 0x00;
        if let Some(end_object_type) = raw.iter().position(is_ascii_space) {
            if let Some(end_obj_size) = raw.iter().skip(end_object_type).position(is_ascii_null) {
                let obj_type = &raw[..end_object_type - 1];
                let obj_size = &raw[end_object_type + 1..end_obj_size - 1];
                let obj_content = &raw[end_obj_size + 1..];

                let obj_size: usize = String::from_utf8_lossy(&obj_size).parse()?;
                if obj_size == obj_content.len() {
                    let obj_type = String::from_utf8_lossy(obj_type);
                    let obj_content = String::from_utf8_lossy(obj_content);
                    if let Some(git_object) =
                        factory(&(*obj_type), (*obj_content).to_owned())
                    {
                        Ok(git_object)
                    } else {
                        Err(ObjectParseError::ObjectTypeNotRecognized())
                    }
                } else {
                    Err(ObjectParseError::ObjectWrongSize())
                }
            } else {
                Err(ObjectParseError::SizeNotFound())
            }
        } else {
            Err(ObjectParseError::ObjectTypeNotRecognized())
        }
    }
} //impl GitRepo

fn default_config() -> Ini {
    let mut config = Ini::new();
    config.set("core", "repositoryformatversion", Some("0".to_owned()));
    config.set("core", "filemode", Some("false".to_owned()));
    config.set("core", "bare", Some("false".to_owned()));
    config
}

fn find_repo_dir(path: &str) -> Result<PathBuf, GitError> {
    println!("{}", path);
    let path = PathBuf::from(path).canonicalize()?;
    let path = clean_unc(path);
    println!("{}", path.to_str().unwrap());
    let git_dir = path.join(".git");

    if git_dir.exists() {
        Ok(path)
    } else {
        if let Some(parent) = path.parent() {
            find_repo_dir(parent.to_str().unwrap())
        } else {
            Err(GitError::NotAGitDirectory())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{GitRepository,find_repo_dir};
    use configparser::ini::Ini;
    use std::path::{Path,PathBuf};

    fn init_test_repo(temp_dir: &str) -> GitRepository {
        let worktree = get_test_dir(temp_dir);
        let gitdir = worktree.join(".git");
        let config = Ini::new();
        GitRepository::new(worktree, gitdir, config)
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
                assert!(repo.worktree() == test_dir);
                assert!(test_dir.join(".git").exists());
            }
            Err(err) => assert!(false, "Error initializing repo: {}", err),
        }
    }

    #[test]
    fn find_repo_in_path() {
        let test_dir = get_test_dir("find_repo_in_path");
        let deep_dir = test_dir.join(["A", "B", "C"].iter().collect::<PathBuf>());
        let git_dir = test_dir.join(["A", ".git"].iter().collect::<PathBuf>());
        if !deep_dir.exists() {
            std::fs::create_dir_all(&deep_dir).expect("Problem creating directory structure");
        }
        if !git_dir.exists() {
            std::fs::create_dir(&git_dir).expect("Problem creating git directory");
        }

        match find_repo_dir(deep_dir.to_str().unwrap()) {
            //There is weird case stuff here that only works on Windows, that's why there is the to_lower
            Ok(repo_dir) => assert!(
                repo_dir.to_str().unwrap().to_lowercase()
                    == git_dir.parent().unwrap().to_str().unwrap().to_lowercase(),
                "found git dir at: {}, was at {}",
                repo_dir.to_str().unwrap(),
                git_dir.parent().unwrap().to_str().unwrap()
            ),

            Err(error) => assert!(false, "Problem finding repo: {}", error),
        }
    }
}
