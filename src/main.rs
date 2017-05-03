//! WEB_SERVER
//! Homework 4
//!
//! This program takes an association list as input, as well as  
//! a start and end point on the graph from user input.
//! By constructing a specialized graph struct, the program searches
//! for a path between the two identified points, then requests another
//! pair. The program terminates when the user inputs a blank line or 
//! 999.
//!
//! ASSUMPTIONS:
//! -input space after HTTP token will cause faulty response
//!
//! TO TEST:
//! Run this code and seperately connect to 127.0.0.1:8080
//!      SERVER INPUT  ..................... RESPONSE
//! GET {/path/to}/src/main.rs HTTP - 200 OK, Content-Length: 1383
//! GET {/path/to}/src HTTP - 200 OK, Content-Length: 19
//! GET /a/b/c HTTP - 404 File Not Found
//! a b c - 400 Bad Request

use std::io::Write;
use std::net::TcpListener;
use std::thread;
use std::fs::File;
use std::sync::{Arc, Mutex};

mod server_handler;
use server_handler::{handle_request, Response};

extern crate time;
// use time::{Tm};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    println!("Listening for connections on port 8080");

    let log_file = Arc::new(Mutex::new(File::create("server_log.txt").unwrap()));

    for stream in listener.incoming() {
        println!("Connected stream");
        let lf = log_file.clone();

        match stream {
            Ok(stream) => {
                println!("Creating thread");
                thread::spawn(move || {
                    let response = handle_request(stream);
                    log_response(&lf, &response);
                });
            }
            Err(e) => {
                println!("Unable to connect: {}", e);
            }
        }
    }
}

fn log_response(log_file: &Arc<Mutex<File>>, response: &Response) {
    // Trying to follow format specified in https://en.wikipedia.org/wiki/Common_Log_Format
    // 127.0.0.1 user-identifier frank [10/Oct/2000:13:55:36 -0700] "GET /apache_pb.gif HTTP/1.0" 200 2326

    let mut log_file_guard = log_file.lock().unwrap();
    let output = format!("[{}] \"{}\" {} {}", response.time.rfc3339(), response.request, response.status_code, response.response_size);
    println!("{}", output);
    log_file_guard.write(output.as_bytes()).expect("Failed to log response");
}