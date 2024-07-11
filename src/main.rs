use std::{
    env,
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

fn send_octet_stream(stream: &mut TcpStream, contents: &[u8]) -> std::io::Result<()> {
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\n\r\n",
        contents.len(),
    );
    stream.write_all(response.as_bytes())?;
    stream.write_all(contents)?;
    Ok(())
}

fn handle_connection(mut stream: TcpStream, directory: &str) -> std::io::Result<()> {
    let buf_reader = BufReader::new(&mut stream);

    let mut lines = buf_reader.lines();
    let request_line = lines.next().unwrap().unwrap();

    let _host_line = lines.next().unwrap().unwrap();
    let user_agent_line = lines.next().unwrap().unwrap();

    println!("{}", request_line);
    println!("{}", user_agent_line);
    println!("Serving files from directory: {}", directory);

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
        "files" => {
            let file = route.split('/').nth(2).unwrap();
            let path = format!("{}/{}", directory, file);

            let contents = match std::fs::read(path) {
                Ok(contents) => contents,
                Err(_) => {
                    let response = "HTTP/1.1 404 Not Found\r\n\r\n";
                    stream.write_all(response.as_bytes())?;
                    return Ok(());
                }
            };
            send_octet_stream(&mut stream, &contents)?;
        }
        _ => {
            let response = "HTTP/1.1 404 Not Found\r\n\r\n";
            stream.write_all(response.as_bytes())?;
        }
    }

    Ok(())
}

fn get_directory() -> Option<String> {
    let args: Vec<String> = env::args().collect();

    for n in 1..args.len() {
        let arg = args.get(n)?;
        if arg == "--directory" {
            let directory = args.get(n + 1)?;
            println!("Serving files from directory: {}", directory);
            return Some(directory.into());
        }
    }

    None
}

fn main() -> std::io::Result<()> {
    let directory = get_directory().unwrap_or("files".into());

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(_stream) => {
                println!("accepted new connection");

                if let Err(e) = handle_connection(_stream, &directory) {
                    println!("error: {}", e);
                }
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }

    Ok(())
}
