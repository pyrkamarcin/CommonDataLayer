use super::*;
use serde::{
    de::{Error, Visitor},
    Deserializer, Serializer,
};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct ObjectIdPair {
    pub schema_id: Uuid,
    pub object_id: Uuid,
}

impl Serialize for ObjectIdPair {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("{},{}", self.schema_id, self.object_id))
    }
}

impl<'de> Deserialize<'de> for ObjectIdPair {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct Helper;

        impl<'de> Visitor<'de> for Helper {
            type Value = ObjectIdPair;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(formatter, "two UUID separated by single comma")
            }

            fn visit_str<E: Error>(self, value: &str) -> Result<ObjectIdPair, E> {
                let mut split = value.split(',');
                let schema_id = split.next();
                let object_id = split.next();
                let rest = split.next();

                match (schema_id, object_id, rest) {
                    (Some(schema_id), Some(object_id), None) => {
                        let schema_id =
                            Uuid::parse_str(schema_id).map_err(|e| E::custom(format!("{}", e)))?;
                        let object_id =
                            Uuid::parse_str(object_id).map_err(|e| E::custom(format!("{}", e)))?;

                        Ok(ObjectIdPair {
                            schema_id,
                            object_id,
                        })
                    }
                    _ => Err(E::custom("Expected two UUID separated by single comma")),
                }
            }
        }

        deserializer.deserialize_str(Helper)
    }
}
