use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Project {
    pub project_name: OsString,
    pub project_path: PathBuf,
    pub project_remote: String,
}

pub fn get_all_projects(proj_dirs: &[String]) -> Vec<Project> {
    proj_dirs
        .iter()
        .map(|dir| parse_dir(dir))
        .flat_map(|dir| {
            fs::read_dir(&dir)
                .ok()
                .into_iter()
                .flat_map(|read_dir| read_dir.filter_map(Result::ok))
        })
        .filter(|entry| entry.path().join(".git").exists())
        .map(|entry| Project {
            project_name: entry.file_name(),
            project_path: entry.path(),
            project_remote: "google.com".to_string(),
        })
        .collect()
}

fn parse_dir(proj_dir: &str) -> PathBuf {
    if proj_dir == "~" {
        return home_dir();
    }

    if let Some(stripped) = proj_dir.strip_prefix("~/") {
        return home_dir().join(stripped);
    }

    PathBuf::from(proj_dir)
}

fn home_dir() -> PathBuf {
    dirs::home_dir().expect("could not determine home directory")
}
