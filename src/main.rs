use bittorrentclient::decode::decode_bencoded_value;
use clap::{Parser,Subcommand};
use anyhow::{Context, Ok,Result};
use serde::Deserialize;
use serde_bytes::ByteBuf;

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

#[derive(Deserialize,Debug)]
struct Info {
    length: Option<u64>,
    name: String,
    //piece_length :u64,
    pieces: ByteBuf,
}

#[derive(Deserialize,Debug)]
struct Torrent {
    announce: String,
    info:Info
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    match args.command {
        Command::Decode { value } => {
            let decodedvalue = decode_bencoded_value(&value).0;
            println!("{decodedvalue}");
        }
        Command::Info { path } => {
            let torrent_file = std::fs::read(path).context("Reading torrent file")?;
            let data:Torrent = serde_bencode::from_bytes(&torrent_file).context("Parsing torrent file")?;
            println!("Torrent url: {}",data.announce);
            println!("info: {}",data.info.length.unwrap());
        }    
    }

    Ok(())
}
