use std::{
    io::Write,
    net::{TcpListener, TcpStream},
};

fn handle_connection(mut stream: TcpStream) {
    let response = "HTTP/1.1 200 OK\r\n\r\n";
    match stream.write_all(response.as_bytes()) {
        Ok(()) => {
            println!("Sent 200");
        }
        Err(e) => {
            println!("error: {}", e);
        }
    }
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(_stream) => {
                println!("accepted new connection");
                handle_connection(_stream);
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
