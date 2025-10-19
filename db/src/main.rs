use wwc_db::{create_pool, get_games};

#[tokio::main]
async fn main() {
    let pool = create_pool().await.expect("Failed to create pool");
    let posts = get_games(&pool).await;
    println!("{:?}", posts);
}
