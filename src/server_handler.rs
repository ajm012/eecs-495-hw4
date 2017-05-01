use std::net::TcpStream;
use std::io::{Read, Write};

type Input = Vec<String>;

// Central server point - takes input and generates output
pub fn handle_client(stream: TcpStream) {
    let input_str = handle_read(&stream);
    //println!("Returns!");

    let input = parse_input(input_str);
    if (!handle_faulty_input(&input)) {
        let response = "Input not formatted properly. Exiting.";
        match stream.write(response) {
            Ok(_) => println!("Response sent"),
            Err(e) => println!("Failed sending response: {}", e),
        }
    }
    else {
        //println!("{}", input);
        handle_write(stream);
    }
}

// Reads an input to the server
fn handle_read(mut stream: &TcpStream) -> String {
    let mut buf = [0u8 ;4096];
    match stream.read(&mut buf) {
        Ok(_) => {
            let req_str = String::from_utf8_lossy(&buf);
            //println!("{}", req_str.to_string());
            return req_str.to_string();
            },
        Err(e) => {
            println!("Unable to read stream: {}", e);
            return "Error".to_string();
        },
    }
}

// Writes a response given the input
fn handle_write(mut stream: TcpStream) {
    let response = b"HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=UTF-8\r\n\r\n<html><body>Hello world</body></html>\r\n";
    match stream.write(response) {
        Ok(_) => println!("Response sent"),
        Err(e) => println!("Failed sending response: {}", e),
    }
}

// Splits input string into a vector of three elements, seperated by spaces
fn parse_input(i_str: String) -> Input {
    let input = Input::new();
}

// Returns true if Input vector is properly formatted,
// false if otherwise
fn check_faulty_input(input: Input) -> bool {

}