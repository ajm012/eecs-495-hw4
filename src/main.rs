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

use std::net::TcpListener;
use std::thread;
//use std::io::prelude::*;
//use std::fs::File;

mod server_handler;
use server_handler::handle_client;

fn main() {
    /*let mut file = File::open("/Users/andrewmcconnell/Desktop/Rust/eecs-495-hw4/src/main.rs").expect("Unable to open the file");
    let mut contents = String::new();
    file.read_to_string(&mut contents).expect("Unable to read the file");
    println!("{}", contents);*/

    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    println!("Listening for connections on port {}", 8080);

    for stream in listener.incoming() {
        //println!("Connected stream");
        match stream {
            Ok(stream) => {
                //println!("Creating thread");
                thread::spawn(|| {
                    handle_client(stream)
                });
            }
            Err(e) => {
                println!("Unable to connect: {}", e);
            }
        }
    }
}
