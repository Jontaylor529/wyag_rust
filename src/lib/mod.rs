use configparser::ini::Ini;
use std::fs::*;
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf, MAIN_SEPARATOR};
pub struct GitRepository {
    worktree: PathBuf,
    gitdir: PathBuf,
    config: Ini,
}

impl GitRepository {
    fn try_init(path: &str, force: bool) -> Result<GitRepository, &str> {
        let worktree = PathBuf::from(path);
        let gitdir = worktree.join(".git");
        let mut config = Ini::new();
        let config_path = gitdir.join("config");

        if !force && !gitdir.exists() {
            return Err("Not a git directory!");
        }

        if config_path.is_file() {
            if config.load(config_path.to_str().unwrap()).is_err() {
                return Err("Problem loading config file");
            }
        } else if !force {
            return Err("No config file found");
        }

        if !force {
            if let Ok(Some(ver)) = config.getint("core", "repositoryformatversion") {
                if ver != 0 {
                    return Err("Unsupported version");
                }
            } else {
                return Err("repositoryformateversion could not be parsed");
            }
        }

        Ok(GitRepository {
            worktree,
            gitdir,
            config,
        })
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

    fn init_test_repo() -> GitRepository {
        let worktree = get_test_dir();
        let gitdir = worktree.join(".git");
        let config = Ini::new();
        GitRepository {
            worktree,
            gitdir,
            config,
        }
    }

    fn get_test_dir() -> PathBuf {
        PathBuf::from("C:\\users\\gameo\\appdata\\local\\temp")
    }

    #[test]
    fn create_repo_path() {
        let test_repo = init_test_repo();
        let rel_head: PathBuf = ["refs", "head"].iter().collect();
        let head = test_repo.repo_path(&rel_head);
        let res_path = get_test_dir().join([".git", "refs", "head"].iter().collect::<PathBuf>());
        assert!(head == res_path, "was {}", head.to_string_lossy());
    }

    #[test]
    fn create_repo_dir() {
        //setup
        let test_repo = init_test_repo();
        let rel_path: PathBuf = ["refs", "head"].iter().collect();
        let res_dir = get_test_dir().join([".git", "refs", "head"].iter().collect::<PathBuf>());
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
        let test_repo = init_test_repo();
        let rel_path = ["objects", "tags", "test"].iter().collect::<PathBuf>();
        let res_file = get_test_dir().join(PathBuf::from(".git").join(&rel_path));

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
}