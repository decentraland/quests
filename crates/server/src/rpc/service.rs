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
use quests_message_broker::{channel::ChannelSubscriber, QUEST_UPDATES_CHANNEL_NAME};
use quests_protocol::quests::{
    user_update, AbortQuestRequest, AbortQuestResponse, EventRequest, EventResponse,
    GetAllQuestsResponse, GetQuestDefinitionRequest, GetQuestDefinitionResponse, ProtoQuest,
    QuestInstance, QuestStateWithData, QuestsServiceServer, ServerStreamResponse,
    StartQuestRequest, StartQuestResponse, UserAddress, UserUpdate,
};

pub struct QuestsServiceImplementation {}

type QuestRpcResult<T> = Result<T, UnableToOpenStream>;

#[async_trait::async_trait]
impl QuestsServiceServer<QuestsRpcServerContext, UnableToOpenStream>
    for QuestsServiceImplementation
{
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
                                log::error!("QuestServiceImplementation > StartQuest Error > Calculating state >{err:?}");
                                // TODO: Returns an error instead of false?
                            }
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
        match quests::abandon_quest(
            context.server_context.db.clone(),
            &request.user_address,
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
    }

    // TODO: Add tracing instrument
    async fn send_event(
        &self,
        request: EventRequest,
        context: ProcedureContext<QuestsRpcServerContext>,
    ) -> QuestRpcResult<EventResponse> {
        match add_event_controller(context.server_context.redis_events_queue.clone(), request).await
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
                let quest_instance_ids = states
                    .iter()
                    .map(|state| state.0.clone())
                    .collect::<Vec<_>>();

                let yielder = generator_yielder.clone();
                let subscription_join_handle = context.server_context.redis_channel_subscriber.subscribe(
                    QUEST_UPDATES_CHANNEL_NAME,
                    move |user_update: UserUpdate| {
                        let generator_yielder = yielder.clone();
                        let ids = quest_instance_ids.clone();
                        async move {
                            if let Some(user_update::Message::QuestStateUpdate(state)) = &user_update.message {
                                if let Some(quest) = &state.quest_data {
                                    if ids.contains(&quest.quest_instance_id) {
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
            Err(_) => Err(UnableToOpenStream {}),
        }
    }

    async fn get_all_quests(
        &self,
        request: UserAddress,
        context: ProcedureContext<QuestsRpcServerContext>,
    ) -> QuestRpcResult<GetAllQuestsResponse> {
        match quests::get_all_quest_states_by_user_address(
            context.server_context.db.clone(),
            request.user_address,
        )
        .await
        {
            Ok(quest_states) => {
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
        match quests::get_quest(context.server_context.db.clone(), request.quest_id).await {
            Ok(quest) => Ok(GetQuestDefinitionResponse::ok(quest)),
            Err(_) => Ok(GetQuestDefinitionResponse::internal_server_error()),
        }
    }
}

pub struct UnableToOpenStream;

impl RemoteErrorResponse for UnableToOpenStream {
    fn error_code(&self) -> u32 {
        500
    }

    fn error_message(&self) -> String {
        "Service Error: Unable to open a stream".to_string()
    }
}
