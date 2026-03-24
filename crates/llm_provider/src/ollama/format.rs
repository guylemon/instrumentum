use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq)]
pub enum Format {
    Json,
    Schema(Value),
}

impl Serialize for Format {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Format::Json => serializer.serialize_str("json"),
            Format::Schema(schema) => schema.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for Format {
    fn deserialize<D>(deserializer: D) -> Result<Format, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        match value {
            Value::String(s) if s == "json" => Ok(Format::Json),
            Value::Object(_) => Ok(Format::Schema(value)),
            _ => Err(serde::de::Error::custom(
                "expected \"json\" or a JSON object",
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_json_roundtrip() {
        let format = Format::Json;
        let json = serde_json::to_string(&format).unwrap();
        assert_eq!(json, r#""json""#);
        let deserialized: Format = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, Format::Json);
    }

    #[test]
    fn test_format_schema_roundtrip() {
        let schema = serde_json::json!({
            "type": "object",
            "properties": {
                "name": {"type": "string"},
                "age": {"type": "number"}
            },
            "required": ["name"]
        });
        let format = Format::Schema(schema.clone());
        let serialized = serde_json::to_string(&format).unwrap();
        let deserialized: Format = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, Format::Schema(schema));
    }
}
