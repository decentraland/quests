use super::QuestsRpcServerContext;
use crate::{
    api::routes::quests::StartQuestRequest as StartQuestRequestAPI,
    domain::{
        events::add_event_controller,
        quests::{
            get_all_quest_states_by_user_address_controller, get_instance_state,
            start_quest_controller,
        },
    },
};
use dcl_rpc::stream_protocol::Generator;
use quests_db::core::definitions::QuestsDatabase;
use quests_protocol::quests::{
    user_update::Message, AbortQuestRequest, AbortQuestResponse, Event, EventResponse,
    QuestStateUpdate, QuestsServiceServer, ServerStreamResponse, StartQuestRequest,
    StartQuestResponse, UserAddress, UserUpdate,
};
use std::sync::Arc;

pub struct QuestsServiceImplementation {}

#[async_trait::async_trait]
impl QuestsServiceServer<QuestsRpcServerContext> for QuestsServiceImplementation {
    // TODO: Add tracing instrument
    async fn start_quest(
        &self,
        req: StartQuestRequest,
        ctx: Arc<QuestsRpcServerContext>,
    ) -> StartQuestResponse {
        let StartQuestRequest {
            user_address,
            quest_id,
        } = req;
        match start_quest_controller(
            ctx.db.clone(),
            // TODO: reuse the auto-generated type
            StartQuestRequestAPI {
                user_address: user_address.clone(),
                quest_id: quest_id.clone(),
            },
        )
        .await
        {
            Ok(new_quest_instance_id) => {
                let user_subscriptions_lock = ctx.subscription_by_user_address.read().await;
                let user_subscription = user_subscriptions_lock.get(&user_address);
                if let Some(user_subscription) = user_subscription {
                    ctx.quest_subscriptions
                        .write()
                        .await
                        .insert(new_quest_instance_id.clone(), user_subscription.clone());
                    match get_instance_state(ctx.db.clone(), &quest_id, &new_quest_instance_id)
                        .await
                    {
                        Ok((quest, quest_state)) => {
                            let user_update = UserUpdate {
                                message: Some(Message::QuestState(QuestStateUpdate {
                                    quest_instance_id: new_quest_instance_id.clone(),
                                    name: quest.name,
                                    description: quest.description,
                                    quest_state: Some(quest_state),
                                })),
                            };
                            if user_subscription.r#yield(user_update).await.is_err() {
                                log::error!("QuestServiceImplementation > StartQuest Error > Not able to send update to susbcription")
                            }
                        }
                        Err(err) => {
                            log::error!("QuestServiceImplementation > StartQuest Error > Calculating state >{err:?}");
                        }
                    }
                }

                StartQuestResponse { accepted: true }
            }
            Err(err) => {
                log::error!("QuestsServiceImplementation > StartQuest Error > {err:?}");
                StartQuestResponse { accepted: false }
            }
        }
    }

    // TODO: Add tracing instrument
    async fn abort_quest(
        &self,
        _req: AbortQuestRequest,
        _ctx: Arc<QuestsRpcServerContext>,
    ) -> AbortQuestResponse {
        // TODO: missing business logic
        AbortQuestResponse { accepted: true }
    }

    // TODO: Add tracing instrument
    async fn send_event(&self, req: Event, ctx: Arc<QuestsRpcServerContext>) -> EventResponse {
        match add_event_controller(ctx.redis_events_queue.clone(), req).await {
            Ok(event_id) => EventResponse {
                event_id: Some(event_id as u32),
                accepted: true,
            },
            Err(error) => {
                log::error!("QuestsServiceImplementation > SendEvent Error > {error:?}");
                EventResponse {
                    event_id: None,
                    accepted: false,
                }
            }
        }
    }

    // TODO: Add tracing instrument
    async fn subscribe(
        &self,
        req: UserAddress,
        ctx: Arc<QuestsRpcServerContext>,
    ) -> ServerStreamResponse<UserUpdate> {
        log::debug!("QuestsServiceImplementation > Subscribe > User {req:?} subscribed");
        let (generator, generator_yielder) = Generator::create();
        let states = get_all_quest_states_by_user_address_controller(
            ctx.db.clone(),
            req.user_address.to_string(),
        )
        .await;

        if let Ok(states) = states {
            for (id, (quest, state)) in states {
                if (generator_yielder
                    .r#yield(UserUpdate {
                        message: Some(Message::QuestState(QuestStateUpdate {
                            name: quest.name,
                            description: quest.description,
                            quest_instance_id: id,
                            quest_state: Some(state),
                        })),
                    })
                    .await)
                    .is_err()
                {
                    log::error!("Failed to push state to response stream")
                }
            }
        }
        match ctx.db.get_user_quest_instances(&req.user_address).await {
            Ok(instances) => {
                log::debug!(
                    "QuestsServiceImplementation > Subscribe > Instances retrieved {instances:?}"
                );

                // store the subscription by the user address because if the user starts a new one
                // we have to have a way to communicate the updates of the new one.
                // TODO: could be done in another way?
                ctx.subscription_by_user_address
                    .write()
                    .await
                    .insert(req.user_address.clone(), generator_yielder.clone());

                for instance in instances {
                    let yielder = generator_yielder.clone();

                    ctx.quest_subscriptions
                        .write()
                        .await
                        .insert(instance.id, yielder);
                }

                generator
            }
            Err(error) => {
                log::error!("QuestsServiceImplementation > Subscribe Error > {error:?}");
                // TODO: fix. returns error
                generator
            }
        }
    }
}
