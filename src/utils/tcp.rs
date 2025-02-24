use tokio::io::{AsyncReadExt,AsyncWriteExt};
use tokio::net::TcpStream;
use std::net::IpAddr;
use rand::Rng;
use rand::distr::Alphanumeric;

pub async fn peerhandshake(peerip:IpAddr,port:u16,info_hash:&[u8;20]) -> Result<(),  Box<dyn std::error::Error>> {
    let peer = format!("{}:{}",peerip,port);
    println!("Connecting to peer: {}",peer);
    let mut peerconnection = TcpStream::connect(peer).await?;
    println!("Connected to peer");

    let h_len = 19u8;
    let h_string = "BitTorrent protocol";
    let h_reserve = [0u8;8];
    let h_peer_id = generate_peer_id().into_bytes();
    
    let mut handshake = Vec::new();
    handshake.push(h_len);
    handshake.extend_from_slice(h_string.as_bytes());
    handshake.extend_from_slice(&h_reserve);
    handshake.extend_from_slice(info_hash);
    handshake.extend_from_slice(&h_peer_id);

    println!("Handshake: {:?}",&handshake);

    peerconnection.write_all(&handshake).await?;
    println!("Handhsake message sent!");

    let mut buffer =[0;1024];
    let n = peerconnection.read(&mut buffer).await?;
    println!("Received: {}", String::from_utf8_lossy(&buffer[..n]));

    Ok(())
}

fn generate_peer_id() -> String {
    let random_part: String = rand::rng()
        .sample_iter(&Alphanumeric)
        .take(12)  // Ensure 12 random characters
        .map(char::from)
        .collect();

    format!("-TG0001-{}", random_part)  // "-TG0001-" + 12 random chars
}