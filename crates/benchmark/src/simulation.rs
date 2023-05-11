use std::fmt::Display;

use async_trait::async_trait;
use dcl_rpc::{client::RpcClient, transports::web_socket::WebSocketTransport};
use log::{error, info};
use rand::seq::SliceRandom;
use serde::Deserialize;

use crate::{
    quests::{create_random_string, random_quest},
    user_update::Message,
    *,
};

#[derive(Deserialize)]
struct CreateQuestResponse {
    id: String,
}

pub struct TestContext {
    pub quest_ids: Vec<String>,
}

impl TestContext {
    pub async fn create_random_quest() -> Result<String, String> {
        let quest = random_quest();

        let client = reqwest::Client::new();
        let res = client
            .post(format!("{}/quests", SERVER_HTTP))
            .json(&quest)
            .send()
            .await
            .map_err(|e| format!("Request failed: {e:?}"))?;

        let CreateQuestResponse { id } = res
            .json::<CreateQuestResponse>()
            .await
            .map_err(|e| format!("Response deserialize error: {e:?}"))?;

        Ok(id)
    }

    pub fn pick_quest_id(&self) -> Option<String> {
        self.quest_ids
            .choose(&mut thread_rng())
            .map(|id| id.to_string())
    }
}

pub struct TestClient {
    pub client: RpcClient<WebSocketTransport>,
    pub quests_service: QuestsServiceClient<WebSocketTransport>,
    pub address: String,
    pub state: ClientState,
}

pub enum ClientState {
    Start,
    Subscribed {
        updates: Generator<UserUpdate>,
    },
    QuestFinished {
        updates: Generator<UserUpdate>,
    },
    StartQuestRequested {
        updates: Generator<UserUpdate>,
        quest_id: String,
    },
    MakeQuestProgress {
        updates: Generator<UserUpdate>,
        quest_instance_id: String,
        quest_state: QuestState,
    },
    FetchQuestUpdate {
        updates: Generator<UserUpdate>,
        quest_instance_id: String,
        quest_state: QuestState,
    },
}

impl Display for ClientState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!(
            "State > {}",
            match self {
                ClientState::Start => "Start".to_string(),
                ClientState::Subscribed { .. } => "Subscribed".to_string(),
                ClientState::QuestFinished { .. } => "Quest finished".to_string(),
                ClientState::StartQuestRequested { quest_id, .. } => {
                    format!("Started Quest {quest_id}")
                }
                ClientState::MakeQuestProgress {
                    quest_instance_id,
                    quest_state,
                    ..
                } => format!(
                    "Make Quest Progress > Instance: {} - Steps left {}",
                    quest_instance_id, quest_state.steps_left
                ),
                ClientState::FetchQuestUpdate {
                    quest_instance_id,
                    quest_state,
                    ..
                } => format!(
                    "Fetch Quest Updates > Instance: {} - Steps left {}",
                    quest_instance_id, quest_state.steps_left
                ),
            },
        ))
    }
}

impl ClientState {
    pub async fn next(
        self,
        user_address: &str,
        quests_service: &QuestsServiceClient<WebSocketTransport>,
        context: &TestContext,
    ) -> Self {
        let current_state_discriminant = std::mem::discriminant(&self);
        let state = match self {
            ClientState::Start => {
                let result = quests_service
                    .subscribe(UserAddress {
                        user_address: user_address.to_string(),
                    })
                    .await;
                match result {
                    Ok(response) => ClientState::Subscribed { updates: response },
                    Err(_) => ClientState::Start,
                }
            }
            ClientState::Subscribed { updates } | ClientState::QuestFinished { updates } => {
                let Some(quest_id) = context.pick_quest_id() else {
                    return ClientState::Subscribed { updates };
                };

                let response = quests_service
                    .start_quest(StartQuestRequest {
                        user_address: user_address.to_string(),
                        quest_id: quest_id.clone(),
                    })
                    .await;
                debug!(
                    "User {} > StartQuestRequest > Response: {:?}",
                    &user_address[..4],
                    response
                );
                match response {
                    Ok(response)
                        if matches!(
                            response.response,
                            Some(start_quest_response::Response::Accepted(_))
                        ) =>
                    {
                        ClientState::StartQuestRequested { updates, quest_id }
                    }
                    _ => ClientState::Subscribed { updates },
                }
            }
            ClientState::StartQuestRequested {
                mut updates,
                quest_id,
            } => {
                debug!("User {} > Start Quest > Next.", &user_address[..4]);
                let quest_updates = updates.next().await;
                debug!("User {} > Start Quest > Done.", &user_address[..4]);

                match quest_updates {
                    Some(UserUpdate {
                        message: Some(Message::QuestState(state)),
                        ..
                    }) if state.quest_state.is_some() => ClientState::MakeQuestProgress {
                        updates,
                        quest_instance_id: state.quest_instance_id,
                        quest_state: state.quest_state.unwrap(),
                    },
                    _ => {
                        error!(
                            "User {} > Start Quest > Received update is not the quest state",
                            &user_address[..4]
                        );
                        ClientState::StartQuestRequested { updates, quest_id }
                    }
                }
            }
            ClientState::MakeQuestProgress {
                updates,
                quest_instance_id,
                quest_state,
            } => {
                // find action to do
                let action = quest_state
                    .current_steps
                    .values()
                    .find(|step| !step.to_dos.is_empty())
                    .and_then(|step| step.to_dos.first())
                    .and_then(|to_do| to_do.action_items.first());

                let event = quests_service
                    .send_event(Event {
                        address: user_address.to_string(),
                        action: action.cloned(),
                    })
                    .await;
                match event {
                    Ok(_) => ClientState::FetchQuestUpdate {
                        updates,
                        quest_instance_id,
                        quest_state,
                    },
                    _ => ClientState::MakeQuestProgress {
                        updates,
                        quest_instance_id,
                        quest_state,
                    },
                }
            }
            ClientState::FetchQuestUpdate {
                mut updates,
                quest_instance_id,
                quest_state,
            } => {
                debug!("User {} > Fetch next event > Next.", &user_address[..4]);
                let quest_update = updates.next().await;
                debug!("User {} > Fetch next event > Done.", &user_address[..4]);

                debug!("User {user_address} > quest_update received > {quest_update:?}");

                match quest_update {
                    Some(update) => match update.message {
                        Some(Message::QuestState(state))
                            if state.quest_instance_id == quest_instance_id
                                && state.quest_state.is_some() =>
                        {
                            let new_quest_state = state.quest_state.unwrap();
                            if new_quest_state.steps_left == 0 {
                                ClientState::QuestFinished { updates }
                            } else {
                                ClientState::MakeQuestProgress {
                                    updates,
                                    quest_instance_id,
                                    quest_state: new_quest_state,
                                }
                            }
                        }
                        Some(Message::EventIgnored(_)) => {
                            error!("User {user_address} > Event ignored");
                            ClientState::MakeQuestProgress {
                                updates,
                                quest_instance_id,
                                quest_state,
                            }
                        }
                        _ => ClientState::FetchQuestUpdate {
                            updates,
                            quest_instance_id,
                            quest_state,
                        },
                    },
                    None => ClientState::Subscribed { updates },
                }
            }
        };

        if std::mem::discriminant(&state) == current_state_discriminant {
            info!("User {} > State didn't change", &user_address[..4]);
        } else {
            info!("User {} > {state}", &user_address[..4]);
        }
        state
    }
}

#[async_trait]
impl Context for TestContext {
    async fn init() -> Self {
        let mut quest_ids = vec![];

        for _ in 0..50 {
            match Self::create_random_quest().await {
                Ok(quest_id) => quest_ids.push(quest_id),
                Err(reason) => debug!("Quest Creation > Couldn't POST quest: {reason}"),
            }
        }

        Self { quest_ids }
    }
}

#[async_trait]
impl Client<TestContext> for TestClient {
    async fn from_rpc_client(mut client: RpcClient<WebSocketTransport>) -> Self {
        let port = client
            .create_port("test-port")
            .await
            .expect("Can create port");

        let quests_service = port
            .load_module::<QuestsServiceClient<_>>("QuestsService")
            .await
            .expect("Can create quests service");

        Self {
            client,
            quests_service,
            address: Self::create_random_address(),
            state: ClientState::Start,
        }
    }

    async fn act(mut self, context: &TestContext) -> Self {
        self.state = self
            .state
            .next(&self.address, &self.quests_service, context)
            .await;

        self
    }
}

impl TestClient {
    fn create_random_address() -> String {
        create_random_string(40)
    }
}
