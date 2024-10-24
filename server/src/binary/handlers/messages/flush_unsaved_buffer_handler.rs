use crate::binary::sender::Sender;
use crate::streaming::session::Session;
use crate::streaming::systems::system::SharedSystem;
use anyhow::Result;
use error_set::ResultContext;
use iggy::error::IggyError;
use iggy::messages::flush_unsaved_buffer::FlushUnsavedBuffer;
use tracing::{debug, instrument};

#[instrument(skip_all, fields(iggy_user_id = session.get_user_id(), iggy_client_id = session.client_id, iggy_stream_id = command.stream_id.as_string(), iggy_topic_id = command.topic_id.as_string(), iggy_partition_id = command.partition_id, iggy_fsync = command.fsync))]
pub async fn handle(
    command: FlushUnsavedBuffer,
    sender: &mut dyn Sender,
    session: &Session,
    system: &SharedSystem,
) -> Result<(), IggyError> {
    debug!("session: {session}, command: {command}");
    let system = system.read().await;
    let stream_id = command.stream_id.clone();
    let topic_id = command.topic_id.clone();
    let partition_id = command.partition_id.clone();
    let fsync = command.fsync;
    system
        .flush_unsaved_buffer(session, stream_id, topic_id, partition_id, fsync)
        .await
        .with_error(|_| {
            format!(
                "Failed to flush unsaved buffer for stream_id: {}, topic_id: {}, partition_id: {}, session: {}",
                command.stream_id, command.topic_id, command.partition_id, session
            )
        })?;
    sender.send_empty_ok_response().await?;
    Ok(())
}
