use test_db::*;

fn main() {
    let posts = get_posts();
    println!("{:?}", posts);
}
