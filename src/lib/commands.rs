use crate::lib::error::git::GitError;
use configparser::ini::Ini;
use std::path::PathBuf;

fn default_config() -> Ini {
    let mut config = Ini::new();
    config.set("core", "repositoryformatversion", Some("0".to_owned()));
    config.set("core", "filemode", Some("false".to_owned()));
    config.set("core", "bare", Some("false".to_owned()));
    config
}

pub fn init<P: Into<PathBuf>>(path: P) -> Result<(), GitError> {
    let path: PathBuf = path.into();
    let git_dir = path.join(".git");
    if git_dir.exists() {
        Err(GitError::AlreadyGitDirectory())
    } else {
        std::fs::create_dir_all(&git_dir)?;
        let config = default_config();
        config.write(git_dir.join("config").to_str().unwrap())?;
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::init;
    use crate::lib::get_test_dir;
    #[test]
    fn create_default_repo() {
        let test_dir = get_test_dir("create_default_repo");
        if test_dir.join(".git").exists() {
            std::fs::remove_dir_all(&test_dir.join(".git")).expect("Error cleaning directory");
        }
        match init(test_dir.to_str().unwrap()) {
            Ok(_) => assert!(true),
            Err(err) => assert!(false, "Error initializing repo: {}", err),
        }
    }
}
