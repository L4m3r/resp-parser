use std::io::Error as IoError;
use std::io::Read;
use std::result::Result as StdResult;

pub type Result<T> = StdResult<T, Error>;

#[derive(Debug)]
pub enum Error {
    IoError(IoError),
    InvalidValue(String),
    EndOfStream,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Value {
    String(String),      // https://redis.io/docs/reference/protocol-spec/#simple-strings
    Error(String),       // https://redis.io/docs/reference/protocol-spec/#simple-errors
    Integer(i64),        // https://redis.io/docs/reference/protocol-spec/#integers
    BulkString(Vec<u8>), // https://redis.io/docs/reference/protocol-spec/#bulk-strings
    Array(Vec<Value>),   // https://redis.io/docs/reference/protocol-spec/#arrays
}
#[derive(Debug)]
struct Deserialer<R: Read> {
    stream: R,
}

impl<'a, R: Read> Deserialer<R> {
    pub fn new(stream: R) -> Deserialer<R> {
        Deserialer { stream }
    }

    fn peek_byte(&mut self) -> Result<u8> {
        let mut buf = [0; 1];
        if 1 != self.stream.read(&mut buf).map_err(Error::IoError)? {
            return Err(Error::EndOfStream);
        }
        Ok(buf[0])
    }

    fn check_ending(&mut self) -> Result<()> {
        if self.peek_byte()? != b'\n' {
            return Err(Error::InvalidValue(
                "Integer does not end with \\r\\n".to_string(),
            ));
        }
        Ok(())
    }

    fn parse_string(&mut self) -> Result<String> {
        let mut result = vec![];
        loop {
            match self.peek_byte()? {
                b'\r' => {
                    self.check_ending()?;
                    let out_str = String::from_utf8(result).map_err(|_| {
                        Error::InvalidValue("Non UTF-8 integer encoding".to_string())
                    })?;
                    return Ok(out_str);
                }
                b'\n' => {
                    return Err(Error::InvalidValue("String contain \\n".to_string()));
                }
                c => {
                    result.push(c);
                }
            }
        }
    }

    fn parse_error(&mut self) -> Result<String> {
        self.parse_string()
    }

    fn parse_integer(&mut self) -> Result<i64> {
        let mut result = vec![];
        loop {
            match self.peek_byte()? {
                b'\r' => {
                    self.check_ending()?;
                    let len_str = String::from_utf8(result).map_err(|_| {
                        Error::InvalidValue("Non UTF-8 integer encoding".to_string())
                    })?;
                    let len_int = len_str.parse::<i64>().map_err(|_| {
                        Error::InvalidValue(format!("Can't parse `{}` as integer", len_str))
                    })?;

                    return Ok(len_int);
                }
                c => {
                    result.push(c);
                }
            }
        }
    }

    fn parse_bulk(&mut self) -> Result<Vec<u8>> {
        let length = self.parse_integer()?;
        let mut resutt = vec![];
        for _ in 0..length {
            let c = self.peek_byte()?;
            resutt.push(c);
        }
        if self.peek_byte()? != b'\r' {
            return Err(Error::InvalidValue(
                "Integer does not end with \\r\\n".to_string(),
            ));
        }
        self.check_ending()?;
        Ok(resutt)
    }

    fn parse_array(&mut self) -> Result<Vec<Value>> {
        let length = self.parse_integer()?;
        let mut result = vec![];
        for _ in 0..length {
            let value = self.parse()?;
            result.push(value);
        }
        Ok(result)
    }

    fn parse(&mut self) -> Result<Value> {
        match self.peek_byte()? {
            b'+' => Ok(Value::String(self.parse_string()?)),
            b'-' => Ok(Value::String(self.parse_error()?)),
            b':' => Ok(Value::Integer(self.parse_integer()?)),
            b'$' => Ok(Value::BulkString(self.parse_bulk()?)),
            b'*' => Ok(Value::Array(self.parse_array()?)),
            c => Err(Error::InvalidValue(format!("Invalid character {}", c))),
        }
    }
}

pub fn from_stream<R: Read>(stream: R) -> Result<Value> {
    let mut d = Deserialer::new(stream);
    d.parse()
}

pub fn from_bytes(data: &[u8]) -> Result<Value> {
    from_stream(data)
}

pub fn from_string(data: &str) -> Result<Value> {
    from_bytes(data.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_int(data: &str) -> Result<i64> {
        let mut d = Deserialer::new(data.as_bytes());
        d.parse_integer()
    }

    fn setup_string(data: &str) -> Result<String> {
        let mut d = Deserialer::new(data.as_bytes());
        d.parse_string()
    }

    fn setup_bulk(data: &str) -> Result<Vec<u8>> {
        let mut d = Deserialer::new(data.as_bytes());
        d.parse_bulk()
    }
    #[test]
    fn parse_integer() {
        let result = setup_int("1234567890\r\n");
        assert!(result.is_ok(), "{:?}", result.err().unwrap());
        let result = result.unwrap();
        let correct = 1234567890;
        assert_eq!(result, correct);
    }

    #[test]
    fn parse_negative_integer() {
        let result = setup_int("-1234567890\r\n");
        assert!(result.is_ok(), "{:?}", result.err().unwrap());
        let result = result.unwrap();
        let correct = -1234567890;
        assert_eq!(result, correct);
    }

    #[test]
    fn parse_zero() {
        let result = setup_int("0\r\n");
        assert!(result.is_ok(), "{:?}", result.err().unwrap());
        let result = result.unwrap();
        let correct = 0;
        assert_eq!(result, correct);
    }

    #[test]
    fn parse_invalid_integer() {
        let data = "r\r\n";
        let result = setup_int(&data);
        assert!(
            result.is_err(),
            "String {} shouldnt parse to integer. Found: {:?}",
            data,
            result.unwrap()
        );
    }

    #[test]
    fn parse_integer_end_of_stream() {
        let data = "8122\r";
        let result = setup_int(&data);
        assert!(
            result.is_err(),
            "String {} should raise an error. Found: {:?}",
            data,
            result.unwrap()
        );
    }

    // TODO: write more tests for string
    #[test]
    fn parse_string() {
        let data = "OK\r\n";
        let result = setup_string(&data);
        assert!(result.is_ok(), "{:?}", result.err().unwrap());
        let result = result.unwrap();
        let correct = "OK".to_string();
        assert_eq!(result, correct);
    }

    #[test]
    fn parse_string_end_of_stream() {
        let data = "OK\r";
        let result = setup_int(&data);
        assert!(
            result.is_err(),
            "String {} should raise an error. Found: {:?}",
            data,
            result.unwrap()
        );
    }

    #[test]
    fn parse_bulk_string() {
        let data = "4\r\nECHO\r\n";
        let result = setup_bulk(&data);
        assert!(result.is_ok(), "{:?}", result.err().unwrap());
        let result = result.unwrap();
        let correct = "ECHO".as_bytes();
        assert_eq!(result, correct);
    }
}
