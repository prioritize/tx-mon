use color_eyre::Result;
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

pub struct FileList {
    files: Vec<FileMeta>,
    total_size: u64,
}

pub struct FileMeta {
    path: PathBuf,
    size: u64,
    md5: Option<String>,
}

impl FileList {
    pub fn create(dir: &Path) -> Result<Self> {
        let mut files = Vec::new();
        for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                let md = entry.metadata()?;
                let fm = FileMeta {
                    path: entry.into_path(),
                    size: md.len(),
                    md5: None,
                };
                files.push(fm);
            }
        }
        let size = files.iter().map(|f| f.size).sum();
        Ok(FileList {
            files,
            total_size: size,
        })
    }
}
pub fn run_remote_ll(user: &str, remote: &str, path: &Path) {
    let output = Command::new("sh")
        .arg("-c")
        .arg("ssh")
        .arg(format!("{user}@{remote} ls -lA {}", path.to_str().unwrap()));
    println!(":?output");
}
pub fn run_local_ll(dir: &Path) -> Result<String> {
    let output = Command::new("sh")
        .arg("-c")
        .arg("ls -lA")
        .arg(dir.to_str().unwrap())
        .output()?;

    println!("{output:?}");
    Ok(String::from_utf8(output.stdout)?)
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, io::Write, path::PathBuf};
    use tempfile::TempDir;

    #[test]
    fn test_run_local_ll() {
        let _ = run_local_ll(Path::new("."));
    }
    /// Helper to extract sorted (path, size) tuples from a FileList
    fn sorted_meta_pairs(list: &FileList) -> Vec<(PathBuf, u64)> {
        let mut v: Vec<_> = list
            .files
            .iter()
            .map(|fm| (fm.path.clone(), fm.size))
            .collect();
        v.sort_by(|a, b| a.0.cmp(&b.0));
        v
    }

    #[test]
    fn create_finds_all_files_and_sizes() -> Result<()> {
        // 1) make a TempDir
        let tmp = TempDir::new()?;
        let root = tmp.path();

        // 2) build a nested tree: root/{foo,foo/bar}, plus files at each level
        fs::create_dir_all(root.join("foo").join("bar"))?;

        // file at root
        let f1 = root.join("root.txt");
        {
            let mut file = fs::File::create(&f1)?;
            write!(file, "rust")?; // 4 bytes
        }

        // file in foo
        let f2 = root.join("foo").join("foo.txt");
        {
            let mut file = fs::File::create(&f2)?;
            write!(file, "hello!")?; // 6 bytes
        }

        // file in foo/bar
        let f3 = root.join("foo").join("bar").join("deep.txt");
        {
            let mut file = fs::File::create(&f3)?;
            write!(file, "😊")?; // UTF‑8, 3 bytes
        }

        // 3) run your builder
        let fl = FileList::create(root)?;

        // 4) assert we saw exactly those three files with the right sizes
        let got = sorted_meta_pairs(&fl);
        let want = {
            let mut v = vec![(f1.clone(), 4), (f2.clone(), 6), (f3.clone(), 4)];
            v.sort_by(|a, b| a.0.cmp(&b.0));
            v
        };
        assert_eq!(got, want);

        Ok(())
    }

    // You can add more tests here, e.g.:
    // - empty directory
    // - symlink handling (if you follow links)
    // - checking that `md5` is computed correctly (once you implement it)
}
