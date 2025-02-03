
use std::any::Any;

#[allow(dead_code)]
pub fn decode_bencoded_value(encoded_value: &str) -> Box<dyn Any> {

    let de_type = decode_type(encoded_value);
    match de_type {
        v if v == "integer" => {
            return Box::new(decode_integer(encoded_value));
        }
        v if v == "string" => {
            return Box::new(decode_string(encoded_value));
        }
        v if v == "list" => {
            return Box::new(decode_lists(encoded_value));
        }
        _ => {
            panic!("Unhandled encoded value: {}", encoded_value)
        }
    }
}

fn decode_type(encoded_value: &str) -> String {
    let string_ecode = std::string::String::from(encoded_value);
    match string_ecode.chars().next().unwrap() {
        'i' => {
            return "integer".to_string();
        }
        ch if ch.is_numeric() => {
            return "string".to_string();
        }
        'l' => {
            return "list".to_string()
        }
        _ => {
            return "unknown".to_string();
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

fn decode_lists(encoded_value: &str) -> Vec<Box<dyn Any>> {
    let itemslist:String = encoded_value.chars().skip(1).take(encoded_value.len().saturating_sub(2)).collect();
    let mut itemschar:Vec<char> = itemslist.chars().collect();
    let mut resultarray:Vec<Box<dyn Any>> = Vec::new();
    
    let mut len_items = itemschar.len();
    let mut i:usize = 0;
    
    while i <= len_items {
        match itemschar[i] {
            'i' => {
                let endindex = itemschar.iter().position(|&c| c == 'e').unwrap();
                let inter_value:String = itemschar[i..endindex].iter().collect();
                itemschar.drain(i..endindex);
                resultarray.push(Box::new(decode_integer(&inter_value)));
                i = endindex ;
                len_items = itemschar.len();
            }
            v if v.is_numeric() => {
                if itemschar.iter().position(|&c| c=='i' || c=='l' || c.is_numeric()) == Some(len_items - 1) {
                    let string_value:String = itemschar[i..len_items-1].iter().collect();
                    itemschar.drain(i..len_items - 1);
                    resultarray.push(Box::new(decode_string(&string_value)));
                    len_items = itemschar.len();
                    i = len_items 
                } else {
                    let endindex = itemschar.iter().position(|&c| c=='i' || c=='l' || c.is_numeric());
                    let string_value:String = itemschar[i..endindex.unwrap()].iter().collect();
                    itemschar.drain(i..endindex.unwrap());
                    resultarray.push(Box::new(decode_string(&string_value)));
                    i = endindex.unwrap();
                    len_items = itemschar.len()
                }
            }
            'l' => {
                let endindex = itemschar.iter().position(|&c| c == 'e').unwrap();
                let list_value:String = itemschar[i..endindex].iter().collect();
                itemschar.drain(i..endindex);
                resultarray.push(Box::new(decode_lists(&list_value)));
                i = endindex ;
                len_items = itemschar.len();
            }
            _ => {
            }
        }
    }

    return resultarray;
}