use std::net::TcpStream;

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

fn connect_local(port: u8) -> Result<()>{
    let tcp = TcpStream::connect(format!("127.0.0.1:{port}"))?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_session() {
        assert!(new_session().is_ok());
    }
}
