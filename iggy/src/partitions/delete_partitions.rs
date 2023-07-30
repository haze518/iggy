use crate::bytes_serializable::BytesSerializable;
use crate::command::CommandPayload;
use crate::error::Error;
use crate::validatable::Validatable;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::str::FromStr;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct DeletePartitions {
    #[serde(skip)]
    pub stream_id: u32,
    #[serde(skip)]
    pub topic_id: u32,
    #[serde(default = "default_partitions_count")]
    pub partitions_count: u32,
}

fn default_partitions_count() -> u32 {
    0
}

impl CommandPayload for DeletePartitions {}

impl Default for DeletePartitions {
    fn default() -> Self {
        DeletePartitions {
            stream_id: 1,
            topic_id: 1,
            partitions_count: 1,
        }
    }
}

impl Validatable for DeletePartitions {
    fn validate(&self) -> Result<(), Error> {
        if self.stream_id == 0 {
            return Err(Error::InvalidStreamId);
        }

        if self.topic_id == 0 {
            return Err(Error::InvalidTopicId);
        }

        Ok(())
    }
}

impl FromStr for DeletePartitions {
    type Err = Error;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let parts = input.split('|').collect::<Vec<&str>>();
        if parts.len() != 3 {
            return Err(Error::InvalidCommand);
        }

        let stream_id = parts[0].parse::<u32>()?;
        let topic_id = parts[1].parse::<u32>()?;
        let partitions_count = parts[2].parse::<u32>()?;
        let command = DeletePartitions {
            stream_id,
            topic_id,
            partitions_count,
        };
        command.validate()?;
        Ok(command)
    }
}

impl BytesSerializable for DeletePartitions {
    fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(12);
        bytes.extend(self.stream_id.to_le_bytes());
        bytes.extend(self.topic_id.to_le_bytes());
        bytes.extend(self.partitions_count.to_le_bytes());
        bytes
    }

    fn from_bytes(bytes: &[u8]) -> Result<DeletePartitions, Error> {
        if bytes.len() != 12 {
            return Err(Error::InvalidCommand);
        }

        let stream_id = u32::from_le_bytes(bytes[..4].try_into()?);
        let topic_id = u32::from_le_bytes(bytes[4..8].try_into()?);
        let partitions_count = u32::from_le_bytes(bytes[8..12].try_into()?);
        let command = DeletePartitions {
            stream_id,
            topic_id,
            partitions_count,
        };
        command.validate()?;
        Ok(command)
    }
}

impl Display for DeletePartitions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}|{}|{}",
            self.stream_id, self.topic_id, self.partitions_count
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_be_serialized_as_bytes() {
        let command = DeletePartitions {
            stream_id: 1,
            topic_id: 2,
            partitions_count: 3,
        };

        let bytes = command.as_bytes();
        let stream_id = u32::from_le_bytes(bytes[..4].try_into().unwrap());
        let topic_id = u32::from_le_bytes(bytes[4..8].try_into().unwrap());
        let partitions_count = u32::from_le_bytes(bytes[8..12].try_into().unwrap());

        assert!(!bytes.is_empty());
        assert_eq!(stream_id, command.stream_id);
        assert_eq!(topic_id, command.topic_id);
        assert_eq!(partitions_count, command.partitions_count);
    }

    #[test]
    fn should_be_deserialized_from_bytes() {
        let stream_id = 1u32;
        let topic_id = 2u32;
        let partitions_count = 3u32;
        let bytes = [
            stream_id.to_le_bytes(),
            topic_id.to_le_bytes(),
            partitions_count.to_le_bytes(),
        ]
        .concat();
        let command = DeletePartitions::from_bytes(&bytes);
        assert!(command.is_ok());

        let command = command.unwrap();
        assert_eq!(command.stream_id, stream_id);
        assert_eq!(command.topic_id, topic_id);
        assert_eq!(command.partitions_count, partitions_count);
    }

    #[test]
    fn should_be_read_from_string() {
        let stream_id = 1u32;
        let topic_id = 2u32;
        let partitions_count = 3u32;
        let input = format!("{}|{}|{}", stream_id, topic_id, partitions_count);
        let command = DeletePartitions::from_str(&input);
        assert!(command.is_ok());

        let command = command.unwrap();
        assert_eq!(command.stream_id, stream_id);
        assert_eq!(command.topic_id, topic_id);
        assert_eq!(command.partitions_count, partitions_count);
    }
}
