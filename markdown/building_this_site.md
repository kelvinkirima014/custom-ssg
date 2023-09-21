## How I Built This Site

I have spent the last few weeks working on getting this blog out; So I thought I'll make good use of the effort by documenting how I built it. I once had a blog on a different domain that I didn't really like because I had just copy pasted my content into a template, so it didn't really feel like my own. For this one, I wanted something that I'd enjoy building and customising in any way I saw fit. I decided to employ the static site generator model and built a generator that takes markdown files as input and produces .html files that can be rendered on a browser. I then host the html files, and their corresponding CSS, and what you see infront of you is a result of that endeavour.

## Reading the Markdown Files

The first thing I wanted is a way to read the markdown files that I'd later convert to html. I came up with a simple solution; a Posts struct with just a constructor function and a single method to fetch the posts:

```rust
pub struct Posts {
    post_path: PathBuf,
}

impl Posts {
    pub fn new(post_path: PathBuf) -> Self {
        Posts {
            post_path,
        }
    }

    pub fn fetch_posts(&self) -> Result<Vec<(PathBuf, String)>, Error>{

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
```
With this implementation in place, consuming markdown files from a markdown folder is as easy as:

```rust

let posts_path = PathBuf::from("./markdown");

let posts = Posts::new(posts_path);

let fetch_posts = posts.fetch_posts();
```

## The HTML Generator, Initial Approach

Now that I have access to the markdown files, the next thing I need to do is parse this markdown to html.Initially, I wrote a simple helper function that takes a string slice: ```markdown```, and returns a html string using a parser from the pulldown_cmark crate. The code looked like this:

```rust
fn md_to_html(markdown: &str) -> String {
    let parser = pulldown_cmark::Parser::new_ext(&markdown,pulldown_cmark::Options::Empty());
    let mut html_output = String::new();
    pulldown_cmark::html::push_str(&html_output, parser);

    html_output
}
```

I consumed the ```md_to_html``` inside the function that actually does the generation: 

```rust
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
```

In the code above, I use the Handlebars templating engine to register a template file in order to enhance the appearance and functionality of the blog. Typically, markdown content takes this structure:

```
---
title: blog_title
description: blog_description
---
```
The dotted lines are known as frontmatter, and we've to split them in order to extract the content:

```rust
let split_front_matter: Vec<&str> = contents.splitn(3, "---").collect();
let yaml_front_matter = split_front_matter.get(1).unwrap_or(&"");
let markdown = split_front_matter.get(2).unwrap_or(&"");

let yaml_data: serde_yaml::Value = serde_yaml::from_str(yaml_front_matter).unwrap();
```
We combine the markdown content and the handlebars template like so:
```rust
let post_title = yaml_data["title"].as_str();
let post_description = yaml_data["description"].as_str();
let content = md_to_html(markdown);        let post_date = yaml_data["date"].as_str();
    let rendered_html = match handlebars.render("blog_template", &json!({          "post_title": post_title,
    "post_description": post_description,
    "post_content": content,
    "post_date": post_date,
})) {
        Ok(html) => html,
        Err(err) => return Err(io::Error::new(io::ErrorKind::Other, err)),    
    }; 

```

This approach looked reasonable at the start, but I ran into a problem when I tried publishing an article with code blocks; the code blocks lacked syntax highlighting. 