#![allow(unused)]
use std::collections::BTreeMap;
use bittorrentclient::decode::{decode_bencoded_value,extract_piecce_hashes,compute_info_hash};
use clap::{builder::Str, Parser, Subcommand};
use anyhow::{Context};
use std::result::Result;
use serde::{de::value, Deserialize, Deserializer};
use serde_bytes::{deserialize, ByteBuf};
use serde_bencode::value::Value;
use hex::encode as hex_encode;


#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command : Command
}

#[derive(Subcommand)]
enum Command {
    Decode {value:String},
    Info {path:String}
}

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


pub fn info_hash_to_bencode_string(info_hash: &[u8; 20]) -> Value {
    // Create a Bencoded string (e.g., "20:<info_hash_bytes>")
    let info_hash_len = info_hash.len();
    let mut bencoded_str = Vec::new();

    // First, write the length of the string (20 in this case)
    bencoded_str.push_str(&info_hash_len.to_string());

    // Then, write the actual byte array
    bencoded_str.extend_from_slice(info_hash);

    // Return as Bencode String Value
    Value::String(bencoded_str.into())
}

fn main() -> () {
    let args = Args::parse();
    
    match args.command {
        Command::Decode { value } => {
            let decodedvalue = decode_bencoded_value(&value).0;
            println!("{decodedvalue}");
        }
        Command::Info { path } => {
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

            let info_hash = compute_info_hash(&data.raw_info).unwrap();
            let bencoded_info_hash = info_hash_to_bencode_string(&info_hash);
            println!("Info hash: {:?}",bencoded_info_hash);

            println!("Piece length: {:?}",data.info.plength);
            let piece_hash = extract_piecce_hashes(&data.info.pieces);
            for (i,hash) in piece_hash.iter().enumerate() {
                println!("Piece: {}, hash: {}",i,hex_encode(hash));
            }
        }    
    }

    ()
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