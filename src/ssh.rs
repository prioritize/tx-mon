use std::{
    env,
    io::Read,
    net::TcpStream,
    path::{Path, PathBuf},
};

use color_eyre::Result;
use ssh2::Session;

fn new_session() -> Result<()> {
    let sess = Session::new()?;
    let mut agent = sess.agent()?;
    for identity in agent.identities()? {
        println!("An Identity!: {}", identity.comment());
        let pubkey = identity.blob();
        println!("{pubkey:?}");
    }
    Ok(())
}

pub fn connect_local(user: &str, password: &str, port: u32) -> Result<Session> {
    let tcp = TcpStream::connect(format!("127.0.0.1:{port}"))?;
    let mut sess = Session::new()?;
    sess.set_tcp_stream(tcp);
    sess.handshake()?;
    sess.userauth_password(user, password)?;
    assert!(sess.authenticated());
    Ok(sess)
}
pub fn connect_remote(user: &str, port: u32) -> Result<Session> {
    let password = env::var("T_PW")?;
    let tcp = TcpStream::connect(format!("127.0.0.1:{port}"))?;
    let mut sess = Session::new()?;
    sess.set_tcp_stream(tcp);
    sess.handshake()?;
    sess.userauth_agent(user)?;
    assert!(sess.authenticated());
    sess.userauth_password(user, &password)?;
    assert!(sess.authenticated());
    Ok(sess)
}
pub fn ssh_command(session: Session, command: &str) -> Result<String> {
    let mut channel = session.channel_session()?;
    channel.exec(command)?;
    let mut s = String::new();
    channel.read_to_string(&mut s)?;
    channel.wait_close().unwrap();
    match channel.exit_status() {
        Ok(_) => Ok(s),
        Err(code) => Err(code.into()),
    }
}
pub fn list_files(session: Session, path: &Path) -> Result<String> {
    let file_list = ssh_command(session, &format!("ls -la {}", path.display()))?;
    println!("{file_list}");
    let (files, directories) = parse_ls(file_list)?;
    todo!()
}
pub fn parse_ls(list: String) -> Result<(Vec<String>, Vec<String>)> {
    let entries: Vec<Vec<String>> = list
        .lines()
        .map(|line| line.split(" ").map(|word| String::from(word)).collect())
        .collect();
    println!("{entries:?}");
    todo!()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn test_new_session() {
        assert!(new_session().is_ok());
    }
    #[test]
    fn test_connect_local() {
        let _ = connect_local("secureuser", "changeme", 2222);
    }

    #[test]
    fn test_ssh_command_ls() -> Result<()> {
        let sess = connect_local("secureuser", "changeme", 2222)?;
        let _ = ssh_command(sess, "ls");
        Ok(())
    }
    #[test]
    fn test_list_files() -> Result<()> {
        let sess = connect_local("secureuser", "changeme", 2222)?;
        let _ = list_files(sess, &PathBuf::from("~/"));
        Ok(())
    }
}
