use std::io::{BufReader, Read, BufRead};
use std::convert::TryFrom;

use anyhow::{Result, Context, Error};

const CRLF: &'static str = "\r\n";

#[derive(Debug, PartialEq)]
pub struct Header {
    op: String,
    pub count: i64,
}

impl TryFrom<String> for Header {
    type Error = Error;

    fn try_from(line: String) -> Result<Self> {
        let line = line.strip_suffix(CRLF);
        let line = line.context("Header did not terminate with \\r\\n")?;

        let mut line = line.chars();
        let (op, count) = (
          line.next().ok_or(Error::msg("Could not find operator"))?.to_string(),
          line.next().ok_or(Error::msg("Could not find operator count"))?.to_string(),
        );
        Ok(Self { op, count: count.parse::<i64>()? })
    }
}

pub fn decode<T: Read>(mut reader: BufReader<T>) -> Result<Vec<String>> {
    let header = read_header(&mut reader)?;
    let mut lines = vec![];
    
    for _ in 0..header.count {
        lines.push(read_line(&mut reader)?);
    }

    Ok(lines)
}

fn read_header<T: Read>(reader: &mut BufReader<T>) -> Result<Header> {
    let mut header = String::new();
    reader.read_line(&mut header)?;

    Header::try_from(header)
}

fn read_line<T: Read>(reader: &mut BufReader<T>) -> Result<String> {
    let mut line = String::new();
    reader.read_line(&mut line)?;

    let header = Header::try_from(line)?;
    let mut line = String::new();
    reader.read_line(&mut line)?;
    let line = line.strip_suffix("\r\n").ok_or(Error::msg("Line did not end with line feed terminator"))?;
    assert!(line.as_bytes().len() == header.count as usize);

    Ok(line.to_string())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_decodes_a_message() {
        let buf_reader = BufReader::new("*2\r\n$3\r\nfoo\r\n$3\r\nbar\r\n".as_bytes());
        assert_eq!(
            decode(buf_reader).unwrap(),
            vec!["foo", "bar"]
        );
    }

    #[test]
    fn it_decodes_a_set_message() {
        let buf_reader = BufReader::new("*3\r\n$3\r\nSET\r\n$1\r\nA\r\n$5\r\nhello\r\n".as_bytes());
        assert_eq!(
            decode(buf_reader).unwrap(),
            vec!["SET", "A", "hello"]
        );
    }

    #[test]
    fn it_errors_on_missing_line_feed() {
        let buf_reader = BufReader::new("*3".as_bytes());
        assert_eq!(decode(buf_reader).unwrap_err().to_string(), "Header did not terminate with \\r\\n");
    }

    #[test]
    fn it_errors_on_missing_operator() {
        let buf_reader = BufReader::new("\r\n".as_bytes());
        assert_eq!(decode(buf_reader).unwrap_err().to_string(), "Could not find operator");
    }

    #[test]
    fn it_errors_on_missing_count() {
        let buf_reader = BufReader::new("*\r\n".as_bytes());
        assert_eq!(decode(buf_reader).unwrap_err().to_string(), "Could not find operator count");
    }
}
