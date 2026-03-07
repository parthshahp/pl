use std::ffi::OsString;
use std::fs::{self, DirEntry};
use std::io;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Project {
    pub project_name: OsString,
    pub project_path: PathBuf,
    pub project_remote: String,
}

pub fn get_all_projects(proj_dirs: &[String]) -> Vec<Project> {
    let mut projects: Vec<Project> = proj_dirs
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
            project_remote: get_remote(&entry).unwrap_or("".to_string()),
        })
        .collect();

    projects.sort_by(|a, b| a.project_name.cmp(&b.project_name));
    projects
}

fn get_remote(entry: &DirEntry) -> Result<String, io::Error> {
    let path = entry.path();
    let mut command = std::process::Command::new("git");
    command
        .args(["config", "--get", "remote.origin.url"])
        .current_dir(path);
    let url = command.output()?.stdout;

    if url.is_empty() {
        return Err(io::Error::new(io::ErrorKind::NotFound, "No Git Remote"));
    }

    let url = String::from_utf8(url).unwrap();
    let url = convert_git_remote_to_url(url);

    Ok(url)
}

// This was implemented with AI
fn convert_git_remote_to_url(remote: String) -> String {
    let remote = remote.trim();

    if remote.is_empty() {
        return String::new();
    }

    if remote.starts_with("http://") || remote.starts_with("https://") {
        return remote.trim_end_matches(".git").to_string();
    }

    if let Some(rest) = remote.strip_prefix("git@")
        && let Some((host, path)) = rest.split_once(':')
    {
        return format!("https://{host}/{}", path.trim_end_matches(".git"));
    }

    if let Some(rest) = remote.strip_prefix("ssh://") {
        let rest = rest.strip_prefix("git@").unwrap_or(rest);
        if let Some((host, path)) = rest.split_once('/') {
            return format!("https://{host}/{}", path.trim_end_matches(".git"));
        }
    }

    if let Some(rest) = remote.strip_prefix("git://")
        && let Some((host, path)) = rest.split_once('/')
    {
        return format!("https://{host}/{}", path.trim_end_matches(".git"));
    }

    remote.trim_end_matches(".git").to_string()
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
