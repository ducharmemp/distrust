use std::io::prelude::*;
use std::io::{BufReader, Result as IoResult};
use std::convert::TryFrom;

const LF: u8 = b'\n';
const CRLF: &[u8] = b"\r\n";

#[derive(Debug, PartialEq)]
pub struct Header {
    op: String,
    arg: i64,
}

impl TryFrom<String> for Header {
    fn try_from(line: String) -> Result<Self, ()> {
        let line = line.strip_suffix("\r\n");
        let line = line.expect("Header did not terminate with \\r\\n");

        let mut line = line.chars();
        let (op, arg) = (
          line.next().unwrap().to_string(),
          line.next().unwrap().to_string(),
        );
        Ok(Self { op, arg: arg.parse::<i64>()? })
    }
}

pub fn decode<T: Read>(mut reader: BufReader<T>) -> IoResult<Header> {
    let header = read_header(&mut reader)?;
    Ok(header)
}

fn read_header<T: Read>(reader: &mut BufReader<T>) -> IoResult<Header> {
    let mut header = String::new();
    reader.read_line(&mut header)?;

    Ok(Header::from_string(header))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_decodes_a_message() {
        let buf_reader = BufReader::new("*2\r\n$3\r\nfoo\r\n$3\r\nbar\r\n".as_bytes());
        assert_eq!(
            decode(buf_reader).unwrap(),
            Header { op: "*".to_string(), arg: 2 }
        );
    }

    #[test]
    #[should_panic]
    fn it_panics_on_empty_header() {
        let buf_reader = BufReader::new("\r\n".as_bytes());
        decode(buf_reader);
    }
}
