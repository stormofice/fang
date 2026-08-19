pub mod handlers;
pub mod models;
pub mod resolver;

fn split_backlinks(links: String) -> Vec<String> {
    links.split("🙂‍↕️").map(str::to_string).collect()
}

// Thought about accepting generic iterators, was too lazy
fn join_backlinks(links: Vec<String>) -> String {
    <[String]>::join(&links, "🙂‍↕️")
}
