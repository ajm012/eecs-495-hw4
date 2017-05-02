use std::net::TcpStream;
use std::io::{Read, Write};
use std::fs::{File, metadata};
use std::string::String;

extern crate regex;
use self::regex::Regex;

// Central server function - takes and checks input, generates output
pub fn handle_client(stream: TcpStream) {
    let input_str = handle_read(&stream);
    match parse_request(&input_str) {
        None => handle_bad_request(stream),
        Some(path) => handle_valid_request(stream, &path),
    }
}

// Processes a bad request and returns an HTTP 400 error code
fn handle_bad_request(mut stream: TcpStream) {
    let response = b"HTTP/1.1 400 Bad Request\n";
    match stream.write(response) {
        Ok(_) => println!("400 Sent - Bad Request"),
        Err(e) => println!("Failed sending response: {}", e),
    }    
}

// Processes a valid request for a file and returns the file or 
// an error (forbidden or not found)
fn handle_valid_request(mut stream: TcpStream, path: &str) {
    let is_html = is_html(path);
    let response = get_file(path);
    if response == "403" { // File restricted
        let response = b"HTTP/1.0 403 Forbidden\n";
        match stream.write(response) {
            Ok(_) => println!("403 Sent - Forbidden"),
            Err(e) => println!("Failed sending response: {}", e),
        }
    } else if response == "404" { // File not found
        let response = b"HTTP/1.0 404 File Not Found\n";
        match stream.write(response) {
            Ok(_) => println!("404 Sent - File Not Found"),
            Err(e) => println!("Failed sending response: {}", e),
        }
    } else {
        handle_write(stream, response.clone(), is_html, response.len());
    }
}

// Reads an input to the server
fn handle_read(mut stream: &TcpStream) -> String {
    let mut buf = [0u8 ;4096];
    match stream.read(&mut buf) {
        Ok(_) => {
            let req_str = String::from_utf8_lossy(&buf);
            return req_str.to_string();
            },
        Err(e) => {
            println!("Unable to read stream: {}", e);
            return "Error".to_string();
        },
    }
}

// Writes a response given the input
fn handle_write(mut stream: TcpStream, response: String, html: bool, len: usize) {
    //let response2 = b"HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=UTF-8\r\n\r\n<html><body>Hello world</body></html>\r\n";
    let mut text_type = "plain";
    if html {text_type = "html";}
    let step0 = format!("{}{}{}{}{}","HTTP/1.0 200 OK\r\nWeb-Server/0.1\r\nContent-Type: text/",text_type,"\r\nContent-length: ", len, "\r\n\r\n");
    let step1 = format!("{}{}",step0, response);
    let step2 = format!("{}{}", step1, "</body></html>\r\n");
    match stream.write(step2.as_bytes()) {
        Ok(_) => println!("Response sent"),
        Err(e) => println!("Failed sending response: {}", e),
    }
}

// Parses a GET request of the form: GET $web_file_path$ HTTP...
// and formats $web_file_path$ into a standard file path. Returns None if 
// the request does not follow the above format, else returns Some(file_path)
fn parse_request(request: &str) -> Option<String> {
    let re = Regex::new(r"GET (.*) HTTP(?:.*)").unwrap();
    match re.captures(request) {
        Some(caps) => Some(caps.get(1).unwrap().as_str().to_string().replace("%20", " ")),
        _ => None
    }
}

#[cfg(test)]
mod parse_request_tests {
    use super::*;

    #[test]
    fn parse_valid_request_test() {
        let request = "GET /Users/feelmyears/Google%20Drive/Spring%20Quarter/EECS%20395/Homework/eecs-495-hw4/src/main.rs HTTP".to_string();
        let result = parse_request(&request);
        assert_eq!(result.unwrap(), "/Users/feelmyears/Google Drive/Spring Quarter/EECS 395/Homework/eecs-495-hw4/src/main.rs".to_string());
    }
}     

// Searches for the correct file, returns "error" is not found
// If filename leads to a directory, recursively searches for a
// index file (.html, .shtml, .txt)
fn get_file(filename: &str) -> String {
    println!("Attempting to open {}", filename);

    let md = match metadata(filename) {
        Ok(file) => file,
        Err(_) => {return "404".to_string();}, 
    };
    if md.is_dir() {
        println!("Directory found...attempting to find index.html");
        let check1 = get_file(format!("{}{}", filename, "/index.html").as_str());
        if check1 != "404" {return check1;}
        println!("Directory found...attempting to find index.shtml");
        let check2 = get_file(format!("{}{}", filename, "/index.shtml").as_str());
        if check2 != "404" {return check2;}
        println!("Directory found...attempting to find index.txt");
        let check3 = get_file(format!("{}{}", filename, "/index.txt").as_str());
        if check3 != "404" {return check3;}
        else {return check3;}
    }

    let mut file = match File::open(filename) {
        Ok(file) => file,
        Err(_) => {return "404".to_string();}, 
    };
    //let mut file = File::open("/Users/andrewmcconnell/Desktop/Rust/eecs-495-hw4/src/main.rs").expect("Unable to open the file");
    let mut contents = String::new();
    match file.read_to_string(&mut contents) {
        Ok(_) => {},
        Err(_) => {return "403".to_string();},
    }
    return contents;
}

// Checks if the file is of type .html
// If so returns true
fn is_html(filename: &str) -> bool {
    let check_rev: String = filename.chars().rev().take(5).collect(); // returns a reverse string (i.e. "lmth." not ".html")
    let check: String = check_rev.chars().rev().take(5).collect(); 
    if check == ".html" {println!("HTML found"); return true;}
    return false;
}