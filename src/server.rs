use std::{sync::Arc, io};
use std::path::PathBuf;
use std::fs;
use hyper::{Body, Response, Request, StatusCode};


pub(crate) async fn serve_html(
    req: Request<Body>,
    content_dir: Arc<PathBuf>,
) -> Result<Response<Body>, hyper::Error> {

    let request_path = req.uri().path().trim_start_matches('/');

    let dir_path = content_dir.join(request_path);
    println!("Directory path is; {:?}", dir_path);
    
    if dir_path.is_dir(){

        let entries = fs::read_dir(dir_path)
            .expect("Coud not read Directory")
            .map(|res| res.map(|e| e.file_name()))
            .collect::<Result<Vec<_>, io::Error>>()
            .expect("Could not collect paths");
        
        let body = entries.iter()
            .map(|entry| format!("<a href=\"/{0}\">{0}</a><br />", entry.to_string_lossy()))
            .collect::<Vec<_>>()
            .join("");

        let response = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/html")
        .body(Body::from(body))
        .unwrap();

        return Ok(response);

    } else if dir_path.is_file()  {
        println!("Found File");
        match fs::read(dir_path){
            Ok(contents) =>     {
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
