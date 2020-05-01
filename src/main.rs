#[macro_use]
extern crate lazy_static;
extern crate futures;
extern crate hyper;
extern crate hyper_staticfile;
extern crate rand;
extern crate regex;
extern crate tokio;

use futures::{future, Future, Stream};
use hyper::service::service_fn;
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use hyper_staticfile::FileChunkStream;
use rand::distributions::Alphanumeric;
use rand::Rng;
use regex::Regex;
use std::fs;
use std::io::{Error, ErrorKind};
use std::path::Path;
use tokio::fs::File;

// static constant to hold the name of microservice
static INDEX: &[u8] = b"Images Microservice";

// macro to define regex
lazy_static! {
    // define regex for download file url as a static variable for easy parsing
    static ref DOWNLOAD_FILE: Regex = Regex::new("^/download/(?P<filename>\\w{20})?$").unwrap();
}

// handle conversion of box error into single format (io error)
fn other<E>(err: E) -> Error
where
    E: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    Error::new(ErrorKind::Other, err)
}

// prepare response with various status codes
fn response_with_code(
    status_code: StatusCode,
) -> Box<dyn Future<Item = Response<Body>, Error = std::io::Error> + Send> {
    let resp = Response::builder()
        .status(status_code)
        .body(Body::empty())
        .unwrap();
    Box::new(future::ok(resp))
}

// microservice default handler
fn microservice_handler(
    req: Request<Body>,
    files: &Path,
) -> Box<dyn Future<Item = Response<Body>, Error = std::io::Error> + Send> {
    // match by request method and url
    match (req.method(), req.uri().path().to_owned().as_ref()) {
        // for base url get requests
        (&Method::GET, "/") => Box::new(future::ok(Response::new(INDEX.into()))),
        // for get requests to /download path
        (&Method::GET, path) if path.starts_with("/download") => {
            // parse url with regex and get value filename from it
            if let Some(cap) = DOWNLOAD_FILE.captures(path) {
                let filename = cap.name("filename").unwrap().as_str();
                // get default filepath of the application
                let mut filepath = files.to_path_buf();
                // adding the file name to base url path
                filepath.push(filename);
                // reading file from full file path
                let open_file = File::open(filepath);
                // prepare response body from file
                let body = open_file.map(|file| {
                    // creating a stream of the file in chunks
                    let chunks = FileChunkStream::new(file);
                    Response::new(Body::wrap_stream(chunks))
                });
                Box::new(body)
            } else {
                response_with_code(StatusCode::NOT_FOUND)
            }
        }
        // for post requests to /upload path
        (&Method::POST, "/upload") => {
            // generate random string for file name;
            let name: String = rand::thread_rng()
                .sample_iter(&Alphanumeric)
                .take(20)
                .collect();
            // getting default application file path
            let mut filepath = files.to_path_buf();
            // creating file name for the new file by adding generated string to default filepath
            filepath.push(&name);
            // creates a file in write mode
            let create_file = File::create(filepath);
            let write = create_file.and_then(|file| {
                // takes the request file chunks accumulates it and writes it to the file opened
                req.into_body().map_err(other).fold(file, |file, chunk| {
                    tokio::io::write_all(file, chunk).map(|(file, _)| file)
                })
            });
            // return the name of the file as response
            let body = write.map(|_| Response::new(name.into()));
            Box::new(body)
        }
        _ => response_with_code(StatusCode::NOT_FOUND),
    }
}

fn main() {
    // create a new path from string
    let files = Path::new("./files");
    // creates a directory if not exists
    fs::create_dir(files).ok();
    // starts server
    let addr = ([127, 0, 0, 1], 8080).into();
    let builder = Server::bind(&addr);
    let server = builder.serve(move || service_fn(move |req| microservice_handler(req, &files)));
    let server = server.map_err(drop);
    hyper::rt::run(server);
}
