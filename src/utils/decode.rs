use core::panic;
use anyhow::{Context,Result};
use std::result::Result::Ok;
use serde_bencode::to_bytes;
use serde_json;
use sha1::{Digest, Sha1};

#[allow(dead_code)]
pub fn decode_bencoded_value(encoded_value: &str) -> (serde_json::Value,&str)  {

    match encoded_value.chars().next() {
        Some('i') => {
            if let Some((n,rest)) = encoded_value
                                    .split_at(1)
                                    .1.split_once('e')
                                    .and_then(|(digits,rest)| {
                                        let n = digits.parse::<i64>().ok()?;
                                        Some((n,rest))
                                    })
                                    {
                                        return (n.into(),rest);
                                    }
        }
        Some('l') => {
            let mut values = Vec::new();
            let mut rest = encoded_value.split_at(1).1;
            while !rest.is_empty() && !rest.starts_with('e') {
                let (v,remainder) = decode_bencoded_value(rest);
                values.push(v);
                rest = remainder;
            }

            return (values.into(),&rest[1..]);
        }

        Some('d') => {
            let mut dict = serde_json::Map::new();
            let mut rest = encoded_value.split_at(1).1;
            while !rest.is_empty() && !rest.starts_with('e') {
                let (k,remainder) = decode_bencoded_value(rest);
                let k = match k {
                    serde_json::Value::String(k) => k,
                    k => {
                        panic!("dict keys must be strings, not {k:?}");
                    }
                };
                let (v,remainder) =decode_bencoded_value(remainder);
                dict.insert(k, v);
                rest = remainder;
            }
            return (dict.into(),&rest[1..]);
        }

        Some('0'..'9') => {
            if let Some((len, rest)) = encoded_value.split_once(':') {

                if let Ok(len) = len.parse::<usize>() {

                    return (rest[..len].to_string().into(), &rest[len..]);

                }
            }
        }
        _ => {}
    }

    panic!("unhandles bencode value:{}",encoded_value)
}

pub fn extract_piecce_hashes(pieces: &[u8]) -> Vec<[u8;20]> {
    pieces.chunks_exact(20)
          .map(|chunk| chunk.try_into().expect("Invalid sha1 length"))
          .collect()
}

pub fn compute_info_hash(raw_info: &[u8]) -> [u8; 20] {
    let hash_result = Sha1::digest(raw_info);
    let mut info_hash = [0u8; 20];
    info_hash.copy_from_slice(&hash_result[..20]);
    
    println!("Computed Info Hash (Hex): {}", hex::encode(&info_hash));
    info_hash
}