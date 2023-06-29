use std::convert::Infallible;
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use hyper::{ Body, Request, Response, StatusCode };
use hyper::service::service_fn;

async fn serve_html_files(req: Request<Body>) -> Result<(), Box<dyn Error>> {
    let req = Arc::new(req);
    let service = service_fn(move |_: Request<Body>| {
    let cloned_req = Arc::clone(&req);
         async move {
            let path = cloned_req.uri().path().to_owned();
            let content_dir = Arc::new(PathBuf::from(".."));
            if let Some(file_name) = path.strip_prefix("/") {
               let file_path = content_dir.join(file_name);
               if file_path.is_file() {
                match fs::read(file_path) {
                    Ok(content) => {
                        let response = Response::new(Body::from(content));
                        
                    }
                    Err(err) => {
                        eprintln!("Failed to read file: {}", err);
                        let response = Response::builder()
                         .status(StatusCode::INTERNAL_SERVER_ERROR)
                         .body(Body::empty())
                         .unwrap();
                    }   
                }
                
               } 
            }
            
        }

    }
       
    );

    Ok(())
}