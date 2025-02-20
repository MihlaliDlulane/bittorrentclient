use std::collections::BTreeMap;
use crate::utils::decode::{decode_bencoded_value,extract_piecce_hashes,compute_info_hash};
use anyhow::{anyhow,Context};
use std::result::Result;
use serde::{Deserialize, Deserializer};
use serde_bytes::ByteBuf;
use serde_bencode::value::Value;
use serde_json;
use hex::encode as hex_encode;
use sha1::{Digest,Sha1};
use reqwest::Client;
use std::collections::HashMap;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC,percent_encode};
use url::form_urlencoded;
use bip_bencode::{BencodeRef, BRefAccess, BDecodeOpt};
use std::default::Default;


#[derive(Deserialize,Clone,Debug)]
struct Info {
    name:String, // Name of the torrent file
    #[serde(rename = "piece length")]
    length:usize, // Size of each piece as bytes     
    pieces:ByteBuf, // raw SHA1 Hashes of pieces
    #[serde(flatten)]
    keys:Keys // Can be lenght if its single file torrent or 'files' if its a multi file torrent
}

#[derive(Clone,Debug)]
struct Torrent {
    announce:String,
    raw_info:Vec<u8>,
    info:Info,
}


#[derive(Deserialize,Clone,Debug)]
#[serde(untagged)]
enum Keys{

    SingleFile {
        length:usize
    },
    MultiFile {
        files:Vec<File>
    }
}


#[derive(Deserialize,Clone,Debug)]
struct File{
    length:usize, // File size in bytes
    path:Vec<String> //File path split in directories
}

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

pub fn print_info(path:String) {
    let torrent_file = std::fs::read(path).context("Reading torrent file");
    let data:Torrent = serde_bencode::from_bytes(&torrent_file.unwrap()).context("Parsing torrent file").unwrap();
    println!("Torrent url: {}",data.announce);

    match &data.info.keys {
        Keys::SingleFile {length} => println!("File size: {}",length),
        Keys::MultiFile { files } => {
            for file in files{
                println!("File: {:?},Size: {}",file.path,file.length);
            }
        }
    }

    let info_hash = serde_bencode::to_bytes(&data.raw_info).context("re bencode?").unwrap();
    let mut hasher = Sha1::new();
    hasher.update(&info_hash);
    let info_has = hasher.finalize();
    println!("Info hash: {}",hex::encode(&info_has));

    println!("Piece length: {:?}",data.info.length);
    let piece_hash = extract_piecce_hashes(&data.info.pieces);
    for (i,hash) in piece_hash.iter().enumerate() {
        println!("Piece: {}, hash: {}",i,hex_encode(hash));
    }
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

    if response.status().is_success() {
        let body = response.text().await?;
        println!("Response: {}", body);
    } else {
        eprintln!("Request failed: {}", response.status());
    }

    Ok(())
}

fn makequery(path: String) -> TorrentGetRequest {
    let torrent_file = std::fs::read(path).expect("Failed to read torrent file");
    let data: Torrent = serde_bencode::from_bytes(&torrent_file).expect("Parsing failed");

    let raw_data = BencodeRef::decode(&torrent_file, BDecodeOpt::default()).unwrap();
    let lookup = raw_data.dict().unwrap().lookup("info".as_bytes()).unwrap();
    let raw_lookup = BencodeRef::buffer(lookup);

    let url1 = data.announce;
    let info_hash = compute_info_hash(&raw_lookup);

    // Only encode non-alphanumeric characters in info_hash
    let encoded_info_hash = utf8_percent_encode(&info_hash, NON_ALPHANUMERIC).to_string(); // Keep it raw

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

fn url_encode_info_hash(info_hash: &[u8]) -> String {
    form_urlencoded::byte_serialize(info_hash).collect::<String>()
}

impl<'de> Deserialize<'de> for Torrent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut map = BTreeMap::<String, Value>::deserialize(deserializer)?;

        // Extract announce
        let announce_value = map
            .remove("announce")
            .ok_or_else(|| serde::de::Error::missing_field("announce"))?;

        let announce = match announce_value {
            Value::Bytes(bytes) => String::from_utf8(bytes)
                .map_err(|e| serde::de::Error::custom(e.to_string()))?,
            _ => return Err(serde::de::Error::custom("announce must be a bencoded string")),
        };

        // Extract raw info as bencoded dictionary (not bytes)
        let raw_info_value = map
            .remove("info")
            .ok_or_else(|| serde::de::Error::missing_field("info"))?;

        // Debugging raw_info_value to see the structure
        //println!("Raw info value: {:?}", raw_info_value);

        let raw_info_bytes = match raw_info_value {
            Value::Dict(_) => {
                 // Serialize the `info` dictionary to bencoded bytes
                 serde_bencode::to_bytes(&raw_info_value)
                 .map_err(|e| serde::de::Error::custom(format!("Bencode serialization error: {}", e)))?
            }
            _ => return Err(serde::de::Error::custom("info must be a bencoded dictionary")),
        };

        //println!("raw_info_byts: {:?}",raw_info_bytes);

        // Return the Torrent struct with the correct info_hash
        let info: Info = serde_bencode::from_bytes(&raw_info_bytes)
            .map_err(|e| serde::de::Error::custom(format!("Bencode deserialization error: {}", e)))?;


        Ok(Torrent {
            announce,
            raw_info: raw_info_bytes,
            info,
        })
    }
}

