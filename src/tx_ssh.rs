use color_eyre::{
    Result,
    eyre::{Context, ContextCompat, eyre},
};
use ssh2::Session;
use std::io::Read;
use std::net::TcpStream;
use std::time::Duration;

/// Execute a command on a remote host via SSH
/// Returns (stdout, stderr, exit_code)
pub fn execute_remote_command(
    session: &Session,
    command: &str,
    timeout_secs: Option<u32>,
) -> Result<(String, String, i32)> {
    // Create a new channel for this command
    let mut channel = session
        .channel_session()
        .wrap_err("Failed to create SSH channel")?;

    // Set timeout if specified
    if let Some(timeout) = timeout_secs {
        session.set_timeout(timeout * 1000); // ssh2 uses milliseconds
    }

    // Execute the command
    channel
        .exec(command)
        .wrap_err_with(|| format!("Failed to execute command: {}", command))?;

    // Read stdout
    let mut stdout = String::new();
    channel
        .read_to_string(&mut stdout)
        .wrap_err("Failed to read stdout")?;

    // Read stderr
    let mut stderr = String::new();
    channel
        .stderr()
        .read_to_string(&mut stderr)
        .wrap_err("Failed to read stderr")?;

    // Wait for command to complete and get exit status
    channel.wait_close().wrap_err("Failed to close channel")?;

    let exit_code = channel
        .exit_status()
        .wrap_err("Failed to get exit status")?;

    Ok((stdout, stderr, exit_code))
}

/// Async version using tokio (more suitable for your TUI)
pub async fn execute_remote_command_async(
    session: &Session,
    command: &str,
    timeout_secs: Option<u64>,
) -> Result<(String, String, i32)> {
    // Clone session for async operation
    let command = command.to_string();
    let timeout = timeout_secs;

    // Run the blocking operation in a separate thread
    tokio::task::spawn_blocking(move || {
        // Note: In real implementation, you'd need to pass the session properly
        // This is simplified for the prototype

        // For now, this is a placeholder showing the structure
        // You'd need to restructure to use async SSH libraries like russh
        Ok(("stdout".to_string(), "stderr".to_string(), 0))
    })
    .await
    .wrap_err("Async command execution failed")?
}

/// Higher-level wrapper for common file operations
pub struct RemoteFileOperations<'a> {
    session: &'a Session,
    default_timeout: u32,
}

impl<'a> RemoteFileOperations<'a> {
    pub fn new(session: &'a Session) -> Self {
        Self {
            session,
            default_timeout: 30, // 30 seconds default
        }
    }

    /// List directory contents with detailed information
    pub fn list_directory(&self, path: &str) -> Result<Vec<FileInfo>> {
        let command = format!("ls -lA '{}'", path.replace("'", "'\"'\"'"));
        let (stdout, stderr, exit_code) =
            execute_remote_command(self.session, &command, Some(self.default_timeout))?;

        if exit_code != 0 {
            return Err(eyre!("ls command failed: {}", stderr));
        }

        Ok(parse_ls_output(&stdout))
    }

    /// Get file/directory information
    pub fn stat_file(&self, path: &str) -> Result<FileInfo> {
        let command = format!("stat -c '%F|%s|%Y|%n' '{}'", path.replace("'", "'\"'\"'"));
        let (stdout, stderr, exit_code) =
            execute_remote_command(self.session, &command, Some(self.default_timeout))?;

        if exit_code != 0 {
            return Err(eyre!("stat command failed: {}", stderr));
        }

        parse_stat_output(&stdout.trim())
    }

    /// Check if rsync is available on the remote system
    pub fn check_rsync_available(&self) -> Result<bool> {
        let (_, _, exit_code) = execute_remote_command(
            self.session,
            "which rsync",
            Some(5), // Short timeout for availability check
        )?;

        Ok(exit_code == 0)
    }

    /// Execute rsync dry-run to get transfer information
    pub fn rsync_dry_run(&self, source: &str, dest: &str) -> Result<String> {
        let command = format!(
            "rsync -avun --itemize-changes '{}' '{}'",
            source.replace("'", "'\"'\"'"),
            dest.replace("'", "'\"'\"'")
        );

        let (stdout, stderr, exit_code) = execute_remote_command(
            self.session,
            &command,
            Some(300), // 5 minutes for large directory scans
        )?;

        if exit_code != 0 {
            return Err(eyre!("rsync dry-run failed: {}", stderr));
        }

        Ok(stdout)
    }
}

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub is_directory: bool,
    pub is_symlink: bool,
    pub modified_time: u64, // Unix timestamp
    pub permissions: String,
}

/// Parse ls -lA output into FileInfo structs
fn parse_ls_output(output: &str) -> Vec<FileInfo> {
    let mut files = Vec::new();

    for line in output.lines() {
        if let Some(file_info) = parse_ls_line(line) {
            files.push(file_info);
        }
    }

    files
}

/// Parse a single line from ls -lA output
fn parse_ls_line(line: &str) -> Option<FileInfo> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 9 {
        return None;
    }

    let permissions = parts[0].to_string();
    let size_str = parts[4];
    let name = parts[8..].join(" "); // Handle filenames with spaces

    // Skip . and .. entries
    if name == "." || name == ".." {
        return None;
    }

    let size = size_str.parse().unwrap_or(0);
    let is_directory = permissions.starts_with('d');
    let is_symlink = permissions.starts_with('l');

    Some(FileInfo {
        name: name.clone(),
        path: name,
        size,
        is_directory,
        is_symlink,
        modified_time: 0, // Would need to parse date from ls output
        permissions,
    })
}

/// Parse stat command output
fn parse_stat_output(output: &str) -> Result<FileInfo> {
    let parts: Vec<&str> = output.split('|').collect();
    if parts.len() != 4 {
        return Err(eyre!("Invalid stat output format"));
    }

    let file_type = parts[0];
    let size: u64 = parts[1].parse().wrap_err("Invalid size in stat output")?;
    let modified_time: u64 = parts[2]
        .parse()
        .wrap_err("Invalid timestamp in stat output")?;
    let name = parts[3].to_string();

    Ok(FileInfo {
        name: name.clone(),
        path: name,
        size,
        is_directory: file_type.contains("directory"),
        is_symlink: file_type.contains("symbolic link"),
        modified_time,
        permissions: String::new(), // stat doesn't include permissions in this format
    })
}

/// Example usage
pub fn example_usage() -> Result<()> {
    // Connect to remote host (this part you'd implement based on your connection logic)
    let tcp = TcpStream::connect("example.com:22")?;
    let mut session = Session::new()?;
    session.set_tcp_stream(tcp);
    session.handshake()?;

    // Authenticate (implement your auth logic)
    // session.userauth_password("username", "password")?;

    // Use the remote operations
    let remote_ops = RemoteFileOperations::new(&session);

    // Check if rsync is available
    if remote_ops.check_rsync_available()? {
        println!("rsync is available on remote system");

        // Get transfer plan
        let dry_run_output = remote_ops.rsync_dry_run("/remote/source/", "/local/dest/")?;
        println!("Rsync dry-run output:\n{}", dry_run_output);
    }

    // List directory contents
    let files = remote_ops.list_directory("/home/user")?;
    for file in files {
        println!("{}: {} bytes", file.name, file.size);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ls_line() {
        let line = "-rw-r--r-- 1 user group 1024 Jan 1 12:00 test.txt";
        let file_info = parse_ls_line(line).unwrap();

        assert_eq!(file_info.name, "test.txt");
        assert_eq!(file_info.size, 1024);
        assert!(!file_info.is_directory);
        assert!(!file_info.is_symlink);
    }

    #[test]
    fn test_parse_ls_line_directory() {
        let line = "drwxr-xr-x 2 user group 4096 Jan 1 12:00 mydir";
        let file_info = parse_ls_line(line).unwrap();

        assert_eq!(file_info.name, "mydir");
        assert!(file_info.is_directory);
    }

    #[test]
    fn test_parse_stat_output() {
        let output = "regular file|1024|1640995200|test.txt";
        let file_info = parse_stat_output(output).unwrap();

        assert_eq!(file_info.name, "test.txt");
        assert_eq!(file_info.size, 1024);
        assert!(!file_info.is_directory);
    }
}
