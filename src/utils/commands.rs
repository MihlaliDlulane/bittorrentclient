use crate::utils::decode::{decode_bencoded_value,compute_info_hash};
use crate::utils::torrent_info::Torrent;
use std::result::Result;
use serde::{Deserialize,Serialize,Deserializer};
use serde::de::Visitor;
use serde_bencode::de;
use reqwest::Client;
use std::collections::HashMap;
use percent_encoding::{percent_encode,NON_ALPHANUMERIC};
use bip_bencode::{BencodeRef, BRefAccess, BDecodeOpt};
use std::default::Default;
use std::net::{IpAddr,Ipv4Addr};
use std::fmt;


#[derive(Deserialize,Clone,Debug)]
struct TorrentGetRequest{
    url:String, // url
    url_list:Option<Vec<Vec<String>>>, // list of trackers
    info_hash:String, //  the info hash of the torrent
    info_hash_bytes:[u8;20],
    peer_id:String, //  a unique identifier for your client
    port:u16, // the port your client is listening on
    uploaded:u64, // the total amount uploaded so far
    dowloaded:u64, // the total amount downloaded so far
    left:usize, //  the number of bytes left to download
    compact:u8 // whether the peer list should use the compact representation
}

#[derive(Debug, Deserialize)]
struct GetResponse{
    interval:u64,
    #[serde(deserialize_with = "deserialize_peers")]
    peers:Vec<Peer>
}

#[derive(Debug,Serialize,Deserialize)]
pub struct Peer{
    pub ip:IpAddr,
    pub port:u16
}

fn deserialize_peers<'de, D>(deserializer: D) -> Result<Vec<Peer>, D::Error>
where
    D: Deserializer<'de>,
{
    struct PeersVisitor;

    impl<'de> Visitor<'de> for PeersVisitor {
        type Value = Vec<Peer>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a byte array representing peers in a compact format")
        }

        fn visit_bytes<E>(self, value: &[u8]) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            let mut peers = Vec::new();

            if value.len() % 6 != 0 {
                return Err(E::custom(format!(
                    "Peers data length {} is not a multiple of 6",
                    value.len()
                )));
            }

            for chunk in value.chunks_exact(6) {
                let ip = Ipv4Addr::new(chunk[0], chunk[1], chunk[2], chunk[3]);
                let port = u16::from_be_bytes([chunk[4], chunk[5]]);
                peers.push(Peer {
                    ip: IpAddr::V4(ip),
                    port,
                });
            }

            Ok(peers)
        }
    }

    deserializer.deserialize_bytes(PeersVisitor)
}


pub fn print_decode(value:String) {
    let decodedvalue = decode_bencoded_value(&value).0;
            println!("{decodedvalue}");
}


pub async fn print_peers(path: String) -> Result<(), Box<dyn std::error::Error>> {
    let data = makequery(path);
    let client = Client::new();
    let base_url = data.url.clone();

    // println!("Url request to {}", base_url);
    // println!("info hash: {}", data.info_hash);

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
    let bencode_response = de::from_bytes::<GetResponse>(&response);
    
    match bencode_response {
        Ok(x) =>{
            println!("Raw Peers: {:?}",x.peers);
        }
        Err(e) =>{panic!("Error: {e:?}")}
    }

 
    Ok(())
}

pub async fn return_peers_and_infohash(path: String) -> Result<(Vec<Peer>,[u8;20]), Box<dyn std::error::Error>> {
    let data = makequery(path);
    let client = Client::new();
    let base_url = data.url.clone();

    // println!("Url request to {}", base_url);
    // println!("info hash: {}", data.info_hash);

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
    let bencode_response = de::from_bytes::<GetResponse>(&response);
    
    match bencode_response {
        Ok(x) =>{
            return Ok((x.peers,data.info_hash_bytes))
        }
        Err(e) =>{panic!("Error: {e:?}")}
    }

}

fn makequery(path: String) -> TorrentGetRequest {
    let torrent_file = std::fs::read(path).expect("Failed to read torrent file");
    let torrent_detials = de::from_bytes::<Torrent>(&torrent_file);

    let raw_data = BencodeRef::decode(&torrent_file, BDecodeOpt::default()).unwrap();
    let lookup = raw_data.dict().unwrap().lookup("info".as_bytes()).unwrap();
    let raw_lookup = BencodeRef::buffer(lookup);
    let mut url1 = String::new();
    let mut trackerlist : Vec<Vec<String>> = Vec::new();

    match torrent_detials {
        Ok(x) => {
            match x.announce {
                None =>{
                    let announce_list = x.announce_list;
                    trackerlist = announce_list.clone().unwrap();
                    let first_url = announce_list.unwrap()[0].clone();
                    url1 = first_url[0].clone();
                }

                _ =>{
                    url1 = x.announce.unwrap();
                }
            }
        }
        Err(e) => { panic!("Error: {e:?}")}
    }


    let info_hash = compute_info_hash(&raw_lookup);

    // Only encode non-alphanumeric characters in info_hash
    let encoded_info_hash = percent_encode(&info_hash, NON_ALPHANUMERIC).to_string();

    TorrentGetRequest {
        url: url1,
        url_list: Some(trackerlist),
        info_hash:encoded_info_hash,
        info_hash_bytes : info_hash, 
        peer_id: "11111222223333344444".to_string(),
        port: 6881,
        uploaded: 0,
        dowloaded: 0,
        left: 0,
        compact: 1,
    }
}

async fn try_trackers(trackers: Vec<Vec<String>>, request_url: &str) -> Option<String> {
    let client = Client::new();

    for tier in trackers {
        for tracker in tier {
            let url = format!("{}?{}", tracker, request_url);
            println!("Trying tracker: {}", url);

            match client.get(&url).send().await {
                Ok(response) => {
                    if !response.status().is_success() {
                        println!("Tracker {} returned HTTP {}", tracker, response.status());
                        continue;
                    }

                    match response.bytes().await {
                        Ok(body) => {
                            let decoded: Result<HashMap<String, serde_bencode::value::Value>, _> =
                                serde_bencode::from_bytes(&body);
                            if let Ok(data) = decoded {
                                if let Some(failure_reason) = data.get("failure reason") {
                                    println!("Tracker {} failed: {:?}", tracker, failure_reason);
                                    continue;
                                }
                                println!("Tracker {} succeeded!", tracker);
                                return Some(tracker); // Success, return working tracker
                            } else {
                                println!("Tracker {} returned an invalid response.", tracker);
                            }
                        }
                        Err(_) => println!("Failed to read response from {}", tracker),
                    }
                }
                Err(e) => println!("Failed to connect to tracker {}: {}", tracker, e),
            }
        }
    }
    
    println!("No working tracker found.");
    None
}
