#![allow(dead_code)]
use std::env;
use std::path::PathBuf;

fn get_user_home_dir() -> Option<PathBuf> {
    if cfg!(target_os = "windows") {
        // On Windows, check the `USERPROFILE` or `HOMEDRIVE` + `HOMEPATH`
        env::var("USERPROFILE").or_else(|_| {
            let homedrive = env::var("HOMEDRIVE");
            let homepath = env::var("HOMEPATH");
            match (homedrive, homepath) {
                (Ok(drive), Ok(path)) => Ok(format!("{}{}", drive, path)),
                _ => Err(env::VarError::NotPresent),
            }
        }).ok().map(PathBuf::from)
    } else {
        // On Unix-like systems (Linux, macOS), check the `HOME` environment variable
        env::var("HOME").ok().map(PathBuf::from)
    }
}