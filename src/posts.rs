use std::io::{Error, Read, self};
use std::fs::{self, File};
use std::path::PathBuf;
use std::io::Write;

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