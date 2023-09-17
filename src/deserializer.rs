use std::result::Result as StdResult;
use std::io::Error as IoError;
use std::io::Read;

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
        Deserialer {
            stream,
        }
    }

    fn parse_string(&mut self) -> Result<Value> {
        todo!();
    }

    fn parse_error(&mut self) -> Result<Value> {
        todo!()
    }

    fn parse_integer(&mut self) -> Result<Value> {
        todo!()
    }

    fn parse_bulk(&mut self) -> Result<Value> {
        todo!()
    }

    fn parse_array(&mut self) -> Result<Value> {
        todo!()
    }

    fn parse(&mut self) -> Result<Value> {
        let mut buf = [0; 1];
        if 1 != self.stream.read(&mut buf).map_err(Error::IoError)? {
            return Err(Error::EndOfStream);
        }

        match buf[0] {
            b'+' => self.parse_string(),
            b'-' => self.parse_error(),
            b':' => self.parse_integer(),
            b'$' => self.parse_bulk(),
            b'*' => self.parse_array(),
            c => Err(Error::InvalidValue(format!("Invalid character {}", c))),
        }
    }
}

pub fn from_bytes(data: &[u8]) -> Result<Value> {
    let mut d = Deserialer::new(data);
    d.parse()
}

pub fn from_string(data: &str) -> Result<Value> {
    from_bytes(data.as_bytes())
}

pub fn from_stream<R: Read>(stream: R) -> Result<Value> {
    todo!();
}
