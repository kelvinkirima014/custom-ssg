use std::convert::{ Infallible, From };
use std::io::{Error, Read, self};
use std::fs::{self, File};
use std::path::PathBuf;
use std::sync::Arc;
use handlebars::Handlebars;
use hyper::service::{service_fn, make_service_fn};
use hyper::{ Body, Request, Response, Server, StatusCode };
use serde_json::json;
use pulldown_cmark::{html, Options, Parser};

//use tokio::runtime::Handle;

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


fn generate_html(posts: &[(PathBuf, String)]) -> Result<(), io::Error> {

    let mut handlebars = Handlebars::new();
        handlebars.register_template_file("blog_template", "templates/posts.hbs")
        .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;

        for (path, contents) in posts {

            let yaml_front_matter = contents.split("---").nth(1).unwrap_or("");
            let yaml_data: serde_yaml::Value = serde_yaml::from_str(yaml_front_matter).unwrap();

            let post_title = yaml_data["title"].as_str().unwrap_or("");
            let post_description = yaml_data["title"].as_str().unwrap_or("");
            let post_date = yaml_data["date"].as_str().unwrap_or("");

            let rendered_html = match handlebars.render("blog_template", &json!({
                "title": post_title,
                "description": post_description,
                "content": md_to_html(&contents),
                "date": post_date,
            })) {
                Ok(html) => html,
                Err(err) => return Err(io::Error::new(io::ErrorKind::Other, err)),
             };
            //  html.push_str(&rendered_html);
            let file_name = path.file_stem().and_then(|stem| stem.to_str()).unwrap_or("");
            let output_file = format!("{}.html", file_name);
            fs::write(&output_file, rendered_html)?;
        }
    Ok(())
}


#[tokio::main]
async fn main () -> Result<(), Box<dyn std::error::Error>> {

    let post_path = PathBuf::from("./markdown");

    let posts = Posts::new(post_path);

    let get_posts = posts.fetch_posts()?;

    generate_html(&get_posts)?;


      let make_serve = make_service_fn(|_| {
        let content_dir = Arc::new(PathBuf::from("."));
        async move {
            Ok::<_, Infallible>(service_fn(move |req: Request<Body>| {
                let path = req.uri().path().to_owned();
                let content_dir = content_dir.clone();

                async move {
                    // Check if the requested path corresponds to a generated HTML file
                    if let Some(file_name) = path.strip_prefix("/") {
                        let file_path = content_dir.join(file_name);
                        if file_path.is_file() {
                            if let Ok(content) = fs::read(file_path){
                                let response = Response::new(Body::from(content));
                                return Ok::<_, Infallible>(response);
                            }
                        }
                    }

                    // Return a 404 Not Found response for all other requests
                    let response = Response::builder()
                        .status(StatusCode::NOT_FOUND)
                        .body(Body::empty())
                        .unwrap();
                    Ok::<_, Infallible>(response)
                }
            }))
        }
    });
    

    let addr = ([127, 0, 0, 1], 3000).into();


    let server = Server::bind(&addr).serve(make_serve);

    println!("Listening on http://{}", addr);
    
    server.await.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    Ok(())
  
}