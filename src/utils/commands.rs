use crate::utils::decode::{decode_bencoded_value,compute_info_hash};
use std::result::Result;
use serde::Deserialize;
use reqwest::Client;
use std::collections::HashMap;
use percent_encoding::{percent_encode,NON_ALPHANUMERIC};
use bip_bencode::{BencodeRef, BRefAccess, BDecodeOpt};
use std::default::Default;


#[derive(Deserialize,Clone,Debug)]
struct TorrentGetRequest{
    url:String, // url
    info_hash:String, //  the info hash of the torrent
    peer_id:String, //  a unique identifier for your client
    port:u16, // the port your client is listening on
    uploaded:u64, // the total amount uploaded so far
    dowloaded:u64, // the total amount downloaded so far
    left:usize, //  the number of bytes left to download
    compact:u8 // whether the peer list should use the compact representation
}


pub fn print_decode(value:String) {
    let decodedvalue = decode_bencoded_value(&value).0;
            println!("{decodedvalue}");
}


pub async fn print_peers(path: String) -> Result<(), Box<dyn std::error::Error>> {
    let data = makequery(path);
    let client = Client::new();
    let base_url = data.url.clone();

    println!("Url request to {}", base_url);
    println!("info hash: {}", data.info_hash);

    // URL encode all parameters except "info_hash"
    let mut params = HashMap::new();
    params.insert("peer_id", data.peer_id);
    params.insert("port", data.port.to_string());
    params.insert("uploaded", data.uploaded.to_string());
    params.insert("downloaded", data.dowloaded.to_string());
    params.insert("left", data.left.to_string());
    params.insert("compact", data.compact.to_string());

    // Serialize parameters (except info_hash) using serde_urlencoded
    let encoded_params = serde_urlencoded::to_string(&params)?;

    // Manually construct final URL with unencoded info_hash
    let tracker_url = format!("{}?{}&info_hash={}", base_url, encoded_params, data.info_hash);

    println!("Final URL: {}", tracker_url);

    let response = client.get(&tracker_url).send().await?;
    let response = response.bytes().await?;
    println!("Bencode response: {:?}",response);

 
    Ok(())
}

fn makequery(path: String) -> TorrentGetRequest {
    let torrent_file = std::fs::read(path).expect("Failed to read torrent file");

    let raw_data = BencodeRef::decode(&torrent_file, BDecodeOpt::default()).unwrap();
    let lookup = raw_data.dict().unwrap().lookup("info".as_bytes()).unwrap();
    let raw_lookup = BencodeRef::buffer(lookup);

    let announce_raw =  raw_data.dict().unwrap().lookup("announce".as_bytes()).unwrap();
    let announce =  announce_raw.str().unwrap();
    let url1 = announce.to_string();
    let info_hash = compute_info_hash(&raw_lookup);

    // Only encode non-alphanumeric characters in info_hash
    let encoded_info_hash = percent_encode(&info_hash, NON_ALPHANUMERIC).to_string(); // Keep it raw

    TorrentGetRequest {
        url: url1,
        info_hash:encoded_info_hash, 
        peer_id: "11111222223333344444".to_string(),
        port: 6881,
        uploaded: 0,
        dowloaded: 0,
        left: 0,
        compact: 1,
    }
}


