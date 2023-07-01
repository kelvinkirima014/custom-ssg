use std::{path::PathBuf, fs};

use handlebars::Handlebars;
use pulldown_cmark::{ Parser, Options, html };
use serde_yaml;
use serde_json::{self, json};
use std::io;


fn md_to_html(markdown: &str) -> String {
    let parser = Parser::new_ext(markdown, Options::empty());
    
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    html_output
}       

pub fn generate_html(posts: &[(PathBuf, String)]) -> Result<(), io::Error> {
    let mut handlebars = Handlebars::new();
        handlebars.register_template_file("blog_template", "templates/posts.hbs")
        .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;

    let output_dir = PathBuf::from("bloghtml");
    match fs::create_dir_all(&output_dir) {
        Ok(dir) => dir,
        Err(err) => return Err(err),
    }
    
    for (path, contents) in posts {
        let yaml_front_matter = contents.split("---").nth(1).unwrap_or("");
        let yaml_data: serde_yaml::Value = serde_yaml::from_str(yaml_front_matter).unwrap();

        let post_title = yaml_data["title"].as_str();
        let post_description = yaml_data["description"].as_str();
        let content = md_to_html(contents);
        let post_date = yaml_data["date"].as_str();

        let rendered_html = match handlebars.render("blog_template", &json!({
            "post_title": post_title,
            "post_description": post_description,
            "post_content": content,
            "post_date": post_date,
        })) {
            Ok(html) => html,
            Err(err) => return Err(io::Error::new(io::ErrorKind::Other, err)),    
        }; 

        let file_name = path.file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("");
        let output_file = output_dir.join(format!("{}.html", file_name));
        fs::write(&output_file, rendered_html)?;

    }

    Ok(())
}