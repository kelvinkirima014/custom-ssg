use std::io::{Error, Read};
use std::fs::{self, File};
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

    // fn fetch_metadata(&self, filepath: PathBuf) -> Result<Self, Error>{
    //     let content = fs::read_to_string(&filepath);
    //     let metadata: Self = serde_yaml::from_str(&content)?;


    //     Ok(())
    // }
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
#[allow(dead_code)]
pub struct Pages {
    page_list: String,
}

impl Pages {
     fn new(page_list: String) -> Self {
        Self { 
            page_list
         }
     }
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
            for (path, contents) in posts_path.iter() {
                println!("{}", path.display());
                println!("contents: {}", contents);
            }
        },
        Err(err) => {
            println!("Error fetching posts: {}", err);
        }
    }

}