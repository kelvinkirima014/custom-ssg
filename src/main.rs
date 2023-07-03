//use std::io;
use std::{path::PathBuf, sync::Arc};

use hyper::service::{make_service_fn, service_fn};
use hyper::{Server, server::conn::AddrStream};

pub mod posts;
pub mod templater;
pub mod server;


#[tokio::main]
async fn main () -> Result<(), Box<dyn std::error::Error>> {

    let post_path = PathBuf::from("./markdown");

    let posts = posts::Posts::new(post_path);

    let get_posts = posts.fetch_posts()?;

    templater::generate_html(&get_posts)?;


    let content_dir = Arc::new(PathBuf::from("./blog"));//.canonicalize()?);
    println!("content dir: {:?}", content_dir);

    let make_hyper_service = make_service_fn(|_: &AddrStream| {
        let content_dir = content_dir.clone();
        async {
            Ok::<_, hyper::Error>(service_fn(move |req| {
                let content_dir = content_dir.clone();
                let file_path = content_dir.join(&req.uri().path().trim_start_matches("/"));//.trim_start_matches('/'));
                println!("filepath from main is: {:?}", file_path);
                let file_name = file_path.file_name().unwrap_or_default().to_owned();
                println!("file_name: {:?}", file_name);
                server::serve_html(req, Arc::new(file_path))
            }))
        }
    });

    let addr = ([127, 0, 0, 1], 3000).into();
    let server = Server::bind(&addr).serve(make_hyper_service);

    println!("Server listening on port http://{}", addr);

    if let Err(e) = server.await {
     eprintln!("server error: {}", e);
    }

    //server.await.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
  
    Ok(())
  
}