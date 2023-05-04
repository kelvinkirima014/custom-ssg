use std::collections::BTreeMap;
use std::io::{Error, Read};
use std::fs::{self, File};
use std::path::PathBuf;
use std::time::SystemTime;
use std::io::Write;
use handlebars::Handlebars;
use hyper::service::service_fn;
use hyper::{ Body, Request, Response, Server, service};
use serde_json::json;
//ds that represent the site we want to generate

#[allow(dead_code)]
pub struct SiteMetadata{
    title: String,
    description: String,
    date: SystemTime,
}


#[allow(dead_code)]
pub struct Posts {
    post_path: PathBuf,
}

#[allow(dead_code)]
impl Posts {
    fn new(post_path: PathBuf) -> Self {
        Posts {
            post_path,
        }
    }

    fn fetch_posts(&self) -> Result<Vec<(PathBuf, String)>, Error>{

        let mut posts = vec![];

        let files = fs::read_dir(&self.post_path)?;

        let mut handlebars = Handlebars::new();
            //iterate over the contents of the directory
            for file in files {
                let file = file?;
                let path = file.path();

                //check if the entry is a file and has .md extension
                if file.file_type()?.is_file() && path.extension()
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

use pulldown_cmark::{html, Options, Parser};

fn md_to_html(markdown: &str) -> String {
    // Set up the parser with default options
    let parser = Parser::new_ext(markdown, Options::empty());

    // Convert the markdown to HTML
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    html_output
}


fn main () {
    let post_path = PathBuf::from("./markdown");
    let posts = Posts::new(post_path);
    match posts.fetch_posts() {
        Ok(posts_path) => {
            for (path, contents) in posts_path.iter() {
                println!("{}", path.display());
                println!("contents: {}", contents);
            }
        },
        Err(err) => {
            println!("Error fetching posts: {}", err);
        }
    }

    let post_contents = posts;

    let input_src = "<html><body>{{{content}}}</body></html>";

    let mut handlebars = Handlebars::new();
    handlebars.register_template_string("a_template", input_src).unwrap();

    let html = handlebars.render("a_template",&json!({"content": md_to_html(post_contents)})).unwrap();
    
    let mut file = fs::File::create("index.html").unwrap();
    file.write_all(html.as_bytes()).unwrap();

    let addr = ([127, 0, 0, 1], 3000).into();

    let server = Server::bind(&addr)
        .serve(|| service_fn(|_req| {
            let content = fs::read("index.html").unwrap();
            Response::new(Body::from(content))
        }))
        .map_err(|e| eprintln!("Server error: {}", e));

    println!("Listening on http://{}", addr);
    hyper::rt::run(server);
  
}