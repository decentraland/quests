use super::QuestsRpcServerContext;
use crate::{
    api::routes::errors::CommonError,
    domain::{
        events::{add_event_controller, AddEventError},
        quests::{self, start_quest, QuestError},
    },
};
use dcl_rpc::{
    rpc_protocol::RemoteErrorResponse, service_module_definition::ProcedureContext,
    stream_protocol::Generator,
};
use log::error;
use quests_message_broker::channel::{ChannelPublisher, ChannelSubscriber};
use quests_protocol::definitions::*;
use quests_system::{get_all_quest_states_by_user_address, get_quest};
use quests_system::{get_instance_state, QUESTS_CHANNEL_NAME};
use tokio::time::Instant;

pub struct QuestsServiceImplementation;

type QuestRpcResult<T> = Result<T, ServiceError>;

#[async_trait::async_trait]
impl QuestsServiceServer<QuestsRpcServerContext, ServiceError> for QuestsServiceImplementation {
    async fn start_quest(
        &self,
        request: StartQuestRequest,
        context: ProcedureContext<QuestsRpcServerContext>,
    ) -> QuestRpcResult<StartQuestResponse> {
        let record_procedure_duration = context
            .server_context
            .metrics_collector
            .record_procedure_call_duration(Procedure::StartQuest);

        context
            .server_context
            .metrics_collector
            .record_in_procedure_call_size(Procedure::StartQuest, request.encoded_len());

        let StartQuestRequest { quest_id } = request;
        let transport_contexts = context.server_context.transport_contexts.read().await;
        let Some(transport_context) = transport_contexts.get(&context.transport_id) else {
            // should not be possible
            context
                .server_context
                .metrics_collector
                .record_procedure_call(Procedure::StartQuest, Status::NotExistsTransportID);
            return Err(ServiceError::NotExistsTransportID);
        };

        match start_quest(
            context.server_context.db.clone(),
            &transport_context.user_address.to_string(),
            &quest_id,
        )
        .await
        {
            Ok(new_quest_instance_id) => {
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
                            message: Some(user_update::Message::NewQuestStarted(QuestInstance {
                                id: new_quest_instance_id,
                                quest: Some(quest),
                                state: Some(quest_state),
                            })),
                            user_address: transport_context.user_address.to_string(),
                        };
                        context
                            .server_context
                            .redis_channel_publisher
                            .publish(user_update)
                            .await;
                    }
                    Err(err) => {
                        error!("QuestServiceImplementation > StartQuest Error > Calculating state > {err:?}");
                    }
                }

                context
                    .server_context
                    .metrics_collector
                    .record_procedure_call(Procedure::StartQuest, Status::Accepted);

                record_procedure_duration(Status::Accepted);

                let response = StartQuestResponse::accepted();
                context
                    .server_context
                    .metrics_collector
                    .record_out_procedure_call_size(
                        Procedure::StartQuest,
                        Status::Accepted,
                        response.encoded_len(),
                    );

                Ok(response)
            }
            Err(err) => {
                error!("QuestsServiceImplementation > StartQuest Error > QuestID: {quest_id} > {err:?}");
                match err {
                    QuestError::NotFoundOrInactive => {
                        context
                            .server_context
                            .metrics_collector
                            .record_procedure_call(Procedure::StartQuest, Status::NotFound);

                        record_procedure_duration(Status::NotFound);

                        let response = StartQuestResponse::invalid_quest();

                        context
                            .server_context
                            .metrics_collector
                            .record_out_procedure_call_size(
                                Procedure::StartQuest,
                                Status::NotFound,
                                response.encoded_len(),
                            );

                        Ok(response)
                    }
                    QuestError::QuestAlreadyStarted => {
                        context
                            .server_context
                            .metrics_collector
                            .record_procedure_call(
                                Procedure::StartQuest,
                                Status::QuestAlreadyStarted,
                            );

                        record_procedure_duration(Status::QuestAlreadyStarted);

                        let response = StartQuestResponse::quest_already_started();

                        context
                            .server_context
                            .metrics_collector
                            .record_out_procedure_call_size(
                                Procedure::StartQuest,
                                Status::QuestAlreadyStarted,
                                response.encoded_len(),
                            );

                        Ok(response)
                    }
                    QuestError::CommonError(CommonError::NotUUID) => {
                        context
                            .server_context
                            .metrics_collector
                            .record_procedure_call(Procedure::StartQuest, Status::NotUUID);

                        record_procedure_duration(Status::NotUUID);

                        let response = StartQuestResponse::not_uuid_error();

                        context
                            .server_context
                            .metrics_collector
                            .record_out_procedure_call_size(
                                Procedure::StartQuest,
                                Status::NotUUID,
                                response.encoded_len(),
                            );

                        Ok(response)
                    }
                    _ => {
                        context
                            .server_context
                            .metrics_collector
                            .record_procedure_call(
                                Procedure::StartQuest,
                                Status::InternalServerError,
                            );

                        record_procedure_duration(Status::InternalServerError);

                        let response = StartQuestResponse::internal_server_error();

                        context
                            .server_context
                            .metrics_collector
                            .record_out_procedure_call_size(
                                Procedure::StartQuest,
                                Status::InternalServerError,
                                response.encoded_len(),
                            );

                        Ok(response)
                    }
                }
            }
        }
    }

    async fn abort_quest(
        &self,
        request: AbortQuestRequest,
        context: ProcedureContext<QuestsRpcServerContext>,
    ) -> QuestRpcResult<AbortQuestResponse> {
        let record_procedure_duration = context
            .server_context
            .metrics_collector
            .record_procedure_call_duration(Procedure::AbortQuest);

        context
            .server_context
            .metrics_collector
            .record_in_procedure_call_size(Procedure::AbortQuest, request.encoded_len());

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
                Ok(_) => {
                    context
                        .server_context
                        .metrics_collector
                        .record_procedure_call(Procedure::AbortQuest, Status::Accepted);

                    record_procedure_duration(Status::Accepted);

                    let response = AbortQuestResponse::accepted();

                    context
                        .server_context
                        .metrics_collector
                        .record_out_procedure_call_size(
                            Procedure::AbortQuest,
                            Status::Accepted,
                            response.encoded_len(),
                        );

                    Ok(response)
                }
                Err(err) => {
                    error!("QuestsServiceImplementation > AbortQuest Error > InstanceID: {:?} > {err:?}", request.quest_instance_id);
                    match err {
                        QuestError::NotInstanceOwner => {
                            context
                                .server_context
                                .metrics_collector
                                .record_procedure_call(Procedure::AbortQuest, Status::NotAuth);

                            record_procedure_duration(Status::NotAuth);

                            let response = AbortQuestResponse::not_owner();

                            context
                                .server_context
                                .metrics_collector
                                .record_out_procedure_call_size(
                                    Procedure::AbortQuest,
                                    Status::NotAuth,
                                    response.encoded_len(),
                                );

                            Ok(response)
                        }
                        QuestError::CommonError(CommonError::NotFound) => {
                            context
                                .server_context
                                .metrics_collector
                                .record_procedure_call(Procedure::AbortQuest, Status::NotFound);

                            record_procedure_duration(Status::NotFound);

                            let response = AbortQuestResponse::not_found_quest_instance();

                            context
                                .server_context
                                .metrics_collector
                                .record_out_procedure_call_size(
                                    Procedure::AbortQuest,
                                    Status::NotFound,
                                    response.encoded_len(),
                                );

                            Ok(response)
                        }
                        QuestError::CommonError(CommonError::NotUUID) => {
                            context
                                .server_context
                                .metrics_collector
                                .record_procedure_call(Procedure::AbortQuest, Status::NotUUID);

                            record_procedure_duration(Status::NotUUID);

                            let response = AbortQuestResponse::not_uuid_error();

                            context
                                .server_context
                                .metrics_collector
                                .record_out_procedure_call_size(
                                    Procedure::AbortQuest,
                                    Status::NotUUID,
                                    response.encoded_len(),
                                );

                            Ok(response)
                        }
                        _ => {
                            context
                                .server_context
                                .metrics_collector
                                .record_procedure_call(
                                    Procedure::AbortQuest,
                                    Status::InternalServerError,
                                );
                            record_procedure_duration(Status::InternalServerError);

                            let response = AbortQuestResponse::internal_server_error();

                            context
                                .server_context
                                .metrics_collector
                                .record_out_procedure_call_size(
                                    Procedure::AbortQuest,
                                    Status::InternalServerError,
                                    response.encoded_len(),
                                );

                            Ok(response)
                        }
                    }
                }
            }
        } else {
            context
                .server_context
                .metrics_collector
                .record_procedure_call(Procedure::AbortQuest, Status::NotExistsTransportID);
            record_procedure_duration(Status::NotExistsTransportID);
            Err(ServiceError::NotExistsTransportID)
        }
    }

    async fn send_event(
        &self,
        request: EventRequest,
        context: ProcedureContext<QuestsRpcServerContext>,
    ) -> QuestRpcResult<EventResponse> {
        let record_procedure_duration = context
            .server_context
            .metrics_collector
            .record_procedure_call_duration(Procedure::SendEvent);

        context
            .server_context
            .metrics_collector
            .record_in_procedure_call_size(Procedure::SendEvent, request.encoded_len());

        let transport_contexts = context.server_context.transport_contexts.read().await;
        let Some(transport_context) = transport_contexts.get(&context.transport_id) else {
            context
                .server_context
                .metrics_collector
                .record_procedure_call(Procedure::SendEvent, Status::NotExistsTransportID);
            return Err(ServiceError::NotExistsTransportID);
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
            Ok(event_id) => {
                context
                    .server_context
                    .metrics_collector
                    .record_procedure_call(Procedure::SendEvent, Status::Accepted);

                record_procedure_duration(Status::Accepted);

                let response = EventResponse::accepted(event_id);

                context
                    .server_context
                    .metrics_collector
                    .record_out_procedure_call_size(
                        Procedure::SendEvent,
                        Status::Accepted,
                        response.encoded_len(),
                    );

                Ok(response)
            }
            Err(error) => {
                error!("QuestsServiceImplementation > SendEvent Error > {error:?}");
                match error {
                    AddEventError::NoAction => {
                        context
                            .server_context
                            .metrics_collector
                            .record_procedure_call(Procedure::SendEvent, Status::Ignored);

                        record_procedure_duration(Status::Ignored);

                        let response = EventResponse::ignored();

                        context
                            .server_context
                            .metrics_collector
                            .record_out_procedure_call_size(
                                Procedure::SendEvent,
                                Status::Ignored,
                                response.encoded_len(),
                            );

                        Ok(response)
                    }
                    AddEventError::PushFailed => {
                        context
                            .server_context
                            .metrics_collector
                            .record_procedure_call(
                                Procedure::SendEvent,
                                Status::InternalServerError,
                            );

                        record_procedure_duration(Status::InternalServerError);

                        let response = EventResponse::internal_server_error();

                        context
                            .server_context
                            .metrics_collector
                            .record_out_procedure_call_size(
                                Procedure::SendEvent,
                                Status::InternalServerError,
                                response.encoded_len(),
                            );

                        Ok(response)
                    }
                }
            }
        }
    }

    async fn subscribe(
        &self,
        context: ProcedureContext<QuestsRpcServerContext>,
    ) -> QuestRpcResult<ServerStreamResponse<UserUpdate>> {
        let record_procedure_duration = context
            .server_context
            .metrics_collector
            .record_procedure_call_duration(Procedure::Subscribe);

        let transport_contexts = context.server_context.transport_contexts.read().await;
        let Some(transport_context) = transport_contexts.get(&context.transport_id) else {
            context
                .server_context
                .metrics_collector
                .record_procedure_call(Procedure::Subscribe, Status::NotExistsTransportID);

            record_procedure_duration(Status::NotExistsTransportID);

            return Err(ServiceError::NotExistsTransportID);
        };

        let user_address = transport_context.user_address.to_string();
        drop(transport_contexts);
        let (generator, generator_yielder) = Generator::create();

        let moved_user_address = user_address.clone();
        let yielder = generator_yielder.clone();
        let metrics_collector = context.server_context.metrics_collector.clone();

        let subscription_join_handle = context.server_context.redis_channel_subscriber.subscribe(
            QUESTS_CHANNEL_NAME,
            move |user_update: UserUpdate| {
                let generator_yielder = yielder.clone();
                let user_address = moved_user_address.clone();
                let metrics_collector = metrics_collector.clone();
                // Just return false on failure
                async move {
                    match user_address.eq_ignore_ascii_case(&user_update.user_address) {
                        true => {
                            let bytes = user_update.encoded_len();
                            if generator_yielder.r#yield(user_update).await.is_err() {
                                error!(
                                    "User Update received > Couldn't send update to subscriptors"
                                );
                                false
                            } else {
                                metrics_collector.record_out_procedure_call_size(
                                    Procedure::Subscribe,
                                    Status::Stream,
                                    bytes,
                                );
                                true
                            }
                        }
                        false => true,
                    }
                }
            },
        );

        let accepted_response = UserUpdate {
            message: Some(user_update::Message::Subscribed(true)),
            user_address: user_address.clone(),
        };

        context
            .server_context
            .metrics_collector
            .record_out_procedure_call_size(
                Procedure::Subscribe,
                Status::Accepted,
                accepted_response.encoded_len(),
            );

        if let Err(err) = generator_yielder.r#yield(accepted_response).await {
            // Would be impossible to happen, an "unwrap()" should be safe here
            error!("QuestsServiceImplementation > Subscribe Error > Generator Error before returning it > {err:?}");
            return Err(ServiceError::InternalError);
        }

        context
            .server_context
            .transport_contexts
            .write()
            .await
            .entry(context.transport_id)
            .and_modify(|current_context| {
                current_context.subscription_handle =
                    Some((subscription_join_handle, Instant::now()));
            });

        context
            .server_context
            .metrics_collector
            .record_procedure_call(Procedure::Subscribe, Status::Accepted);
        record_procedure_duration(Status::Accepted);

        Ok(generator)
    }

    async fn get_all_quests(
        &self,
        context: ProcedureContext<QuestsRpcServerContext>,
    ) -> QuestRpcResult<GetAllQuestsResponse> {
        let record_procedure_duration = context
            .server_context
            .metrics_collector
            .record_procedure_call_duration(Procedure::GetAllQuests);

        let transport_contexts = context.server_context.transport_contexts.read().await;
        let Some(transport_context) = transport_contexts.get(&context.transport_id) else {
            context
                .server_context
                .metrics_collector
                .record_procedure_call(Procedure::GetAllQuests, Status::NotExistsTransportID);

            record_procedure_duration(Status::NotExistsTransportID);

            return Err(ServiceError::NotExistsTransportID);
        };

        let user_address = transport_context.user_address.to_string();
        drop(transport_contexts);

        match get_all_quest_states_by_user_address(context.server_context.db.clone(), &user_address)
            .await
        {
            Ok(mut quest_states) => {
                let mut quests = Vec::new();
                for (instance_id, (ref mut quest, state)) in quest_states.iter_mut() {
                    quest.hide_actions();
                    let quest_definition_and_state = QuestInstance {
                        id: instance_id.to_string(),
                        quest: Some(quest.clone()),
                        state: Some(state.clone()),
                    };
                    quests.push(quest_definition_and_state);
                }
                context
                    .server_context
                    .metrics_collector
                    .record_procedure_call(Procedure::GetAllQuests, Status::Accepted);

                record_procedure_duration(Status::Accepted);

                let response = GetAllQuestsResponse::ok(quests);

                context
                    .server_context
                    .metrics_collector
                    .record_out_procedure_call_size(
                        Procedure::GetAllQuests,
                        Status::Accepted,
                        response.encoded_len(),
                    );

                Ok(response)
            }
            Err(err) => {
                error!("QuestsServiceImplementation > GetAllQuests > get_all_quest_states_by_user_address > {err:?}");
                context
                    .server_context
                    .metrics_collector
                    .record_procedure_call(Procedure::GetAllQuests, Status::InternalServerError);

                record_procedure_duration(Status::InternalServerError);

                let response = GetAllQuestsResponse::internal_server_error();

                context
                    .server_context
                    .metrics_collector
                    .record_out_procedure_call_size(
                        Procedure::GetAllQuests,
                        Status::InternalServerError,
                        response.encoded_len(),
                    );

                Ok(response)
            }
        }
    }

    async fn get_quest_definition(
        &self,
        request: GetQuestDefinitionRequest,
        context: ProcedureContext<QuestsRpcServerContext>,
    ) -> QuestRpcResult<GetQuestDefinitionResponse> {
        let record_procedure_duration = context
            .server_context
            .metrics_collector
            .record_procedure_call_duration(Procedure::GetQuestDefinition);

        match get_quest(context.server_context.db.clone(), &request.quest_id).await {
            Ok(mut quest) => {
                context
                    .server_context
                    .metrics_collector
                    .record_procedure_call(Procedure::GetQuestDefinition, Status::Accepted);

                record_procedure_duration(Status::Accepted);

                quest.hide_actions();

                let response = GetQuestDefinitionResponse::ok(quest);

                context
                    .server_context
                    .metrics_collector
                    .record_out_procedure_call_size(
                        Procedure::GetQuestDefinition,
                        Status::Accepted,
                        response.encoded_len(),
                    );

                Ok(response)
            }
            Err(_) => {
                context
                    .server_context
                    .metrics_collector
                    .record_procedure_call(
                        Procedure::GetQuestDefinition,
                        Status::InternalServerError,
                    );

                record_procedure_duration(Status::InternalServerError);

                let response = GetQuestDefinitionResponse::internal_server_error();

                context
                    .server_context
                    .metrics_collector
                    .record_out_procedure_call_size(
                        Procedure::GetQuestDefinition,
                        Status::InternalServerError,
                        response.encoded_len(),
                    );

                Ok(response)
            }
        }
    }
}

pub enum ServiceError {
    NotExistsTransportID,
    InternalError,
}

impl RemoteErrorResponse for ServiceError {
    fn error_code(&self) -> u32 {
        match self {
            Self::NotExistsTransportID => 1,
            Self::InternalError => 2,
        }
    }

    fn error_message(&self) -> String {
        match self {
            Self::NotExistsTransportID => "Not exists transport id".to_string(),
            Self::InternalError => "Internal error".to_string(),
        }
    }
}

enum Procedure {
    StartQuest,
    AbortQuest,
    SendEvent,
    Subscribe,
    GetAllQuests,
    GetQuestDefinition,
}

impl<'a> From<Procedure> for &'a str {
    fn from(val: Procedure) -> Self {
        match val {
            Procedure::StartQuest => "StartQuest",
            Procedure::AbortQuest => "AbortQuest",
            Procedure::SendEvent => "SendEvent",
            Procedure::Subscribe => "Subscribe",
            Procedure::GetAllQuests => "GetAllQuests",
            Procedure::GetQuestDefinition => "GetQuestDefinition",
        }
    }
}

enum Status {
    Accepted,
    NotUUID,
    InternalServerError,
    NotAuth,
    NotExistsTransportID,
    NotFound,
    QuestAlreadyStarted,
    Ignored,
    Stream,
}

impl<'a> From<Status> for &'a str {
    fn from(value: Status) -> Self {
        match value {
            Status::Accepted => "ACCEPTED",
            Status::NotUUID => "NOT_UUID",
            Status::NotAuth => "NOT_AUTH",
            Status::InternalServerError => "INTERNAL_SERVER_ERROR",
            Status::NotFound => "NOT_FOUND",
            Status::QuestAlreadyStarted => "QUEST_ALREADY_STARTED",
            Status::NotExistsTransportID => "NOT_EXISTS_TRANSPORT_ID",
            Status::Ignored => "IGNORED",
            Status::Stream => "STREAM",
        }
    }
}
