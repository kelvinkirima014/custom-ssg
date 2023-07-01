use std::sync::Arc;
use std::path::PathBuf;
use std::fs;
use hyper::{Body, Response, Request, StatusCode};


pub(crate) async fn serve_html(
    req: Request<Body>,
    content_dir: Arc<PathBuf>,
) -> Result<Response<Body>, hyper::Error> {

    let path = req.uri().path();
    let file_name = path.trim_start_matches("/");
    //let file_path = content_dir.join(file_name);
    let file_path = content_dir.join(&req.uri().path()[1..]);
  
    println!("filepath: {:?}", file_path);

     if file_path.is_file() {
        println!("File exists");
    } else {
        println!("File does not exist");
    }

    println!("file_name: {:?}", file_name);
    println!("content_dir: {:?}", content_dir);
    
    if file_path.is_file(){
        match fs::read(file_path){
            Ok(contents) => {
                let response = Response::builder()
                    .status(StatusCode::OK)
                    .body(Body::from(contents))
                    .unwrap();
                return Ok(response);
            }
            Err(err) => {
                eprintln!("Failed to read file: {}", err);
                let error_response = Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(Body::empty())
                    .unwrap();
                return Ok(error_response);
            }
        }
    } else {
        println!("File not found");
        let response = Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::empty())
            .unwrap();
        Ok(response)
    }
}