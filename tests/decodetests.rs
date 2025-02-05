use bittorrentclient::decode::decode_bencoded_value;
use std::any::Any;

fn are_vecs_equal(vec1:&Vec<Box<dyn Any>>,vec2:&Vec<Box<dyn Any>>) -> bool{
    // Check if dynamic array contain the same items
    if vec1.len() != vec2.len() {
        return false;
    }
    for x in 0..vec1.len() {

        let typeval1 = dyn_type_check(&vec1[x]);
        let typeval2 = dyn_type_check(&vec2[x]);

        if typeval1 != "array".to_string() && typeval2 != "array".to_string() {
            if typeval1 != typeval2 {
                return false;
            }
        } else {

            if let (Some(inner_vec1), Some(inner_vec2)) = (
                vec1[x].downcast_ref::<Vec<Box<dyn Any>>>(),
                vec2[x].downcast_ref::<Vec<Box<dyn Any>>>(),
            ) {
                if !are_vecs_equal(inner_vec1, inner_vec2) {
                    return false;
                }
            } else {
                return false;
            }
        }
    }
        
    return true;
}

fn dyn_type_check(value1:&Box<dyn Any>) -> String {
    // Check what type a dynamic variable is 
    match value1 {

        value if value.is::<i64>() => {
            return "int".to_string();
        }

        value if value.is::<String>() => {
            return "string".to_string();
        }

        value if value.is::<Vec<Box<dyn Any>>>() => {
            return "array".to_string();
        }
        
        _ => {
            return "unkown".to_string();
        }
    }
    
}

#[test]
fn test_intergers() {
    let mut expected:i64;
    
    expected = 52;
    assert_eq!(*decode_bencoded_value("i52e").downcast_ref::<i64>().unwrap(),expected);
    expected = -52;
    assert_eq!(*decode_bencoded_value("i-52e").downcast_ref::<i64>().unwrap(),expected);
    expected = 0;
    assert_eq!(*decode_bencoded_value("i0e").downcast_ref::<i64>().unwrap(),expected);
    expected = 51322424;
    assert_eq!(*decode_bencoded_value("i51322424e").downcast_ref::<i64>().unwrap(),expected);

}

#[test]
fn test_strings(){
    let mut expected:String;

    expected = "hell".to_string();
    assert_eq!(*decode_bencoded_value("4:Hello").downcast_ref::<String>().unwrap(),expected);
    expected = "hello".to_string();
    assert_eq!(*decode_bencoded_value("5:Hello").downcast_ref::<String>().unwrap(),expected);
    expected = "hello world".to_string();
    assert_eq!(*decode_bencoded_value("11:Hello world").downcast_ref::<String>().unwrap(),expected);
    expected = "Rust Is AwEsOmE".to_string();
    assert_eq!(*decode_bencoded_value("15:Rust Is AwEsOmE").downcast_ref::<String>().unwrap(),expected);

}

#[test]
fn test_lists() {
    let mut expected: Vec<Box<dyn Any>>;

    expected = vec![Box::new(42), Box::new(52)];
    let returnedresult = decode_bencoded_value("li42ei52ee");
    assert!(compare_boxed_vecs(&returnedresult, &expected));

    expected = vec![Box::new("hello".to_string()), Box::new(42)];
    let returnedresult = decode_bencoded_value("l5:helloi42ee");
    assert!(compare_boxed_vecs(&returnedresult, &expected));

    expected = vec![
        Box::new("a".to_string()),
        Box::new(vec![Box::new("nested".to_string())]),
        Box::new("list".to_string()),
    ];
    let returnedresult = decode_bencoded_value("l1:al6:nestede4:liste");
    assert!(compare_boxed_vecs(&returnedresult, &expected));

    expected = vec![];
    let returnedresult = decode_bencoded_value("le");
    assert!(compare_boxed_vecs(&returnedresult, &expected));
}

/// Helper function to compare `Box<dyn Any>` containing a `Vec<Box<dyn Any>>`
fn compare_boxed_vecs(boxed_value: &Box<dyn Any>, expected: &Vec<Box<dyn Any>>) -> bool {
    if let Some(returned_vec) = boxed_value.downcast_ref::<Vec<Box<dyn Any>>>() {
        are_vecs_equal(returned_vec, expected)
    } else {
        false // If it's not a vector, the test fails
    }
}


#[test]
fn test_dicts(){

}