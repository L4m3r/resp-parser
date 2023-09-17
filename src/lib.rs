mod deserializer;
use deserializer::{Value, from_bytes, from_string, from_stream};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = from_string("*2\r\n$4\r\nECHO\r\n$3\r\nhey\r\n");
        assert_eq!(result.is_ok(), true);
        let result = result.unwrap();
        let correct = Value::Array(
            vec![
                Value::BulkString(Vec::from("ECHO".as_bytes())),
                Value::BulkString(Vec::from("hey".as_bytes())),
            ]
        );
        assert_eq!(result, correct);
    }
}
