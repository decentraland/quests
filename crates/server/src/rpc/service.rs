use super::QuestsRpcServerContext;
use crate::domain::{
    events::add_event_controller,
    quests::{
        self, get_all_quest_states_by_user_address, get_instance_state, start_quest, QuestError,
    },
};
use dcl_rpc::{
    rpc_protocol::RemoteErrorResponse, service_module_definition::ProcedureContext,
    stream_protocol::Generator,
};
use log::error;
use quests_message_broker::{channel::ChannelSubscriber, QUEST_UPDATES_CHANNEL_NAME};
use quests_protocol::quests::{
    user_update::Message, AbortQuestRequest, AbortQuestResponse, Event, EventResponse,
    QuestStateUpdate, QuestsServiceServer, ServerStreamResponse, StartQuestRequest,
    StartQuestResponse, UserAddress, UserUpdate,
};

pub struct QuestsServiceImplementation {}

type QuestRpcResult<T> = Result<T, QuestError>;

#[async_trait::async_trait]
impl QuestsServiceServer<QuestsRpcServerContext, QuestError> for QuestsServiceImplementation {
    // TODO: Add tracing instrument
    async fn start_quest(
        &self,
        req: StartQuestRequest,
        ctx: ProcedureContext<QuestsRpcServerContext>,
    ) -> QuestRpcResult<StartQuestResponse> {
        let StartQuestRequest {
            user_address,
            quest_id,
        } = req;
        match start_quest(ctx.server_context.db.clone(), &user_address, &quest_id).await {
            Ok(new_quest_instance_id) => {
                let user_subscriptions_lock =
                    ctx.server_context.subscription_by_transport_id.read().await;
                let user_subscription = user_subscriptions_lock.get(&ctx.transport_id);
                if let Some(user_subscription) = user_subscription {
                    match get_instance_state(
                        ctx.server_context.db.clone(),
                        &quest_id,
                        &new_quest_instance_id,
                    )
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
                            // TODO: Returns an error instead of false?
                        }
                    }
                }

                Ok(StartQuestResponse { accepted: true })
            }
            Err(err) => {
                log::error!("QuestsServiceImplementation > StartQuest Error > {err:?}");
                // TODO: Returns an error instead of false?
                Ok(StartQuestResponse { accepted: false })
            }
        }
    }

    // TODO: Add tracing instrument
    async fn abort_quest(
        &self,
        req: AbortQuestRequest,
        ctx: ProcedureContext<QuestsRpcServerContext>,
    ) -> QuestRpcResult<AbortQuestResponse> {
        let accepted = quests::abandon_quest(
            ctx.server_context.db.clone(),
            &req.user_address,
            &req.quest_instance_id,
        )
        .await
        .is_ok();

        // TODO: Returns an error instead of false?
        Ok(AbortQuestResponse { accepted })
    }

    // TODO: Add tracing instrument
    async fn send_event(
        &self,
        req: Event,
        ctx: ProcedureContext<QuestsRpcServerContext>,
    ) -> QuestRpcResult<EventResponse> {
        match add_event_controller(ctx.server_context.redis_events_queue.clone(), req).await {
            Ok(event_id) => Ok(EventResponse {
                event_id: Some(event_id as u32),
                accepted: true,
            }),
            Err(error) => {
                log::error!("QuestsServiceImplementation > SendEvent Error > {error:?}");
                // TODO: Returns an error instead of false?
                Ok(EventResponse {
                    event_id: None,
                    accepted: false,
                })
            }
        }
    }

    // TODO: Add tracing instrument
    async fn subscribe(
        &self,
        req: UserAddress,
        ctx: ProcedureContext<QuestsRpcServerContext>,
    ) -> QuestRpcResult<ServerStreamResponse<UserUpdate>> {
        log::debug!("QuestsServiceImplementation > Subscribe > User {req:?} subscribed");
        match get_all_quest_states_by_user_address(
            ctx.server_context.db.clone(),
            req.user_address.to_string(),
        )
        .await
        {
            Ok(states) => {
                let (generator, generator_yielder) = Generator::create();
                let mut quest_instance_ids = vec![];
                for (id, (quest, state)) in states {
                    quest_instance_ids.push(id.clone());
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
                        log::error!(
                            "QuestsServiceImplementation > Failed to push state to response stream"
                        );
                        // TODO: Return an error
                    }
                }

                ctx.server_context
                    .subscription_by_transport_id
                    .write()
                    .await
                    .insert(ctx.transport_id, generator_yielder.clone());

                let subscription_join_handle = ctx.server_context.redis_channel_subscriber.subscribe(
                    QUEST_UPDATES_CHANNEL_NAME,
                    move |user_update: UserUpdate| {
                        let generator_yielder = generator_yielder.clone();
                        let ids = quest_instance_ids.clone();
                        async move {
                            if let Some(Message::QuestState(state)) = &user_update.message {
                                if ids.contains(&state.quest_instance_id) {
                                    if generator_yielder.r#yield(user_update).await.is_err() {
                                        error!(
                                            "User Update received > Couldn't send update to subscriptors"
                                        );
                                        return Err(())
                                    } else {
                                        return Ok(());
                                    }
                                }
                            }
                            Ok(())
                        }
                    },
                );

                ctx.server_context
                    .subscriptions_handle_by_transport_id
                    .write()
                    .await
                    .insert(ctx.transport_id, subscription_join_handle);
                Ok(generator)
            }
            Err(err) => Err(err),
        }
    }
}

impl RemoteErrorResponse for QuestError {
    fn error_code(&self) -> u32 {
        match self {
            QuestError::QuestValidation(_) => 400,
            QuestError::CommonError(_) => 500,
            QuestError::DeserializationError => 400,
        }
    }

    fn error_message(&self) -> String {
        self.to_string()
    }
}
