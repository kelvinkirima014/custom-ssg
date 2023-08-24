use std::{path::PathBuf, fs};
use handlebars::Handlebars;
use pulldown_cmark::{ Parser, Options, html };
use serde_yaml;
use serde_json::{self, json};
use std::io;
use regex::Regex;

use syntect::html::highlighted_html_for_string;
use syntect::parsing::SyntaxSet;
use syntect::highlighting::ThemeSet;

fn md_to_html(markdown: &str) -> String {

    let syntax_set = SyntaxSet::load_defaults_newlines();

    let theme_set = ThemeSet::load_defaults();

    let syntax_option = syntax_set.find_syntax_by_token("rust");
    let theme_option = theme_set.themes.get("base16-ocean.light");
    assert!(syntax_option.is_some() && theme_option.is_some(), "Failed to load syntax or theme");

    let highlight_code = |code: &str, lang: &str| -> String {
        let syntax = match lang {
            "rust" => syntax_set.find_syntax_by_token("rust"),
            "javascript" => syntax_set.find_syntax_by_token("javascript"),
            "typescript" => syntax_set.find_syntax_by_token("typescript"),
            _ => None,
        
        };

        if let Some(s) = syntax {
            highlighted_html_for_string(code, &syntax_set, s, &theme_set.themes["base16-ocean.dark"]).unwrap()
        } else {
            code.to_string()
        }

    };

    let tags = ["rust", "javascript", "typescript"];

    let mut result_md = markdown.to_string();

    for tag in &tags {
        let re = Regex::new(&format!(r#"<code class="{}">(.*?)</code>"#, tag)).unwrap();

        result_md = re.replace_all(&result_md, |caps: &regex::Captures| {
            let code = &caps[1];
            format!("<code>{}</code>", highlight_code(code, tag))
        }).to_string();
    }

    let parser = Parser::new_ext(&result_md, Options::empty());
    
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    html_output
}       

pub fn generate_html(posts: &[(PathBuf, String)]) -> Result<(), io::Error> {
    let mut handlebars = Handlebars::new();
        handlebars.register_template_file("blog_template", "templates/posts.hbs")
        .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;

    let output_dir = PathBuf::from("blog");
    match fs::create_dir_all(&output_dir) {
        Ok(dir) => dir,
        Err(err) => return Err(err),
    }
    
    for (path, contents) in posts {
        let split_front_matter: Vec<&str> = contents.splitn(3, "---").collect();
        let yaml_front_matter = split_front_matter.get(1).unwrap_or(&"");
        let markdown = split_front_matter.get(2).unwrap_or(&"");

        let yaml_data: serde_yaml::Value = serde_yaml::from_str(yaml_front_matter).unwrap();

        let post_title = yaml_data["title"].as_str();
        let post_description = yaml_data["description"].as_str();
        let content = md_to_html(markdown);
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