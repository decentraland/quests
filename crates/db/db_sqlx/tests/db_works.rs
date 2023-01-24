use std::env;

use quests_db_core::{
    ops::{Connect, Migrate},
    tests::quest_database_works,
    CreateQuest,
};
use quests_db_sqlx::DatabaseOptions;

#[tokio::test]
async fn quests_database_works() {
    let db_url = env::var("DATABASE_URL").unwrap();

    let database_opts = DatabaseOptions::new(&db_url);

    let database = database_opts.connect().await.unwrap();

    database.migrate().await.unwrap();

    quest_database_works(
        &database,
        CreateQuest {
            name: "NEW_QUEST",
            description: "Talk to a NPC",
            definition: vec![0, 1, 4],
        },
    )
    .await;
}
