use std::io::Error;
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;
//ds that represent the site we want to generate

#[allow(dead_code)]
pub struct SiteMetadata{
    title: String,
    description: String,
    date: SystemTime,
}

#[allow(dead_code)]
impl SiteMetadata {
    fn new(
        title: String,
        description: String,
        date: SystemTime,
    ) -> Self {
        SiteMetadata {
            title,
            description,
            date,
        }
    }
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

    fn fetch_posts(&self) -> Result<Vec<PathBuf>, Error>{

        let mut posts = vec![];

        let files = fs::read_dir(&self.post_path)?;

            //iterate over the contents of the directory
            for file in files {
                let file = file?;
                let path = file.path();

                //check if the entry is a file and has .md extension
                if file.file_type()?.is_file() && path.extension()
                    .and_then(|e| e.to_str()) == Some("md") {
                        posts.push(path);
                    }
            }

            Ok(posts)

        // if let Some(path) = post_path {
        //     return Ok(PathBuf::new());
        // } else {
        //     return Err(err) ;
        // }

    }
}
#[allow(dead_code)]
pub struct Pages {
    content: String,
}
#[allow(dead_code)]

pub struct Tags {
    tags: String,
}



fn main () {
    let post_path = PathBuf::from("./markdown");
    let posts = Posts::new(post_path);
    match posts.fetch_posts() {
        Ok(posts_path) => {
            for path in posts_path.iter() {
                println!("{}", path.display());
            }
        },
        Err(err) => {
            println!("Error fetching posts: {}", err);
        }
    }

}