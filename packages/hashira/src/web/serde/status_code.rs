use std::fmt;

use http::StatusCode;
use serde::{Serializer, Deserializer, de::Visitor};

pub fn serialize<S>(status_code: &StatusCode, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_u16(status_code.as_u16())
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<StatusCode, D::Error>
where
    D: Deserializer<'de>,
{
    struct StatusCodeVisitor;
    impl<'de> Visitor<'de> for StatusCodeVisitor {
        type Value = u16;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("`u16`")
        }

        fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            u16::try_from(v)
                .map_err(|_| serde::de::Error::custom(format!("`{v}` is out of u16 range")))
        }

        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            u16::try_from(v)
                .map_err(|_| serde::de::Error::custom(format!("`{v}` is out of u16 range")))
        }
    }

    let code = deserializer.deserialize_u16(StatusCodeVisitor)?;
    StatusCode::from_u16(code)
        .map_err(|_| serde::de::Error::custom(format!("Invalid status code: {code}")))
}
