use std::net::TcpStream;
use std::io::{Read, Write};
//use std::io::prelude::*;
use std::fs::File;

type Input<'a> = Vec<&'a str>;

// Central server point - takes input and generates output
pub fn handle_client(mut stream: TcpStream) {
    let input_str = handle_read(&stream);
    //println!("Returns!");

    let input = parse_input(&input_str);
    if !check_faulty_input(&input) {
        let response = b"Input not formatted properly. Exiting.\n";
        match stream.write(response) {
            Ok(_) => println!("Bad response sent"),
            Err(e) => println!("Failed sending response: {}", e),
        }
    }
    else {
        //println!("{}", input);
        let response = get_file(input[1]);
        handle_write(stream, response);
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
fn handle_write(mut stream: TcpStream, response: String) {
    //let response2 = b"HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=UTF-8\r\n\r\n<html><body>Hello world</body></html>\r\n";
    match stream.write(response.as_bytes()) {
        Ok(_) => println!("Response sent"),
        Err(e) => println!("Failed sending response: {}", e),
    }
}

// Splits input string into a vector of three elements, seperated by spaces
fn parse_input(i_str: &str) -> Input {
    return i_str.split(" ").collect();
}

// Returns true if Input vector is properly formatted,
// false if otherwise
fn check_faulty_input(input: &Input) -> bool {
    if input.len() != 3 {return false;}
    if input[0] != "GET" {return false;}
    //if input[2] != "HTTP" {return false;} // fix this to allow forward-compatibility
    return true;
}

fn get_file(filename: &str) -> String {
    println!("Attempting to open {}", filename);
    //let mut file = File::open("/Users/andrewmcconnell/Desktop/Rust/eecs-495-hw4/src/main.rs").expect("Unable to open the file");
    let mut file = File::open(filename).expect("Unable to open the file");
    let mut contents = String::new();
    file.read_to_string(&mut contents).expect("Unable to read the file");
    //println!("{}", contents);
    return contents;
}