use tokio::io::{AsyncReadExt,AsyncWriteExt};
use tokio::net::TcpStream;
use std::net::IpAddr;
use rand::Rng;
use rand::distr::Alphanumeric;


pub async fn peer_handshake(peer_ip: IpAddr, port: u16, info_hash: &[u8]) -> Result<(), Box<dyn std::error::Error>> {

    let peer_addr = format!("{}:{}", peer_ip, port);
    let mut peer_connection = TcpStream::connect(peer_addr).await?;
    println!("Connected to peer");

    // Generate a valid 20-byte peer_id
    let peer_id: Vec<u8> = format!("-BT7110-{}", generate_random_id(12)).into_bytes();

    let reserved_bytes = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

    // Construct handshake message
    let mut handshake = Vec::new();
    handshake.push(19); // Length of protocol string
    handshake.extend_from_slice(b"BitTorrent protocol"); // Protocol string
    handshake.extend_from_slice(&reserved_bytes); // Reserved bytes (all zeros)
    handshake.extend_from_slice(info_hash); // Info hash (20 bytes)
    handshake.extend_from_slice(&peer_id); // Peer ID (20 bytes)

    print_hex(&handshake);

    //println!("Handshake: {:?}", handshake);


    // Send handshake
    peer_connection.write_all(&handshake).await?;
    println!("Handshake message sent!");

    // Read response
    let mut buffer = [0; 68]; 
    peer_connection.read_exact(&mut buffer).await?;

    println!("Received handshake response: {:?}", buffer);
    Ok(())
}

// Generate a random 12-character string for peer_id
fn generate_random_id(len: usize) -> String {
    rand::rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}


fn print_hex(data: &[u8]) {
    println!("{}", data.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" "));
}