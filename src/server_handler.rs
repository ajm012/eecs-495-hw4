use std::net::TcpStream;
use std::io::{Read, Write};
use std::fs::{File, metadata};
use std::string::String;
use std::path::PathBuf;

extern crate regex;
use self::regex::Regex;

pub enum ResponseStatus {
    OkStatus,
    BadRequestStatus,
    ForbiddenStatus,
    NotFoundStatus
}

pub struct Response {
    pub status: ResponseStatus,
    pub content_type: String,
    pub content_length: usize,
    pub content: String
}

// Central server function - takes and checks input, generates output
pub fn handle_request(stream: TcpStream) -> Response {
    let request = read_from_stream(&stream);
    match parse_get_request(&request) {
        None => handle_bad_request(stream),
        Some(path) => handle_get(stream, &path),
    }
}

// Reads an input to the server
fn read_from_stream(mut stream: &TcpStream) -> String {
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

// Parses a GET request of the form: GET $web_file_path$ HTTP...
// and formats $web_file_path$ into a standard file path. Returns None if 
// the request does not follow the above format, else returns Some(file_path)
fn parse_get_request(request: &str) -> Option<PathBuf> {
    let re = Regex::new(r"GET (.*) HTTP(?:.*)").unwrap();
    match re.captures(request) {
        Some(caps) => Some(PathBuf::from(caps.get(1).unwrap().as_str().to_string().replace("%20", " "))),
        _ => None
    }
}

fn handle_get(stream: TcpStream, path: &PathBuf) -> Response {
    match get_file(path) {
        None => handle_file_not_found_request(stream),
        Some((file, extension)) => {
            let is_html = extension == "html";
            match get_file_contents(file) {
                None => handle_forbidden_request(stream),
                Some(contents) => handle_content_request(stream, contents, is_html)
            }
        }
    }
}

fn respond(mut stream: TcpStream, response: &str) {
    match stream.write(response.as_bytes()) {
        Ok(_) => println!("Successfully responded"),
        Err(e) => println!("Failed sending response: {}", e),
    }    
}

fn handle_bad_request(stream: TcpStream) -> Response {
    println!("400 Sent - Bad Request");
    let response = "HTTP/1.0 400 Bad Request\n";
    respond(stream, response);
    Response {
        status: ResponseStatus::BadRequestStatus,
        content_type: "".to_string(),
        content_length: 0,
        content: "".to_string()
    }
}   


fn handle_file_not_found_request(stream: TcpStream) -> Response {
    println!("404 Sent - Fire Not Found");
    let response = "HTTP/1.0 404 File Not Found\n";
    respond(stream, response);
    Response {
        status: ResponseStatus::NotFoundStatus,
        content_type: "".to_string(),
        content_length: 0,
        content: "".to_string()
    }
}

fn handle_forbidden_request(stream: TcpStream) -> Response {
    println!("403 Sent - Forbidden");
    let response = "HTTP/1.0 403 Forbidden\n";
    respond(stream, response);
    Response {
        status: ResponseStatus::ForbiddenStatus,
        content_type: "".to_string(),
        content_length: 0,
        content: "".to_string()
    }
}

fn handle_content_request(stream: TcpStream, contents: String, is_html: bool) -> Response {
    println!("200 Sent - Content");
    let text_type = match is_html {
        true => "html",
        false => "plain"
    };

    let len = contents.len();
    let step0 = format!("{}{}{}{}{}","HTTP/1.0 200 OK\r\nWeb-Server/0.1\r\nContent-Type: text/",text_type,"\r\nContent-length: ", len, "\r\n\r\n");
    let step1 = format!("{}{}",step0, contents);
    let step2 = format!("{}{}", step1, "</body></html>\r\n");

    respond(stream, &step2);
    Response {
        status: ResponseStatus::OkStatus,
        content_type: text_type.to_string(),
        content_length: len,
        content: contents
    }
}   

// Searches for the correct file, returns "error" is not found
// If filename leads to a directory, recursively searches for a
// index file (.html, .shtml, .txt)
fn get_file(path: &PathBuf) -> Option<(File, String)> {
    println!("Attempting to open {}", path.to_str().unwrap());
    let md = match metadata(path) {
        Ok(meta) => meta,
        Err(_) => return None,
    };

    if md.is_dir() {
        return get_directory_index_file(path);
    }
    
    let extension = path.extension().unwrap().to_str().unwrap().to_string();
    match File::open(path) {
        Ok(file) => Some((file, extension)),
        Err(_) => None
    }
}

fn get_directory_index_file(path: &PathBuf) ->  Option<(File, String)>  {
    println!("Directory found...attempting to find index file");
    let file_types = vec!["index.html", "index.shtml", "index.txt"];
    for &t in &file_types {
        println!("Attempting to find {}", t);
        let index_file = get_file(&path.join(t));
        if index_file.is_some() {
            return index_file;
        }
    }

    return None;
}

fn get_file_contents(mut file: File) -> Option<String> {
    let mut contents = String::new();
    match file.read_to_string(&mut contents) {
        Ok(_) => Some(contents),
        Err(_) => None
    }
}

#[cfg(test)]
mod parse_get_request_tests {
    use super::*;

    #[test]
    fn parse_valid_get_request() {
        let request = "GET /Users/feelmyears/Google%20Drive/Spring%20Quarter/EECS%20395/Homework/eecs-495-hw4/src/main.rs HTTP";
        let result = parse_get_request(request);
        assert_eq!(result.unwrap().to_str().unwrap(), "/Users/feelmyears/Google Drive/Spring Quarter/EECS 395/Homework/eecs-495-hw4/src/main.rs");
    }
}  