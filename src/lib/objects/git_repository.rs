use crate::lib::clean_unc;
use configparser::ini::Ini;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::default::Default;
use std::fmt::{Display,Formatter};
use std::convert::From;

#[derive(Default)]
pub struct GitRepository {
    worktree: PathBuf,
    gitdir: PathBuf,
    config: Ini,
}

#[derive(Debug)]
pub enum RepositoryError {
    FailedToCreateDirctory(std::io::Error),
    DirectoryDoesNotExist(String),
    NoGitDirectory(),
    ConfigLoadFail(String),
    MissingConfig(),
    UnsupportedVersion(i64),
    VersionNotFound(),
}

impl From<std::io::Error> for RepositoryError {
    fn from(io_err: std::io::Error) -> Self {
        RepositoryError::FailedToCreateDirctory(io_err)
    }
}

impl Display for RepositoryError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            RepositoryError::FailedToCreateDirctory(io_err) => {write!(f,"failed to create directory: {}", io_err)},
            RepositoryError::DirectoryDoesNotExist(path) => {write!(f,"Directory does not exist at:{}", path)},
            RepositoryError::NoGitDirectory() => {write!(f, "Git directory not found")},
            RepositoryError::ConfigLoadFail(msg) => {write!(f, "Failed to load config: {}",msg)},
            RepositoryError::MissingConfig() => {write!(f, "Config file not found")},
            RepositoryError::UnsupportedVersion(num) => {write!(f, "Unsuppored version: {}, only 1.0 supported", num)},
            RepositoryError::VersionNotFound() => {write!(f, "Unable to find key for repository version in config")},

        }
    }
}

impl GitRepository {
    pub fn new(worktree: PathBuf, gitdir: PathBuf, config: Ini) -> GitRepository {
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

    pub fn at_path<P: Into<PathBuf>>(path: P, force: bool) -> Result<GitRepository, RepositoryError> {
        let worktree: PathBuf = path.into();
        let gitdir = worktree.join(".git");
        let mut config = Ini::new();
        let config_path = gitdir.join("config");

        if !force && !gitdir.exists() {
            return Err(RepositoryError::NoGitDirectory());
        }

        if config_path.is_file() {
            if let Err(msg) = config.load(config_path.to_str().unwrap()) {
                return Err(RepositoryError::ConfigLoadFail(msg));
            }
        } else if !force {
            return Err(RepositoryError::MissingConfig());
        }

        if !force {
            if let Ok(Some(ver)) = config.getint("core", "repositoryformatversion") {
                if ver != 0 {
                    return Err(RepositoryError::UnsupportedVersion(ver));
                }
            } else {
                return Err(RepositoryError::VersionNotFound());
            }
        }

        Ok(GitRepository {
            worktree,
            gitdir,
            config,
        })
    }

    ///Returns a repo object pointing to a .git folder found anywhere along the given path
    pub fn along_path<P: AsRef<Path>>(path: P, force: bool) -> Result<GitRepository, RepositoryError>{
        let git_dir_folder = find_repo_dir(path.as_ref())?;
        GitRepository::at_path(git_dir_folder, force)
    }

} //impl GitRepo

pub(crate) fn repo_path<P: AsRef<Path>>(repo: &GitRepository, path: P) -> PathBuf {
    repo.gitdir().join(path)
}

pub(crate) fn repo_dir<P: AsRef<Path>>(
    repo: &GitRepository,
    path: P,
    mkdir: bool,
) -> Result<PathBuf, std::io::Error> {
    let path = repo_path(repo, path);

    if path.exists() {
        Ok(path)
    } else if mkdir {
        std::fs::create_dir_all(path.to_str().unwrap_or(""))?;
        Ok(path)
    } else {
        Err(std::io::Error::new(
            ErrorKind::NotFound,
            "Path does not exist",
        ))
    }
}

pub(crate) fn repo_file<P: AsRef<Path>>(
    repo: &GitRepository,
    path: P,
    mkdir: bool,
) -> Result<PathBuf, std::io::Error> {
    let path = repo_path(repo, path);
    if path.is_file() {
        Ok(path)
    } else if mkdir {
        let mut dir = path.clone();
        dir.pop();
        repo_dir(repo, &dir, mkdir)?;
        std::fs::File::create(&path)?;
        Ok(path)
    } else {
        Err(std::io::Error::new(
            ErrorKind::NotFound,
            format!("File does not exist in repo: {}", path.to_string_lossy()),
        ))
    }
}

///Return the path of the folder containing the .git along the given path
fn find_repo_dir<P: Into<PathBuf>>(path: P) -> Result<PathBuf, RepositoryError> {
    let path = path.into().canonicalize()?;
    let path = clean_unc(path);
    let git_dir = path.join(".git");

    if git_dir.exists() {
        Ok(path)
    } else {
        if let Some(parent) = path.parent() {
            find_repo_dir(parent.to_str().unwrap())
        } else {
            Err(RepositoryError::NoGitDirectory())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lib::get_test_dir;
    use configparser::ini::Ini;
    use std::path::{Path, PathBuf};

    fn init_test_repo(temp_dir: &str) -> GitRepository {
        let worktree = get_test_dir(temp_dir);
        let gitdir = worktree.join(".git");
        let config = Ini::new();
        GitRepository::new(worktree, gitdir, config)
    }

    #[test]
    fn create_repo_path() {
        let test_repo = init_test_repo("create_repo_path");
        let rel_head: PathBuf = ["refs", "head"].iter().collect();
        let head = repo_path(&test_repo, &rel_head);
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
        let repo_dir = repo_dir(&test_repo, rel_path, true).expect("Error with repo_dir");

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

        let repo_file = repo_file(&test_repo, rel_path, true).expect(&format!(
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
