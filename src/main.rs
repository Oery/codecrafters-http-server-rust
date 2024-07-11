use std::{
    io::{prelude::*, BufReader, Write},
    net::{TcpListener, TcpStream},
};

fn handle_connection(mut stream: TcpStream) -> std::io::Result<()> {
    let buf_reader = BufReader::new(&mut stream);
    let request_line = buf_reader.lines().next().unwrap().unwrap();

    println!("{}", request_line);

    let route = request_line.split(' ').nth(1).unwrap();
    let base_route = route.split('/').nth(1).unwrap();

    println!("base_route: {}", base_route);

    match base_route {
        "" => {
            let response = "HTTP/1.1 200 OK\r\n\r\n";
            stream.write_all(response.as_bytes())?;
        }
        "echo" => {
            let message = route.split('/').nth(2).unwrap();
            let length = message.len();
            let response = format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {length}\r\n\r\n{message}");
            stream.write_all(response.as_bytes())?;
        }
        _ => {
            let response = "HTTP/1.1 404 Not Found\r\n\r\n";
            stream.write_all(response.as_bytes())?;
        }
    }

    Ok(())
}

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(_stream) => {
                println!("accepted new connection");
                match handle_connection(_stream) {
                    Ok(_) => {}
                    Err(e) => {
                        println!("error: {}", e);
                    }
                };
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }

    Ok(())
}
