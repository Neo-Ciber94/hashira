use std::fmt;
use http::Uri;
use serde::{de::Visitor, Deserializer, Serializer};

pub fn serialize<S>(uri: &Uri, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&uri.to_string())
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<Uri, D::Error>
where
    D: Deserializer<'de>,
{
    struct UriVisitor;
    impl<'de> Visitor<'de> for UriVisitor {
        type Value = Uri;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("`Uri`")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Uri::try_from(v).map_err(|err| serde::de::Error::custom(err))
        }
    }

    deserializer.deserialize_str(UriVisitor)
}
