use std::net::{TcpListener, TcpStream};

mod protocol;


fn handle_client(stream: TcpStream) {
    // Read a message from the stream
    // Decode the message
    // Dispatch the message
    // Encode the response
    // Return
}

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6379")?;
       
    // accept connections and process them serially
    for stream in listener.incoming() {
        handle_client(stream?);
    }
    Ok(())
}