use std::io;
use std::{path::PathBuf, sync::Arc};
use hyper::service::{make_service_fn, service_fn};
use hyper::{Server, server::conn::AddrStream};

pub mod posts;
pub mod templater;
pub mod server;


#[tokio::main]
async fn main () -> Result<(), Box<io::Error>> {

    let post_path = PathBuf::from("./markdown");

    let posts = posts::Posts::new(post_path);

    let get_posts = posts.fetch_posts()?;

    templater::generate_html(&get_posts)?;


    let content_dir = Arc::new(PathBuf::from("./blog"));
    

    let make_hyper_service = make_service_fn(|_: &AddrStream| {

        let content_dir = content_dir.clone();

        async {
            Ok::<_, hyper::Error>(service_fn(move |req| {
                let content_dir = content_dir.clone();
              
                // let file_path = content_dir.join(&req.uri().path().trim_start_matches("/"));
             
                // server::serve_html(req, file_path.into())

                server::serve_html(req, content_dir)
            
            }))
        }
    });

    let addr = ([127, 0, 0, 1], 3000).into();
    let server = Server::bind(&addr).serve(make_hyper_service);

    println!("Server listening on port http://{}", addr);


    server.await.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
  
    Ok(())
  
}