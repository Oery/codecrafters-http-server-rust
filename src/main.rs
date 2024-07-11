use std::{
    io::{prelude::*, BufReader, Write},
    net::{TcpListener, TcpStream},
};

fn send_text(stream: &mut TcpStream, text: &str) -> std::io::Result<()> {
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
        text.len(),
        text
    );
    stream.write_all(response.as_bytes())
}

fn handle_connection(mut stream: TcpStream) -> std::io::Result<()> {
    let buf_reader = BufReader::new(&mut stream);

    let mut lines = buf_reader.lines();
    let request_line = lines.next().unwrap().unwrap();

    let _host_line = lines.next().unwrap().unwrap();
    let user_agent_line = lines.next().unwrap().unwrap();

    println!("{}", request_line);
    println!("{}", user_agent_line);

    let route = request_line.split(' ').nth(1).unwrap();
    let base_route = route.split('/').nth(1).unwrap();

    match base_route {
        "" => {
            let response = "HTTP/1.1 200 OK\r\n\r\n";
            stream.write_all(response.as_bytes())?;
        }
        "user-agent" => {
            let user_agent = user_agent_line.split(": ").nth(1).unwrap();
            send_text(&mut stream, user_agent)?;
        }
        "echo" => {
            let message = route.split('/').nth(2).unwrap();
            send_text(&mut stream, message)?;
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
