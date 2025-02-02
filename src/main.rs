use std::env;
use std::any::Any;

#[allow(dead_code)]
fn decode_bencoded_value(encoded_value: &str) -> Box<dyn Any> {

    let string_ecode = std::string::String::from(encoded_value);
    match string_ecode.chars().next().unwrap() {
        'i' => {
            return Box::new(decode_integer(encoded_value));
        }
        ch if ch.is_numeric() => {
            return Box::new(decode_string(encoded_value));
        }
        _ => {
            panic!("Unhandled encoded value: {}", encoded_value)
        }
    }
}

fn decode_integer(encoded_value: &str) -> i64 {
    let string_ecode = std::string::String::from(encoded_value);
    let uncoded_value = string_ecode.trim_matches(['i','e']);
    // If encoded_value starts with a digit, it's a number
    if uncoded_value.chars().next().unwrap().is_digit(10) || uncoded_value.chars().next().unwrap() == '-' {
        return uncoded_value.parse::<i64>().unwrap();
    } else {
        panic!("Unhandled encoded value: {}", encoded_value)
    }
}

fn decode_string(encoded_value: &str) ->  String {
    let string_ecode = std::string::String::from(encoded_value);
    let decoded_value = string_ecode.trim_matches(|c:char| c.is_numeric()).trim_matches(':');
    return std::string::String::from(decoded_value);
}

// Usage: your_bittorrent.sh decode "<encoded_value>"
fn main() {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    if command == "decode" {
        let encoded_value = &args[2];
        let decoded_value = decode_bencoded_value(encoded_value);

        if let Some(s) = decoded_value.downcast_ref::<String>() {
            println!("\"{}\"",s)
        } else if let Some(n) = decoded_value.downcast_ref::<i64>()  {
            println!("{}",n)
        }

    } else {
        println!("unknown command: {}", args[1])
    }
}
