use crate::binary::mapper;
use crate::binary::sender::Sender;
use crate::state::command::EntryCommand;
use crate::streaming::session::Session;
use crate::streaming::systems::system::SharedSystem;
use anyhow::Result;
use error_set::ResultContext;
use iggy::consumer_groups::create_consumer_group::CreateConsumerGroup;
use iggy::error::IggyError;
use tracing::{debug, instrument};

#[instrument(skip_all, fields(iggy_user_id = session.get_user_id(), iggy_client_id = session.client_id, iggy_stream_id = command.stream_id.as_string(), iggy_topic_id = command.topic_id.as_string()))]
pub async fn handle(
    command: CreateConsumerGroup,
    sender: &mut dyn Sender,
    session: &Session,
    system: &SharedSystem,
) -> Result<(), IggyError> {
    debug!("session: {session}, command: {command}");
    let response;
    {
        let mut system = system.write().await;
        let consumer_group = system
            .create_consumer_group(
                session,
                &command.stream_id,
                &command.topic_id,
                command.group_id,
                &command.name,
            )
            .await
            .with_error(|_| {
                format!(
                    "Failed to create consumer group for stream_id: {}, topic_id: {}, group_id: {:?}, session: {:?}",
                    command.stream_id, command.topic_id, command.group_id, session
                )
            })?;
        let consumer_group = consumer_group.read().await;
        response = mapper::map_consumer_group(&consumer_group).await;
    }

    let system = system.read().await;
    let stream_id = command.stream_id.clone();
    let topic_id = command.topic_id.clone();
    let group_id = command.group_id.clone();

    system
        .state
        .apply(
            session.get_user_id(),
            EntryCommand::CreateConsumerGroup(command),
        )
        .await
        .with_error(|_| {
            format!(
                "Failed to apply create consumer group for stream_id: {}, topic_id: {}, group_id: {:?}, session: {}",
                stream_id, topic_id, group_id, session
            )
        })?;
    sender.send_ok_response(&response).await?;
    Ok(())
}
