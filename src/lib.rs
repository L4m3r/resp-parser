pub mod deserializer;

// TODO: make integration tests
#[cfg(test)]
mod tests {
    use super::*;
    use deserializer::*;

    #[test]
    fn parse_array() {
        let result = from_string("*2\r\n$4\r\nECHO\r\n$3\r\nhey\r\n");
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
