use std::path::PathBuf;
use std::time::SystemTime;
//ds that represent the site we want to generate

pub struct SiteMetadata{
    title: String,
    description: String,
    date: SystemTime,
}

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
pub struct Posts {
    post: PathBuf,
}

impl Posts {
    fn new(post: PathBuf) -> Self {
        Posts {
            post,
     }
    }
}
pub struct Pages {
    content: String,
}

pub struct Tags {
    tags: String,
}



fn main () {

}