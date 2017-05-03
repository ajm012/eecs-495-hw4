use std::net::TcpStream;
use std::io::{Read, Write};
use std::fs::{File, metadata};
use std::string::String;
use std::path::PathBuf;

extern crate regex;
use self::regex::Regex;

extern crate time;
use self::time::{now_utc, Tm};

pub struct Response {
    pub status_code: usize,
    pub request: String,
    pub response_size: usize,
    pub time: Tm
}

// Central server function - takes and checks input, generates output
pub fn handle_request(stream: TcpStream) -> Response {
    let request = read_from_stream(&stream);
    let mut response = Response {
        status_code: 0,
        request: request.clone(),
        response_size: 0,
        time: now_utc()
    };

    match parse_get_request(&request) {
        None => handle_bad_request(stream, &mut response),
        Some(path) => handle_get_request(stream, &mut response, &path),
    }

    response
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
    let re = Regex::new(r"GET (/\S+) HTTP(?:.*)").unwrap();
    match re.captures(request) {
        Some(caps) => Some(PathBuf::from(caps.get(1).unwrap().as_str().to_string().replace("%20", " "))),
        _ => None
    }
}

// Handles a properly formatted get request
fn handle_get_request(stream: TcpStream, response: &mut Response, path: &PathBuf) {
    match get_file(path) {
        None => handle_file_not_found_request(stream, response),
        Some((file, extension)) => {
            let is_html = extension == "html";
            match get_file_contents(file) {
                None => handle_forbidden_request(stream, response),
                Some(contents) => handle_content_request(stream, response, contents, is_html)
            }
        }
    }
}

// Sends a response over to a given TcpStream
fn respond(mut stream: TcpStream, response: &str) {
    match stream.write(response.as_bytes()) {
        Ok(_) => println!("Successfully responded"),
        Err(e) => println!("Failed sending response: {}", e),
    }    
}

// Responds with a 400 Bad Request response
fn handle_bad_request(stream: TcpStream, response: &mut Response) {
    println!("400 Sent - Bad Request");
    respond(stream, "HTTP/1.0 400 Bad Request\n");
    response.status_code = 400;
}   

// Responds with a 404 File Not Found response
fn handle_file_not_found_request(stream: TcpStream, response: &mut Response) {
    println!("404 Sent - Fire Not Found");
    respond(stream, "HTTP/1.0 404 File Not Found\n");
    response.status_code = 404;
}

// Responds with a 403 Forbidden response
fn handle_forbidden_request(stream: TcpStream, response: &mut Response) {
    println!("403 Sent - Forbidden");
    respond(stream, "HTTP/1.0 403 Forbidden\n");
    response.status_code = 403;
}

// Responds with the correctly requested content (either html or plain)
fn handle_content_request(stream: TcpStream, response: &mut Response, contents: String, is_html: bool) {
    println!("200 Sent - Content");
    let text_type = match is_html {
        true => "html",
        false => "plain"
    };

    let len = contents.len();
    let content_response = format!("HTTP/1.0 200 OK\r\nWeb-Server/1.0\r\nConent-type: text/{}\r\nContent-length: {}\r\n\r\n{}\n", text_type, len, contents);

    respond(stream, &content_response);
    response.status_code = 200;
    response.response_size = len;
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

// Searches a directory for an index file with extension html, shtml, or txt
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

// Gets the contents of a given file and puts it in a String. 
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
    fn parse_spaced_get_request() {
        let request = "GET /Users/feelmyears/Google%20Drive/Spring%20Quarter/EECS%20395/Homework/eecs-495-hw4/src/main.rs HTTP\n";
        let expected = "/Users/feelmyears/Google Drive/Spring Quarter/EECS 395/Homework/eecs-495-hw4/src/main.rs";

        assert_parse(request, expected);
    }

    #[test]
    fn parse_unspaced_get_request() {
        let request = "GET /this/is/a/path/index.html HTTP\n";
        let expected = "/this/is/a/path/index.html";
        assert_parse(request, expected);
    }

    #[test]
    fn parse_extra_info_get_request() {
        let request = "GET /this/is/a/path/index.html HTTP/1.1\n blah blah";
        let expected = "/this/is/a/path/index.html";
        assert_parse(request, expected);
    }

    #[test]
    fn parse_invalid_get_requests() {
        let invalid_requests = vec![
            "GET HTTP",
            "GETHTTP",
            "GET  /too/many/spaces/lhs HTTP",
            "GET /too/many/spaces/rhs  HTTP",
            "PUT /wrong/request HTTP",
            "GET /incomplete/request ",
            "GET /misspelled/request HTP",
        ];
        
        for i in invalid_requests {
            assert_eq!(parse_get_request(i), None);
        }
    }

    fn assert_parse(request: &str, expected: &str) {
        assert_eq!(parse_get_request(request).unwrap().to_str().unwrap(), expected);
    }
}  

#[cfg(test)]
mod get_file_tests {
    use super::*;
    use std::fs::{remove_dir_all, create_dir_all};

    #[test]
    fn get_existing_file_test() {
        let directory = "./test4/directory/";
        let filename = "./test4/directory/foo.fee";

        {
            create_dir_all(directory).expect("Failed to create directory"); 
            let file = File::create(filename);
        }

        let (file, extension) = get_file(&PathBuf::from(filename)).unwrap();
        assert_eq!(extension, "fee");

        remove_dir_all("./test4/");
    }

    #[test]
    fn get_nonexisting_file_test() {
        let directory = "./test5/directory/";
        let filename = "./test5/directory/imnotreal.txt";

        create_dir_all(directory).expect("Failed to create directory"); 
        assert!(get_file(&PathBuf::from(filename)).is_none());

        remove_dir_all("./test5/");
    }
}

#[cfg(test)]
mod get_directory_index_file_tests {
    use super::*;
    use std::fs::{remove_dir_all, create_dir_all};

    #[test]
    fn get_html_index_file_test() {
        let directory = "./test/directory/";
        let filename = "./test/directory/index.html";

        {
            create_dir_all(directory).expect("Failed to create directory"); 
            let file = File::create(filename);
        }

        let (file, extension) = get_directory_index_file(&PathBuf::from(directory)).unwrap();
        assert_eq!(extension, "html");

        remove_dir_all("./test/");
    }

    #[test]
    fn get_shtml_index_file_test() {
        let directory = "./test1/directory/";
        let filename = "./test1/directory/index.shtml";

        {
            create_dir_all(directory).expect("Failed to create directory"); 
            let file = File::create(filename);
        }

        let (file, extension) = get_directory_index_file(&PathBuf::from(directory)).unwrap();
        assert_eq!(extension, "shtml");

        remove_dir_all("./test1/");
    }

    #[test]
    fn get_txt_index_file_test() {
        let directory = "./test2/directory/";
        let filename = "./test2/directory/index.txt";

        {
            create_dir_all(directory).expect("Failed to create directory"); 
            let file = File::create(filename);
        }

        let (file, extension) = get_directory_index_file(&PathBuf::from(directory)).unwrap();
        assert_eq!(extension, "txt");

        remove_dir_all("./test2/");
    }

    #[test]
    fn no_index_file_test() {
        let directory = "./test3/directory/";
        let filename = "./test3/directory/blah.txt";

        {
            create_dir_all(directory).expect("Failed to create directory"); 
            let file = File::create(filename);
        }

        assert!(get_directory_index_file(&PathBuf::from(directory)).is_none());
        remove_dir_all("./test3/");
    }
}

#[cfg(test)]
mod get_file_contents_tests {
    use super::*;
    use std::fs::{remove_file};

    #[test]
    fn get_file_contents_test() {
        let contents = "Hello world!";
        let filename = "test.txt";

        {
            let mut file = File::create(filename).expect("Failed to create file");
            file.write(contents.as_bytes()).expect("Failed to write to file");    
        }
        

        assert_eq!(get_file_contents(File::open(filename).unwrap()).unwrap(), contents);
        remove_file(filename);
    }

}