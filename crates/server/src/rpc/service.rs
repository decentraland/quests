use super::QuestsRpcServerContext;
use crate::domain::{
    events::add_event_controller,
    quests::{
        self, get_all_quest_states_by_user_address, get_instance_state, get_quest, start_quest,
        QuestError,
    },
};
use dcl_rpc::{
    rpc_protocol::RemoteErrorResponse, service_module_definition::ProcedureContext,
    stream_protocol::Generator,
};
use log::error;
use quests_message_broker::{channel::ChannelSubscriber, QUEST_UPDATES_CHANNEL_NAME};
use quests_protocol::quests::{
    user_update::Message, AbortQuestRequest, AbortQuestResponse, Event, EventResponse, ProtoQuest,
    QuestDefinitionRequest, QuestInstance, QuestOfferRequest, QuestOfferResponse, QuestOffering,
    QuestStateUpdate, Quests, QuestsServiceServer, ServerStreamResponse, StartQuestRequest,
    StartQuestResponse, UserAddress, UserUpdate,
};

pub struct QuestsServiceImplementation {}

type QuestRpcResult<T> = Result<T, QuestError>;

#[async_trait::async_trait]
impl QuestsServiceServer<QuestsRpcServerContext, QuestError> for QuestsServiceImplementation {
    async fn get_quest_offer(
        &self,
        request: QuestOfferRequest,
        context: ProcedureContext<QuestsRpcServerContext>,
    ) -> QuestRpcResult<QuestOfferResponse> {
        if let Ok(quest) = get_quest(context.server_context.db.clone(), &request.quest_id).await {
            let user_subscriptions_lock = context.server_context.transport_contexts.read().await;
            if let Some(transport_context) = user_subscriptions_lock.get(&context.transport_id) {
                if let Some(subscription) = &transport_context.subscription {
                    let user_update = UserUpdate {
                        message: Some(Message::QuestOffer(QuestOffering {
                            id: request.quest_id,
                            name: quest.name,
                            description: quest.description,
                            definition: Some(quest.definition),
                        })),
                    };
                    if subscription.r#yield(user_update).await.is_err() {
                        log::error!("QuestServiceImplementation > StartQuest Error > Not able to send update to susbcription")
                    }
                }
            }

            Ok(QuestOfferResponse { offered: true })
        } else {
            Ok(QuestOfferResponse { offered: false })
        }
    }

    // TODO: Add tracing instrument
    async fn start_quest(
        &self,
        request: StartQuestRequest,
        context: ProcedureContext<QuestsRpcServerContext>,
    ) -> QuestRpcResult<StartQuestResponse> {
        let StartQuestRequest {
            user_address,
            quest_id,
        } = request;
        match start_quest(context.server_context.db.clone(), &user_address, &quest_id).await {
            Ok(new_quest_instance_id) => {
                let user_subscriptions_lock =
                    context.server_context.transport_contexts.read().await;
                if let Some(transport_context) = user_subscriptions_lock.get(&context.transport_id)
                {
                    if let Some(subscription) = &transport_context.subscription {
                        match get_instance_state(
                            context.server_context.db.clone(),
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
                                if subscription.r#yield(user_update).await.is_err() {
                                    log::error!("QuestServiceImplementation > StartQuest Error > Not able to send update to susbcription")
                                }
                            }
                            Err(err) => {
                                log::error!("QuestServiceImplementation > StartQuest Error > Calculating state >{err:?}");
                                // TODO: Returns an error instead of false?
                            }
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
        request: AbortQuestRequest,
        context: ProcedureContext<QuestsRpcServerContext>,
    ) -> QuestRpcResult<AbortQuestResponse> {
        let accepted = quests::abandon_quest(
            context.server_context.db.clone(),
            &request.user_address,
            &request.quest_instance_id,
        )
        .await
        .is_ok();

        // TODO: Returns an error instead of false?
        Ok(AbortQuestResponse { accepted })
    }

    // TODO: Add tracing instrument
    async fn send_event(
        &self,
        request: Event,
        context: ProcedureContext<QuestsRpcServerContext>,
    ) -> QuestRpcResult<EventResponse> {
        match add_event_controller(context.server_context.redis_events_queue.clone(), request).await
        {
            Ok(event_id) => Ok(EventResponse {
                event_id: event_id as u32,
                accepted: true,
            }),
            Err(error) => {
                log::error!("QuestsServiceImplementation > SendEvent Error > {error:?}");
                // TODO: Returns an error instead of false?
                Ok(EventResponse {
                    event_id: 0,
                    accepted: false,
                })
            }
        }
    }

    // TODO: Add tracing instrument
    async fn subscribe(
        &self,
        request: UserAddress,
        context: ProcedureContext<QuestsRpcServerContext>,
    ) -> QuestRpcResult<ServerStreamResponse<UserUpdate>> {
        log::debug!("QuestsServiceImplementation > Subscribe > User {request:?} subscribed");
        match get_all_quest_states_by_user_address(
            context.server_context.db.clone(),
            request.user_address.to_string(),
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

                let yielder = generator_yielder.clone();
                let subscription_join_handle = context.server_context.redis_channel_subscriber.subscribe(
                    QUEST_UPDATES_CHANNEL_NAME,
                    move |user_update: UserUpdate| {
                        let generator_yielder = yielder.clone();
                        let ids = quest_instance_ids.clone();
                        async move {
                            if let Some(Message::QuestState(state)) = &user_update.message {
                                if ids.contains(&state.quest_instance_id) {
                                    if generator_yielder.r#yield(user_update).await.is_err() {
                                        error!(
                                            "User Update received > Couldn't send update to subscriptors"
                                        );
                                        return false // Just return false on failure
                                    } else {
                                        return true
                                    }
                                }
                            }
                            true
                        }
                    },
                );

                context
                    .server_context
                    .transport_contexts
                    .write()
                    .await
                    .entry(context.transport_id)
                    .and_modify(|current_context| {
                        current_context.subscription = Some(generator_yielder);
                        current_context.subscription_handle = Some(subscription_join_handle);
                    });
                Ok(generator)
            }
            Err(err) => Err(err),
        }
    }

    async fn get_all_quests(
        &self,
        request: UserAddress,
        context: ProcedureContext<QuestsRpcServerContext>,
    ) -> QuestRpcResult<Quests> {
        let quest_states = quests::get_all_quest_states_by_user_address(
            context.server_context.db.clone(),
            request.user_address,
        )
        .await?;

        let mut quests = Vec::new();
        for (instance_id, (quest, state)) in quest_states {
            let quest_definition_and_state = QuestInstance {
                instance_id,
                quest: Some(ProtoQuest {
                    name: quest.name,
                    description: quest.description,
                    definition: Some(quest.definition),
                }),
                state: Some(state),
            };
            quests.push(quest_definition_and_state);
        }

        Ok(Quests { quests })
    }

    async fn get_quest_definition(
        &self,
        request: QuestDefinitionRequest,
        context: ProcedureContext<QuestsRpcServerContext>,
    ) -> QuestRpcResult<ProtoQuest> {
        let quest = quests::get_quest(context.server_context.db.clone(), &request.quest_id).await?;

        Ok(ProtoQuest {
            name: quest.name,
            description: quest.description,
            definition: Some(quest.definition),
        })
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
