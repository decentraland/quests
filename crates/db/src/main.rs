use std::env;

use sqlx::postgres::PgPoolOptions;

#[actix_rt::main]
async fn main() {
    let db_url = env::var("DATABASE_URL").expect("set DATABASE_URL env var");
    let db = PgPoolOptions::new()
        .max_connections(2)
        .connect(&db_url)
        .await
        .expect("Unable to form database pool");

    sqlx::migrate!("./migrations/").run(&db).await.unwrap();
}
