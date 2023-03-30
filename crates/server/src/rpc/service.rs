use crate::{
    api::routes::quests::StartQuestRequest as StartQuestRequestAPI,
    domain::{events::add_event_controller, quests::start_quest_controller},
};
use dcl_rpc::stream_protocol::Generator;
use quests_db::core::definitions::QuestsDatabase;
use quests_definitions::quests::{
    AbortQuestRequest, AbortQuestResponse, Event, EventResponse, QuestsServiceServer,
    ServerStreamResponse, StartQuestRequest, StartQuestResponse, UserAddress, UserUpdate,
};
use quests_message_broker::quests_channel::QuestsChannel;
use std::sync::Arc;

use super::QuestsRpcServerContext;
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
            Ok(_) => StartQuestResponse { accepted: true },
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
        let (generator, generator_yielder) = Generator::create();
        match ctx.db.get_user_quest_instances(&req.user_address).await {
            Ok(instances) => {
                for instance in instances {
                    let yielder = generator_yielder.clone();
                    ctx.redis_quests_channel
                        .subscribe(
                            &instance.id,
                            // TODO: change this. remove the box
                            Box::new(move |user_update| {
                                // TODO: fix this
                                let yielder = yielder.clone();
                                Box::pin(async move {
                                    yielder.r#yield(user_update).await.unwrap();
                                    // TODO: handle error
                                })
                            }),
                        )
                        .await;
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
