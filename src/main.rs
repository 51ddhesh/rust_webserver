// Import necessary modules from the standard library
use std::net::{TcpListener, TcpStream}; // For TCP networking
use std::io::{BufReader, prelude::*};   // For buffered reading and I/O traits
use std::fs;                            // For file system operations
use std::thread;                        // For thread sleeping (simulated delay)
use std::time::Duration;                // For specifying sleep duration
use rust_webserver::ThreadPool;         // Custom thread pool implementation

/// Entry point of the web server application.
/// 
/// Binds a TCP listener to localhost on port 6969 and handles incoming connections using a thread pool.
/// Each incoming TCP stream is processed in a worker thread by calling `handle_connection`.
fn main() {
    // Bind the TCP listener to the specified address and port.
    // Panics if binding fails (e.g., port already in use).
    let listener = TcpListener::bind("127.0.0.1:6969").unwrap();
    // Create a thread pool with 4 worker threads.
    let pool = ThreadPool::new(4);
    // Accept incoming connections in a loop.
    for stream in listener.incoming() {
        // Unwrap the Result to get the actual TcpStream.
        // If an error occurs, the server will panic.
        let stream = stream.unwrap();
        // Submit the connection to the thread pool for processing.
        pool.execute(|| {
            handle_connection(stream);
        });
    }
}

/// Handles an individual TCP connection by reading the HTTP request and sending an appropriate response.
/// 
/// # Arguments
/// * `stream` - The TCP stream representing the client connection.
/// 
/// Reads the first line of the HTTP request, determines the requested path, and serves the appropriate HTML file.
/// - For `GET /`, serves `pages/hello.html` with 200 OK.
/// - For `GET /sleep`, waits 5 seconds then serves `pages/hello.html` with 200 OK.
/// - For any other path, serves `pages/404.html` with 404 NOT FOUND.
/// The response includes the HTTP status line, Content-Length header, and the file contents as the body.
fn handle_connection(mut stream: TcpStream) {
    // Wrap the stream in a buffered reader for efficient line-by-line reading.
    let buf_reader = BufReader::new(&stream);

    // Read the first line of the HTTP request (the request line).
    // Example: "GET / HTTP/1.1"
    let request_line = buf_reader.lines().next().unwrap().unwrap();

    // Match the request line to determine the response.
    let (status_line, filename) = match &request_line[..] {
        // Serve hello.html for root path
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "pages/hello.html"),
        // Simulate a slow response for /sleep
        "GET /sleep HTTP/1.1" => {
            thread::sleep(Duration::from_secs(5));
            ("HTTP/1.1 200 OK", "pages/hello.html")
        }
        // Serve 404.html for all other paths
        _ => ("HTTP/1.1 404 NOT FOUND", "pages/404.html"),
    };

    // Read the contents of the HTML file to be served.
    // Panics if the file does not exist or cannot be read.
    let contents = fs::read_to_string(filename).unwrap();
    let length = contents.len();

    // Format the HTTP response with status line, Content-Length header, and body.
    // The response must be separated by CRLF (\r\n) as per HTTP protocol.
    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

    // Write the response to the TCP stream, sending it to the client.
    // Panics if the write fails.
    stream.write_all(response.as_bytes()).unwrap();
}
