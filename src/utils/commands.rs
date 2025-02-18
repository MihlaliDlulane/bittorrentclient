use std::collections::BTreeMap;
use crate::utils::decode::{decode_bencoded_value,extract_piecce_hashes,compute_info_hash};
use anyhow::Context;
use std::result::Result;
use serde::{Deserialize, Deserializer};
use serde_bytes::ByteBuf;
use serde_bencode::value::Value;
use serde_json;
use hex::encode as hex_encode;
use sha1::{Digest,Sha1};
use reqwest::Client;
use std::collections::HashMap;
use url::form_urlencoded;

#[derive(Deserialize,Clone,Debug)]
struct Info {
    name:String, // Name of the torrent file
    #[serde(rename = "piece length")]
    plength:usize, // Size of each piece as bytes     
    pieces:ByteBuf, // raw SHA1 Hashes of pieces
    #[serde(flatten)]
    keys:Keys // Can be lenght if its single file torrent or 'files' if its a multi file torrent
}

#[derive(Clone,Debug)]
struct Torrent {
    announce:String,
    raw_info:Value,
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
    info_hash:Vec<u8>, //  the info hash of the torrent
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

    println!("Piece length: {:?}",data.info.plength);
    let piece_hash = extract_piecce_hashes(&data.info.pieces);
    for (i,hash) in piece_hash.iter().enumerate() {
        println!("Piece: {}, hash: {}",i,hex_encode(hash));
    }
}


pub async fn print_peers(path:String) -> Result<(), Box<dyn std::error::Error>> {
   
    let data = makequery(path);
    let client = Client::new();
    let url = data.url;

    println!("Url request to {}",url);
    let mut params = HashMap::new();

    
    let result = Sha1::digest(data.info_hash);
    let result = form_urlencoded::byte_serialize(&result).collect::<String>();

    println!("result:{}",result);


    params.insert("info_hash",result);
    params.insert("peer_id",data.peer_id);
    params.insert("port",data.port.to_string());
    params.insert("uploaded",data.uploaded.to_string());
    params.insert("downloaded",data.dowloaded.to_string());
    params.insert("left",data.left.to_string());
    params.insert("compact",data.compact.to_string());

    let response = client.get(url).query(&params).send().await?;

    if response.status().is_success() {
        let body = response.text().await?;
        println!("Response: {}", body);
    } else {
        println!("Request failed with status: {}", response.status());
    }

    Ok(())
}

fn makequery(path:String) -> TorrentGetRequest{
    let torrent_file = std::fs::read(path).context("Reading torrent file");
    let data:Torrent = serde_bencode::from_bytes(&torrent_file.unwrap()).context("Parsing torrent file").unwrap();
    let url1 = data.announce;
    let mut torrent_files: Vec<(String,usize)> = Vec::new();

    match &data.info.keys {
        Keys::SingleFile {length} => {torrent_files.push((data.info.name,*length))},
        Keys::MultiFile { files } => {
            for file in files{
                torrent_files.push((file.path[0].clone(),file.length));
            }
        }
    }

    let info_hash1 = serde_bencode::to_bytes(&data.raw_info).context("re bencode?").unwrap();
    
    let querystruct = TorrentGetRequest{
        url: url1,
        info_hash : info_hash1,
        peer_id : "11111222223333344444".to_string(),
        port : 6881,
        uploaded: 0,
        dowloaded: 0,
        left: torrent_files[0].1,
        compact : 1
    };

    return querystruct
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

        // Extract raw info as bencoded bytes
        let raw_info_value = map
            .remove("info")
            .ok_or_else(|| serde::de::Error::missing_field("info"))?;

        let raw_info = raw_info_value.clone();

        let temp_raw_info = serde_bencode::to_bytes(&raw_info_value)
            .map_err(|e| serde::de::Error::custom(format!("Bencode serialization error: {}", e)))?;

        // Deserialize info into the Info struct
        let info: Info = serde_bencode::from_bytes(&temp_raw_info)
            .map_err(|e| serde::de::Error::custom(format!("Bencode deserialization error: {}", e)))?;

        // Return the Torrent struct
        Ok(Torrent {
            announce,
            raw_info,
            info,
        })
    }
}

