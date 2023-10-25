use std::env;

use quests_db::{
    core::definitions::CreateQuest,
    core::ops::{Connect, Migrate},
    core::tests::quest_database_works,
    DatabaseOptions,
};

#[tokio::test]
async fn quests_database_works() {
    let db_url = env::var("DATABASE_URL")
        .unwrap_or("postgres://postgres:postgres@localhost:5432/quests_db".to_string());

    let database_opts = DatabaseOptions::new(&db_url);

    let database = database_opts.connect().await.unwrap();

    let _ = database.migrate().await;

    quest_database_works(
        &database,
        CreateQuest {
            name: "NEW_QUEST",
            description: "Talk to a NPC",
            definition: vec![0, 1, 4],
            image_url: "",
            reward: None,
        },
    )
    .await;
}
