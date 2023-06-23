use crate::error::Error;
use crate::models::client_info::ClientInfo;
use crate::models::message::Message;
use crate::models::offset::Offset;
use crate::models::partition::Partition;
use crate::models::stream::{Stream, StreamDetails};
use crate::models::topic::{Topic, TopicDetails};
use std::str::from_utf8;

const EMPTY_MESSAGES: Vec<Message> = vec![];
const EMPTY_TOPICS: Vec<Topic> = vec![];
const EMPTY_STREAMS: Vec<Stream> = vec![];
const EMPTY_CLIENTS: Vec<ClientInfo> = vec![];

pub fn map_offset(payload: &[u8]) -> Result<Offset, Error> {
    let consumer_id = u32::from_le_bytes(payload[..4].try_into()?);
    let offset = u64::from_le_bytes(payload[4..12].try_into()?);
    Ok(Offset {
        consumer_id,
        offset,
    })
}

pub fn map_clients(payload: &[u8]) -> Result<Vec<ClientInfo>, Error> {
    if payload.is_empty() {
        return Ok(EMPTY_CLIENTS);
    }

    let mut clients = Vec::new();
    let length = payload.len();
    let mut position = 0;
    while position < length {
        let id = u32::from_le_bytes(payload[position..position + 4].try_into()?);
        let transport = payload[position + 4];
        let transport = match transport {
            1 => "TCP",
            2 => "QUIC",
            _ => "Unknown",
        }
        .to_string();
        let address_length =
            u32::from_le_bytes(payload[position + 5..position + 9].try_into()?) as usize;
        let address = from_utf8(&payload[position + 9..position + 9 + address_length])?.to_string();
        position += 4 + 1 + 4 + address_length;
        let client = ClientInfo {
            id,
            transport,
            address,
        };
        clients.push(client);
        if position >= length {
            break;
        }
    }
    clients.sort_by(|x, y| x.id.cmp(&y.id));
    Ok(clients)
}

pub fn map_messages(payload: &[u8]) -> Result<Vec<Message>, Error> {
    if payload.is_empty() {
        return Ok(EMPTY_MESSAGES);
    }

    const PROPERTIES_SIZE: usize = 36;
    let length = payload.len();
    let mut position = 4;
    let mut messages = Vec::new();
    while position < length {
        let offset = u64::from_le_bytes(payload[position..position + 8].try_into()?);
        let timestamp = u64::from_le_bytes(payload[position + 8..position + 16].try_into()?);
        let id = u128::from_le_bytes(payload[position + 16..position + 32].try_into()?);
        let message_length =
            u32::from_le_bytes(payload[position + 32..position + PROPERTIES_SIZE].try_into()?);

        let payload_range =
            position + PROPERTIES_SIZE..position + PROPERTIES_SIZE + message_length as usize;
        if payload_range.start > length || payload_range.end > length {
            break;
        }

        let payload = payload[payload_range].to_vec();
        let total_size = PROPERTIES_SIZE + message_length as usize;
        position += total_size;
        messages.push(Message {
            offset,
            timestamp,
            id,
            length: message_length,
            payload,
        });

        if position + PROPERTIES_SIZE >= length {
            break;
        }
    }

    messages.sort_by(|x, y| x.offset.cmp(&y.offset));
    Ok(messages)
}

pub fn map_streams(payload: &[u8]) -> Result<Vec<Stream>, Error> {
    if payload.is_empty() {
        return Ok(EMPTY_STREAMS);
    }

    let mut streams = Vec::new();
    let length = payload.len();
    let mut position = 0;
    while position < length {
        let (stream, read_bytes) = map_to_stream(payload, position)?;
        streams.push(stream);
        position += read_bytes;
        if position >= length {
            break;
        }
    }
    streams.sort_by(|x, y| x.id.cmp(&y.id));
    Ok(streams)
}

pub fn map_stream(payload: &[u8]) -> Result<StreamDetails, Error> {
    let (stream, mut position) = map_to_stream(payload, 0)?;
    let mut topics = Vec::new();
    let length = payload.len();
    while position < length {
        let (topic, read_bytes) = map_to_topic(payload, position)?;
        topics.push(topic);
        position += read_bytes;
        if position >= length {
            break;
        }
    }

    topics.sort_by(|x, y| x.id.cmp(&y.id));
    let stream = StreamDetails {
        id: stream.id,
        topics_count: stream.topics_count,
        name: stream.name,
        topics,
    };
    Ok(stream)
}

fn map_to_stream(payload: &[u8], position: usize) -> Result<(Stream, usize), Error> {
    let id = u32::from_le_bytes(payload[position..position + 4].try_into()?);
    let topics_count = u32::from_le_bytes(payload[position + 4..position + 8].try_into()?);
    let name_length = u32::from_le_bytes(payload[position + 8..position + 12].try_into()?) as usize;
    let name = from_utf8(&payload[position + 12..position + 12 + name_length])?.to_string();
    let read_bytes = 4 + 4 + 4 + name_length;
    Ok((
        Stream {
            id,
            topics_count,
            name,
        },
        read_bytes,
    ))
}

pub fn map_topics(payload: &[u8]) -> Result<Vec<Topic>, Error> {
    if payload.is_empty() {
        return Ok(EMPTY_TOPICS);
    }

    let mut topics = Vec::new();
    let length = payload.len();
    let mut position = 0;
    while position < length {
        let (topic, read_bytes) = map_to_topic(payload, position)?;
        topics.push(topic);
        position += read_bytes;
        if position >= length {
            break;
        }
    }
    topics.sort_by(|x, y| x.id.cmp(&y.id));
    Ok(topics)
}

pub fn map_topic(payload: &[u8]) -> Result<TopicDetails, Error> {
    let (topic, mut position) = map_to_stream(payload, 0)?;
    let mut partitions = Vec::new();
    let length = payload.len();
    while position < length {
        let (partition, read_bytes) = map_to_partition(payload, position)?;
        partitions.push(partition);
        position += read_bytes;
        if position >= length {
            break;
        }
    }

    partitions.sort_by(|x, y| x.id.cmp(&y.id));
    let topic = TopicDetails {
        id: topic.id,
        name: topic.name,
        partitions_count: partitions.len() as u32,
        partitions,
    };
    Ok(topic)
}

fn map_to_topic(payload: &[u8], position: usize) -> Result<(Topic, usize), Error> {
    let id = u32::from_le_bytes(payload[position..position + 4].try_into()?);
    let partitions_count = u32::from_le_bytes(payload[position + 4..position + 8].try_into()?);
    let name_length = u32::from_le_bytes(payload[position + 8..position + 12].try_into()?) as usize;
    let name = from_utf8(&payload[position + 12..position + 12 + name_length])?.to_string();
    let read_bytes = 4 + 4 + 4 + name_length;
    Ok((
        Topic {
            id,
            partitions_count,
            name,
        },
        read_bytes,
    ))
}

fn map_to_partition(payload: &[u8], position: usize) -> Result<(Partition, usize), Error> {
    let id = u32::from_le_bytes(payload[position..position + 4].try_into()?);
    let segments_count = u32::from_le_bytes(payload[position + 4..position + 8].try_into()?);
    let current_offset = u64::from_le_bytes(payload[position + 8..position + 16].try_into()?);
    let size_bytes = u64::from_le_bytes(payload[position + 16..position + 24].try_into()?);
    let read_bytes = 4 + 4 + 8 + 8;
    Ok((
        Partition {
            id,
            segments_count,
            current_offset,
            size_bytes,
        },
        read_bytes,
    ))
}