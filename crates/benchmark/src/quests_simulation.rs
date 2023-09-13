use std::{fmt::Display, time::Duration};

use async_trait::async_trait;
use dcl_rpc::{client::RpcClient, stream_protocol::Generator};
use log::{debug, error, info};
use quests_protocol::definitions::*;
use rand::{seq::SliceRandom, thread_rng};
use serde::Deserialize;
use tokio::time::timeout;

use crate::{
    args::Args,
    client::{create_test_identity, get_signed_headers, TestWebSocketTransport},
    quests::{create_random_string, random_quest},
    simulation::{Client, Context},
};

#[derive(Deserialize)]
struct CreateQuestResponse {
    id: String,
}

pub struct TestContext {
    pub quest_ids: Vec<String>,
    pub timeout: Duration,
}

impl TestContext {
    pub async fn create_random_quest(api_host: &str) -> Result<String, String> {
        let quest = random_quest();
        let headers = get_signed_headers(
            create_test_identity(),
            "post",
            "/quests",
            serde_json::to_string(&quest).unwrap().as_str(),
        );

        let client = reqwest::Client::new();

        let res = client
            .post(format!("{api_host}/quests"))
            .header(headers[0].0.clone(), headers[0].1.clone())
            .header(headers[1].0.clone(), headers[1].1.clone())
            .header(headers[2].0.clone(), headers[2].1.clone())
            .header(headers[3].0.clone(), headers[3].1.clone())
            .header(headers[4].0.clone(), headers[4].1.clone())
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
    pub client: RpcClient<TestWebSocketTransport>,
    pub quests_service: QuestsServiceClient<TestWebSocketTransport>,
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
        quests_service: &QuestsServiceClient<TestWebSocketTransport>,
        context: &TestContext,
    ) -> Self {
        let current_state_discriminant = std::mem::discriminant(&self);
        let state = match self {
            ClientState::Start => {
                let result = quests_service.subscribe().await;
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
                        quest_id: quest_id.clone(),
                    })
                    .await;
                debug!(
                    "User {} > StartQuestRequest: id {} > Response: {:?}",
                    quest_id,
                    &user_address[..4],
                    response
                );
                match response {
                    Ok(StartQuestResponse {
                        response: Some(start_quest_response::Response::Accepted(_)),
                    }) => ClientState::StartQuestRequested { updates, quest_id },
                    _ => ClientState::Subscribed { updates },
                }
            }
            ClientState::StartQuestRequested {
                mut updates,
                quest_id,
            } => {
                let act = timeout(context.timeout, updates.next()).await;
                let Ok(quest_updates) = act else {
                    error!(
                        "User {} > Timeout while fetching started quest state",
                        &user_address[..4]
                    );
                    return ClientState::StartQuestRequested { updates, quest_id };
                };

                match quest_updates {
                    Some(UserUpdate {
                        message:
                            Some(user_update::Message::NewQuestStarted(QuestInstance {
                                id,
                                state: Some(state),
                                ..
                            })),
                        ..
                    }) => ClientState::MakeQuestProgress {
                        updates,
                        quest_instance_id: id,
                        quest_state: state,
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
                    .send_event(EventRequest {
                        action: action.cloned(),
                    })
                    .await;
                match event {
                    Ok(EventResponse {
                        response: Some(event_response::Response::AcceptedEventId(_)),
                        ..
                    }) => ClientState::FetchQuestUpdate {
                        updates,
                        quest_instance_id,
                        quest_state,
                    },
                    _ => {
                        error!(
                            "> User {} > Make Quest Progress > event not accepted, retrying",
                            &user_address[..4]
                        );
                        ClientState::MakeQuestProgress {
                            updates,
                            quest_instance_id,
                            quest_state,
                        }
                    }
                }
            }
            ClientState::FetchQuestUpdate {
                mut updates,
                quest_instance_id,
                quest_state,
            } => {
                debug!("User {} > Fetch next event > Next.", &user_address[..4]);
                let act = timeout(context.timeout, updates.next()).await;
                let Ok(quest_update) = act else {
                    error!(
                        "User {} > Timeout while fetching next event!",
                        &user_address[..4]
                    );
                    return ClientState::FetchQuestUpdate {
                        updates,
                        quest_instance_id,
                        quest_state,
                    };
                };
                debug!("User {} > Fetch next event > Done.", &user_address[..4]);

                debug!(
                    "User {} > quest_update received > {quest_update:?}",
                    &user_address[..4]
                );

                match quest_update {
                    Some(update) => match update.message {
                        Some(user_update::Message::QuestStateUpdate(QuestStateUpdate {
                            quest_state: Some(state),
                            instance_id,
                            ..
                        })) if instance_id == quest_instance_id => {
                            let new_quest_state = state;
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
                        Some(user_update::Message::EventIgnored(_)) => {
                            error!("User {} > Event ignored", &user_address[..4]);
                            ClientState::MakeQuestProgress {
                                updates,
                                quest_instance_id,
                                quest_state,
                            }
                        }
                        Some(user_update::Message::QuestStateUpdate(QuestStateUpdate {
                            instance_id,
                            ..
                        })) => {
                            debug!(
                                "User {} > QuestStateUpdate received for wrong quest instance {}, expected instance was {}",
                                &user_address[..4],
                                instance_id,
                                quest_instance_id
                            );
                            ClientState::FetchQuestUpdate {
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
    async fn init(args: &Args) -> Self {
        let mut quest_ids = vec![];

        for _ in 0..args.quests {
            match Self::create_random_quest(&args.api_host).await {
                Ok(quest_id) => quest_ids.push(quest_id),
                Err(reason) => debug!("Quest Creation > Couldn't POST quest: {reason}"),
            }
        }
        Self {
            quest_ids,
            timeout: Duration::from_secs(args.timeout as u64),
        }
    }
}

#[async_trait]
impl Client<TestContext> for TestClient {
    async fn from_rpc_client(mut client: RpcClient<TestWebSocketTransport>) -> Self {
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
