use std::io::{BufReader, BufWriter, Write};
use std::net::{TcpListener, TcpStream};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use rayon::ThreadPoolBuilder;

mod commands;
mod protocol;
mod types;

use types::Dictionary;


fn handle_client(stream: TcpStream, dictionary: Dictionary) {
    let mut writer = BufWriter::new(&stream);
    let reader = BufReader::new(&stream);
    
    let command = protocol::decode(reader).unwrap();
    dbg!(&command);
    dbg!(&dictionary);
    let response = commands::dispatch(command, &dictionary);
    dbg!(&response);

    match response {
        Ok(r) => {
            let r = r.map(|val| val.respond()).unwrap_or_else(|| "+OK\r\n".to_string());
            dbg!(&r);
            writer.write(r.as_bytes()).expect("Could not send response");
        },
        Err(e) => {
            let e = e.to_string();
            dbg!(&e);
            writer.write(format!("-{}\r\n", e).as_bytes()).expect("Could not send response");
        }
    };
}

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6379")?;
    let dictionary: Dictionary = Arc::new(Mutex::new(BTreeMap::new()));
    let pool = ThreadPoolBuilder::new().num_threads(8).build().unwrap();
       
    // accept connections and process them serially
    for stream in listener.incoming() {
        let stream = stream?;
        let dictionary = dictionary.clone();
        pool.spawn(move || {
            handle_client(stream, dictionary);
        });
    }
    Ok(())
}