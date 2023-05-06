use std::convert::{ Infallible, From };
use std::io::{Error, Read, self};
use std::fs::{self, File};
use std::path::PathBuf;
use std::io::Write;
use handlebars::Handlebars;
use hyper::service::{service_fn, make_service_fn};
use hyper::{ Body, Request, Response, Server };
use serde_json::json;
use pulldown_cmark::{html, Options, Parser};

pub struct Posts {
    post_path: PathBuf,
}

impl Posts {
    fn new(post_path: PathBuf) -> Self {
        Posts {
            post_path,
        }
    }

    fn fetch_posts(&self) -> Result<Vec<(PathBuf, String)>, Error>{

        let mut posts = vec![];

        let entries = fs::read_dir(&self.post_path)?;
            //iterate over the contents of the directory
            for entry in entries {
                let entry = entry?;
                let path = entry.path();

                //check if the entry is a file and has .md extension
                if entry.file_type()?.is_file() && path.extension()
                    .and_then(|e| e.to_str()) == Some("md") {
                        let mut file = File::open(&path)?;
                        let mut contents = String::new();
                        file.read_to_string(&mut contents)?;
                        posts.push((path, contents));
                    }
            }

            Ok(posts)

    }
}


fn md_to_html(markdown: &str) -> String {
    // Set up the parser with default options
    let parser = Parser::new_ext(markdown, Options::empty());

    // Convert the markdown to HTML
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    html_output
}

#[tokio::main]
async fn main () -> Result<(), Error> {
    let mut handlebars = Handlebars::new();
    let custom_template = "<html><body>{{{content}}}</body></html>";
    handlebars.register_template_string("test_template", custom_template).unwrap();

    let post_path = PathBuf::from("./markdown");

    let posts: Posts = Posts::new(post_path);
    match posts.fetch_posts() {
        Ok(posts_path) => {
            for (path, contents) in posts_path.iter() {
                let html = handlebars.render("test_template", &json!({"content": md_to_html(contents)}));
                
                let mut file = fs::File::create("index.html").unwrap();
                file.write_all(html.expect("ERr").as_bytes()).unwrap();

               println!("Contents of {}: \n{}", path.to_string_lossy(), contents);
            }
        },
        Err(err) => {
            println!("Error fetching posts: {}", err);
        }
    }

    let addr = ([127, 0, 0, 1], 3000).into();

    let make_serve = make_service_fn(|_| async {
        let content =  fs::read("index.html").unwrap();
        Ok::<_, Infallible>(service_fn(move |_: Request<Body>| {
            let response = Response::new(Body::from(content.clone()));
            async move {
                Ok::<_, Infallible>(response)
            }
        }))
    });

    let server = Server::bind(&addr).serve(make_serve);

    println!("Listening on http://{}", addr);
    
    server.await.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    Ok(())
  
}