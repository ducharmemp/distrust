use std::io::{BufReader, BufWriter, Write};
use std::net::{TcpListener, TcpStream};
use std::collections::BTreeMap;

mod commands;
mod protocol;


fn handle_client(stream: TcpStream, dictionary: &mut BTreeMap<String, String>) {
    let mut writer = BufWriter::new(&stream);
    let reader = BufReader::new(&stream);
    // Read a message from the stream
    // Decode the message
    let command = protocol::decode(reader).unwrap();
    dbg!(&command);
    let response = commands::dispatch(command, dictionary);
    dbg!(&response);
    dbg!(dictionary);

    match response {
        Ok(r) => {
            let r = r.map(|val| val.respond()).unwrap_or_else(|| "OK".to_string());
            writer.write(format!("+{}\r\n", r).as_bytes()).expect("Could not send response");
        },
        Err(e) => {
            let e = e.to_string();
            writer.write(format!("-{}\r\n", e).as_bytes()).expect("Could not send response");
        }
    };
}

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6379")?;
    let mut dictionary: BTreeMap<String, String> = BTreeMap::new();
       
    // accept connections and process them serially
    for stream in listener.incoming() {
        handle_client(stream?, &mut dictionary);
    }
    Ok(())
}