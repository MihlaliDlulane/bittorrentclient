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
use sha1::{Digest,Sha1};


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