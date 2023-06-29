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

        let StartQuestRequest { quest_id } = request;
        let transport_contexts = context.server_context.transport_contexts.read().await;
        let Some(transport_context) = transport_contexts.get(&context.transport_id) else {
            // should not be possible
            context
            .server_context
            .metrics_collector
            .record_procedure_call(Procedure::StartQuest, Status::NotExistsTransportID);
            return Err(ServiceError::NotExistsTransportID)
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
                        log::error!("QuestServiceImplementation > StartQuest Error > Calculating state > {err:?}");
                    }
                }
                context
                    .server_context
                    .metrics_collector
                    .record_procedure_call(Procedure::StartQuest, Status::Accepted);
                record_procedure_duration(Status::Accepted);
                Ok(StartQuestResponse::accepted())
            }
            Err(err) => {
                log::error!("QuestsServiceImplementation > StartQuest Error > {err:?}");
                match err {
                    QuestError::NotFoundOrInactive => {
                        context
                            .server_context
                            .metrics_collector
                            .record_procedure_call(Procedure::StartQuest, Status::NotFound);
                        record_procedure_duration(Status::NotFound);
                        Ok(StartQuestResponse::invalid_quest())
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
                        Ok(StartQuestResponse::quest_already_started())
                    }
                    QuestError::CommonError(CommonError::NotUUID) => {
                        context
                            .server_context
                            .metrics_collector
                            .record_procedure_call(Procedure::StartQuest, Status::NotUUID);
                        record_procedure_duration(Status::NotUUID);
                        Ok(StartQuestResponse::not_uuid_error())
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
                        Ok(StartQuestResponse::internal_server_error())
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
                    Ok(AbortQuestResponse::accepted())
                }
                Err(err) => match err {
                    QuestError::NotInstanceOwner => {
                        context
                            .server_context
                            .metrics_collector
                            .record_procedure_call(Procedure::AbortQuest, Status::NotAuth);
                        record_procedure_duration(Status::NotAuth);
                        Ok(AbortQuestResponse::not_owner())
                    }
                    QuestError::CommonError(CommonError::NotFound) => {
                        context
                            .server_context
                            .metrics_collector
                            .record_procedure_call(Procedure::AbortQuest, Status::NotFound);
                        record_procedure_duration(Status::NotFound);
                        Ok(AbortQuestResponse::not_found_quest_instance())
                    }
                    QuestError::CommonError(CommonError::NotUUID) => {
                        context
                            .server_context
                            .metrics_collector
                            .record_procedure_call(Procedure::AbortQuest, Status::NotUUID);
                        record_procedure_duration(Status::NotUUID);
                        Ok(AbortQuestResponse::not_uuid_error())
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
                        Ok(AbortQuestResponse::internal_server_error())
                    }
                },
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
                Ok(EventResponse::accepted(event_id))
            }
            Err(error) => {
                log::error!("QuestsServiceImplementation > SendEvent Error > {error:?}");
                match error {
                    AddEventError::NoAction => {
                        context
                            .server_context
                            .metrics_collector
                            .record_procedure_call(Procedure::SendEvent, Status::Ignored);
                        record_procedure_duration(Status::Ignored);
                        Ok(EventResponse::ignored())
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
                        Ok(EventResponse::internal_server_error())
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
        let subscription_join_handle = context.server_context.redis_channel_subscriber.subscribe(
            QUESTS_CHANNEL_NAME,
            move |user_update: UserUpdate| {
                let generator_yielder = yielder.clone();
                let user_address = moved_user_address.clone();

                // Just return false on failure
                async move {
                    match user_address.eq_ignore_ascii_case(&user_update.user_address) {
                        true => {
                            if generator_yielder.r#yield(user_update).await.is_err() {
                                error!(
                                    "User Update received > Couldn't send update to subscriptors"
                                );
                                false
                            } else {
                                true
                            }
                        }
                        false => true,
                    }
                }
            },
        );

        if let Err(err) = generator_yielder
            .r#yield(UserUpdate {
                message: Some(user_update::Message::Subscribed(true)),
                user_address: user_address.clone(),
            })
            .await
        {
            // Would be impossible to happen, an "unwrap()" should be safe here
            log::error!("QuestsServiceImplementation > Subscribe Error > Generator Error before returning it > {err:?}");
            return Err(ServiceError::InternalError);
        }

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
            Ok(quest_states) => {
                let mut quests = Vec::new();
                for (instance_id, (quest, state)) in quest_states {
                    let quest_definition_and_state = QuestInstance {
                        id: instance_id,
                        quest: Some(quest),
                        state: Some(state),
                    };
                    quests.push(quest_definition_and_state);
                }
                context
                    .server_context
                    .metrics_collector
                    .record_procedure_call(Procedure::GetAllQuests, Status::Accepted);

                record_procedure_duration(Status::Accepted);

                Ok(GetAllQuestsResponse::ok(quests))
            }
            Err(_) => {
                context
                    .server_context
                    .metrics_collector
                    .record_procedure_call(Procedure::GetAllQuests, Status::InternalServerError);

                record_procedure_duration(Status::InternalServerError);

                Ok(GetAllQuestsResponse::internal_server_error())
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
            Ok(quest) => {
                context
                    .server_context
                    .metrics_collector
                    .record_procedure_call(Procedure::GetQuestDefinition, Status::Accepted);
                record_procedure_duration(Status::Accepted);
                Ok(GetQuestDefinitionResponse::ok(quest))
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
                Ok(GetQuestDefinitionResponse::internal_server_error())
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
        }
    }
}
