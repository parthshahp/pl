use std::path::PathBuf;
use std::{fs, io};

const PROJ_DIRS: &[&str; 1] = &["~/Projects"];

fn main() -> io::Result<()> {
    for proj_dir in PROJ_DIRS {
        let proj_dir = parse_dir(proj_dir);

        let projects = fs::read_dir(proj_dir)?;
        for proj in projects {
            let proj = proj?;
            let proj_git_folder = proj.path().join(".git");
            if proj_git_folder.exists() {
                println!("{:?}", proj.file_name());
            }
        }
    }

    Ok(())
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
