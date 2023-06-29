use std::path::PathBuf;

pub mod posts;
pub mod templater;
pub mod server;


#[tokio::main]
async fn main () -> Result<(), Box<dyn std::error::Error>> {

    let post_path = PathBuf::from("./markdown");

    let posts = posts::Posts::new(post_path);

    let get_posts = posts.fetch_posts()?;

    templater::generate_html(&get_posts)?;

  
    Ok(())
  
}