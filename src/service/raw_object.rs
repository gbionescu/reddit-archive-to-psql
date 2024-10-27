use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use sqlx::types::chrono::{DateTime, Utc};

#[derive(Serialize, Debug)]
pub enum AnyTimestamp {
    Integer(i64),
    String(String),
    Float(f64),
    Bool(bool),
}

impl<'de> Deserialize<'de> for AnyTimestamp {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        match value {
            Value::Number(num) => match num.as_i64() {
                Some(int_value) => Ok(AnyTimestamp::Integer(int_value)),
                None => match num.as_f64() {
                    Some(float_value) => Ok(AnyTimestamp::Float(float_value)),
                    None => panic!("expected int or float, found {:?}", num),
                },
            },
            Value::String(string_value) => Ok(AnyTimestamp::String(string_value)),
            Value::Bool(bool_value) => Ok(AnyTimestamp::Bool(bool_value)),
            _ => panic!("expected int, string, float, or bool, found {:?}", value),
        }
    }
}

impl From<&AnyTimestamp> for u64 {
    fn from(created_utc: &AnyTimestamp) -> Self {
        match created_utc {
            AnyTimestamp::Integer(timestamp) => *timestamp as u64,
            AnyTimestamp::String(timestamp) => timestamp.parse::<u64>().unwrap_or_default(),
            AnyTimestamp::Float(timestamp) => *timestamp as u64,
            AnyTimestamp::Bool(_) => 0,
        }
    }
}

// implement from AnyTimestamp to datetime
impl From<&AnyTimestamp> for DateTime<Utc> {
    fn from(created_utc: &AnyTimestamp) -> Self {
        match created_utc {
            AnyTimestamp::Integer(timestamp) => DateTime::from_timestamp(*timestamp, 0).unwrap(),
            AnyTimestamp::String(timestamp) => {
                let timestamp = timestamp.parse::<i64>().unwrap_or_default();
                DateTime::from_timestamp(timestamp, 0).unwrap()
            }
            AnyTimestamp::Float(timestamp) => {
                DateTime::from_timestamp(*timestamp as i64, 0).unwrap()
            }
            AnyTimestamp::Bool(_) => DateTime::from_timestamp(0, 0).unwrap(),
        }
    }
}
