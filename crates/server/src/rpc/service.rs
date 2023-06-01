use super::QuestsRpcServerContext;
use crate::{
    api::routes::errors::CommonError,
    domain::{
        events::{add_event_controller, AddEventError},
        quests::{
            self, get_all_quest_states_by_user_address, get_instance_state, start_quest, QuestError,
        },
    },
};
use dcl_rpc::{
    rpc_protocol::RemoteErrorResponse, service_module_definition::ProcedureContext,
    stream_protocol::Generator,
};
use log::error;
use quests_message_broker::channel::ChannelSubscriber;
use quests_protocol::definitions::*;
use quests_system::QUESTS_CHANNEL_NAME;

pub struct QuestsServiceImplementation {}

type QuestRpcResult<T> = Result<T, ServiceErrors>;

#[async_trait::async_trait]
impl QuestsServiceServer<QuestsRpcServerContext, ServiceErrors> for QuestsServiceImplementation {
    // TODO: Add tracing instrument
    async fn start_quest(
        &self,
        request: StartQuestRequest,
        context: ProcedureContext<QuestsRpcServerContext>,
    ) -> QuestRpcResult<StartQuestResponse> {
        let StartQuestRequest { quest_id } = request;
        let transport_contexts = context.server_context.transport_contexts.read().await;
        let Some(transport_context) = transport_contexts.get(&context.transport_id) else {
            // should not be possible
            return Err(ServiceErrors::NotExistsTransportID)
        };

        match start_quest(
            context.server_context.db.clone(),
            &transport_context.user_address.to_string(),
            &quest_id,
        )
        .await
        {
            Ok(new_quest_instance_id) => {
                if let Some(subscription) = &transport_context.subscription {
                    match get_instance_state(
                        context.server_context.db.clone(),
                        &quest_id,
                        &new_quest_instance_id,
                    )
                    .await
                    {
                        Ok((quest, quest_state)) => {
                            transport_context
                                .quest_instance_ids
                                .lock()
                                .await
                                .push(new_quest_instance_id.clone());

                            let user_update = UserUpdate {
                                message: Some(user_update::Message::NewQuestStarted(
                                    QuestStateWithData {
                                        quest_instance_id: new_quest_instance_id.clone(),
                                        name: quest.name,
                                        description: quest.description,
                                        quest_state: Some(quest_state),
                                    },
                                )),
                            };
                            if subscription.r#yield(user_update).await.is_err() {
                                log::error!("QuestServiceImplementation > StartQuest Error > Not able to send update to susbcription")
                            }
                        }
                        Err(err) => {
                            log::error!("QuestServiceImplementation > StartQuest Error > Calculating state > {err:?}");
                        }
                    }
                }

                Ok(StartQuestResponse::accepted())
            }
            Err(err) => {
                log::error!("QuestsServiceImplementation > StartQuest Error > {err:?}");
                match err {
                    QuestError::NotFoundOrInactive => Ok(StartQuestResponse::invalid_quest()),
                    QuestError::CommonError(CommonError::NotUUID) => {
                        Ok(StartQuestResponse::not_uuid_error())
                    }
                    _ => Ok(StartQuestResponse::internal_server_error()),
                }
            }
        }
    }

    // TODO: Add tracing instrument
    async fn abort_quest(
        &self,
        request: AbortQuestRequest,
        context: ProcedureContext<QuestsRpcServerContext>,
    ) -> QuestRpcResult<AbortQuestResponse> {
        let transport_contexts = context.server_context.transport_contexts.read().await;
        if let Some(transport_context) = transport_contexts.get(&context.transport_id) {
            let user_address = transport_context.user_address.to_string();
            drop(transport_contexts);

            match quests::abandon_quest(
                context.server_context.db.clone(),
                &user_address,
                &request.quest_instance_id,
            )
            .await
            {
                Ok(_) => Ok(AbortQuestResponse::accepted()),
                Err(err) => match err {
                    QuestError::NotInstanceOwner => Ok(AbortQuestResponse::not_owner()),
                    QuestError::CommonError(CommonError::NotFound) => {
                        Ok(AbortQuestResponse::not_found_quest_instance())
                    }
                    QuestError::CommonError(CommonError::NotUUID) => {
                        Ok(AbortQuestResponse::not_uuid_error())
                    }
                    _ => Ok(AbortQuestResponse::internal_server_error()),
                },
            }
        } else {
            Err(ServiceErrors::NotExistsTransportID)
        }
    }

    // TODO: Add tracing instrument
    async fn send_event(
        &self,
        request: EventRequest,
        context: ProcedureContext<QuestsRpcServerContext>,
    ) -> QuestRpcResult<EventResponse> {
        let transport_contexts = context.server_context.transport_contexts.read().await;
        let Some(transport_context) = transport_contexts.get(&context.transport_id) else {
            return Err(ServiceErrors::NotExistsTransportID);
        };

        let user_address = transport_context.user_address.to_string();
        drop(transport_contexts);

        match add_event_controller(
            context.server_context.redis_events_queue.clone(),
            &user_address,
            request,
        )
        .await
        {
            Ok(event_id) => Ok(EventResponse::accepted(event_id)),
            Err(error) => {
                log::error!("QuestsServiceImplementation > SendEvent Error > {error:?}");
                match error {
                    AddEventError::NoAction => Ok(EventResponse::ignored()),
                    AddEventError::PushFailed => Ok(EventResponse::internal_server_error()),
                }
            }
        }
    }

    // TODO: Add tracing instrument
    async fn subscribe(
        &self,
        context: ProcedureContext<QuestsRpcServerContext>,
    ) -> QuestRpcResult<ServerStreamResponse<UserUpdate>> {
        let transport_contexts = context.server_context.transport_contexts.read().await;
        let Some(transport_context) = transport_contexts.get(&context.transport_id) else {
            return Err(ServiceErrors::NotExistsTransportID);
        };

        match get_all_quest_states_by_user_address(
            context.server_context.db.clone(),
            transport_context.user_address.to_string(),
        )
        .await
        {
            Ok(states) => {
                let (generator, generator_yielder) = Generator::create();
                let mut ids = states.iter().map(|(id, _)| id.clone()).collect::<Vec<_>>();
                let quest_instance_ids = transport_context.quest_instance_ids.clone();
                drop(transport_contexts);

                quest_instance_ids.lock().await.append(&mut ids);

                let yielder = generator_yielder.clone();
                let subscription_join_handle = context.server_context.redis_channel_subscriber.subscribe(
                    QUESTS_CHANNEL_NAME,
                    move |user_update: UserUpdate| {
                        let generator_yielder = yielder.clone();
                        let ids = quest_instance_ids.clone();
                        async move {
                            if let Some(user_update::Message::QuestStateUpdate(QuestStateUpdate {
                                quest_data: Some(quest_data),
                                ..
                            })) = &user_update.message
                            {
                                if ids.lock().await.contains(&quest_data.quest_instance_id) {
                                    if generator_yielder.r#yield(user_update).await.is_err() {
                                        error!(
                                            "User Update received > Couldn't send update to subscriptors"
                                        );
                                        return false; // Just return false on failure
                                    } else {
                                        return true;
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
            Err(_) => Err(ServiceErrors::UnableToOpenStream),
        }
    }

    async fn get_all_quests(
        &self,
        context: ProcedureContext<QuestsRpcServerContext>,
    ) -> QuestRpcResult<GetAllQuestsResponse> {
        let transport_contexts = context.server_context.transport_contexts.read().await;
        let Some(transport_context) = transport_contexts.get(&context.transport_id) else {
            return Err(ServiceErrors::NotExistsTransportID);
        };

        let user_address = transport_context.user_address.to_string();
        drop(transport_contexts);

        match quests::get_all_quest_states_by_user_address(
            context.server_context.db.clone(),
            user_address,
        )
        .await
        {
            Ok(quest_states) => {
                let mut quests = Vec::new();
                for (instance_id, (quest, state)) in quest_states {
                    let quest_definition_and_state = QuestInstance {
                        instance_id,
                        quest: Some(quest),
                        state: Some(state),
                    };
                    quests.push(quest_definition_and_state);
                }
                Ok(GetAllQuestsResponse::ok(quests))
            }
            Err(_) => Ok(GetAllQuestsResponse::internal_server_error()),
        }
    }

    async fn get_quest_definition(
        &self,
        request: GetQuestDefinitionRequest,
        context: ProcedureContext<QuestsRpcServerContext>,
    ) -> QuestRpcResult<GetQuestDefinitionResponse> {
        match quests::get_quest(context.server_context.db.clone(), &request.quest_id).await {
            Ok((quest, _)) => Ok(GetQuestDefinitionResponse::ok(quest)),
            Err(_) => Ok(GetQuestDefinitionResponse::internal_server_error()),
        }
    }
}

pub enum ServiceErrors {
    UnableToOpenStream,
    NotExistsTransportID,
}

impl RemoteErrorResponse for ServiceErrors {
    fn error_code(&self) -> u32 {
        match self {
            Self::UnableToOpenStream => 1,
            Self::NotExistsTransportID => 2,
        }
    }

    fn error_message(&self) -> String {
        match self {
            Self::UnableToOpenStream => "Unable to open stream".to_string(),
            Self::NotExistsTransportID => "Not exists transport id".to_string(),
        }
    }
}
