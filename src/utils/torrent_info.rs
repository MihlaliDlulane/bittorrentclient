use serde_bencode::de;
use serde_bytes::ByteBuf;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Node(String, i64);

#[derive(Debug, Deserialize)]
pub struct File {
    path: Vec<String>,
    length: i64,
    #[serde(default)]
    md5sum: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct Info {
    pub name: String,
    pub pieces: ByteBuf,
    #[serde(rename = "piece length")]
    pub piece_length: i64,
    #[serde(default)]
    pub md5sum: Option<String>,
    #[serde(default)]
    pub length: Option<i64>,
    #[serde(default)]
    pub files: Option<Vec<File>>,
    #[serde(default)]
    pub private: Option<u8>,
    #[serde(default)]
    pub path: Option<Vec<String>>,
    #[serde(default)]
    #[serde(rename = "root hash")]
    pub root_hash: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Torrent {
    info: Info,
    #[serde(default)]
    pub announce: Option<String>,
    #[serde(default)]
    nodes: Option<Vec<Node>>,
    #[serde(default)]
    encoding: Option<String>,
    #[serde(default)]
    httpseeds: Option<Vec<String>>,
    #[serde(default)]
    #[serde(rename = "announce-list")]
    pub announce_list: Option<Vec<Vec<String>>>,
    #[serde(default)]
    #[serde(rename = "creation date")]
    creation_date: Option<i64>,
    #[serde(rename = "comment")]
    comment: Option<String>,
    #[serde(default)]
    #[serde(rename = "created by")]
    created_by: Option<String>,
}

pub fn render_torrent(torrent: &Torrent) {
    println!("name:\t\t{}", torrent.info.name);
    println!("announce:\t{:?}", torrent.announce);
    println!("nodes:\t\t{:?}", torrent.nodes);
    if let Some(al) = &torrent.announce_list {
        for a in al {
            println!("announce list:\t{}", a[0]);
        }
    }
    println!("httpseeds:\t{:?}", torrent.httpseeds);
    println!("creation date:\t{:?}", torrent.creation_date);
    println!("comment:\t{:?}", torrent.comment);
    println!("created by:\t{:?}", torrent.created_by);
    println!("encoding:\t{:?}", torrent.encoding);
    println!("piece length:\t{:?}", torrent.info.piece_length);
    println!("private:\t{:?}", torrent.info.private);
    println!("root hash:\t{:?}", torrent.info.root_hash);
    println!("md5sum:\t\t{:?}", torrent.info.md5sum);
    println!("path:\t\t{:?}", torrent.info.path);
    if let Some(files) = &torrent.info.files {
        for f in files {
            println!("file path:\t{:?}", f.path);
            println!("file length:\t{}", f.length);
            println!("file md5sum:\t{:?}", f.md5sum);
        }
    }
}


pub fn handletorret(path:String) {
    let torrent_file = std::fs::read(path).unwrap();
    match de::from_bytes::<Torrent>(&torrent_file) {
        Ok(t) => render_torrent(&t),
        Err(e) => println!("Error: {e:?}")
    }
}