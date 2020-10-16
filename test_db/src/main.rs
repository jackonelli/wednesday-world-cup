use test_db::get_games;

fn main() {
    let posts = get_games();
    println!("{:?}", posts);
}
