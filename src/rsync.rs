use std::{
    io::Read,
    path::{Path, PathBuf},
    process::Command,
};

use color_eyre::Result;
use ssh2::Session;

struct Transfer {
    num_files: u32,
    paths: Vec<PathBuf>,
    bytes: u64,
}

fn dry_run(remote: String, user: String, pass: String, path: &Path) -> Result<Transfer> {
    let str_path = path.to_str().unwrap();
    let mut rsync = Command::new("sshpass");
    rsync
        .arg("-p")
        .arg(pass)
        .arg("rsync")
        .arg("-avhn")
        .arg("--progress")
        .arg("-e")
        .arg("'ssh -p 2222'")
        .arg(format!("{user}@{remote}:{str_path}"));

    println!("{:?}", rsync);
    let output = rsync.output()?;
    println!(
        "{}, {}",
        String::from_utf8(output.stdout).unwrap(),
        String::from_utf8(output.stderr).unwrap()
    );
    // println!(
    //     "{:?}, {:?}",
    //     String::from_utf8(rsync.stdout),
    //     String::from_utf8(rsync.stderr)
    // );
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ssh::connect_local;
    use color_eyre::Result;

    #[test]
    fn test_dry_run() -> Result<()> {
        let _ = dry_run(
            String::from("127.0.0.1"),
            String::from("secureuser"),
            String::from("changeme"),
            Path::new("/"),
        );
        todo!()
    }
}
