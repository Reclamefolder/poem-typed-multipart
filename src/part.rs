//! Implementations for the [`FromMultiPartPart`] trait

use thiserror::Error;

/// Default error returned whenever a value is not absent in a multipart part
#[derive(Debug, Error)]
#[error("The field `{0}` was not found in the request")]
struct ValueNotFoundError(String);

/// Trait that should be implemented for typed that could be parsed from a multipart form-data
/// field.
pub trait FromMultiPartPart<'a>
where
    Self: Sized,
{
    /// The error returned when deserializing the part type fails
    type Error: std::error::Error + Send + Sync + 'static;

    /// Convert the provided byte blob into the part
    fn from_bytes(body: &'a [u8]) -> Result<Self, Self::Error>;

    /// Handler called when the value is absent from the request. A recovery strategy should be
    /// provided here if relevant.
    fn handle_absent_value(key: &str) -> Result<Self, poem::Error> {
        Err(poem::error::BadRequest(ValueNotFoundError(key.to_string())))
    }
}

impl FromMultiPartPart<'_> for String {
    type Error = std::string::FromUtf8Error;

    fn from_bytes(body: &[u8]) -> Result<String, Self::Error> {
        String::from_utf8(body.to_vec())
    }
}

#[derive(Debug, Error)]
/// Errors that could occur while deserializeing a integer
pub enum DeserializeIntError {
    #[error("Could not convert the value to UTF-8")]
    ConvertionToString(
        #[source]
        #[from]
        std::string::FromUtf8Error,
    ),
    #[error("Could not parse the value as a integer: {0}")]
    ConvertionToInt(
        #[source]
        #[from]
        std::num::ParseIntError,
    ),
}

#[derive(Debug, Error)]
/// Errors that could occur while deserializeing a float
pub enum DeserializeFloatError {
    #[error("Could not convert the value to UTF-8: {0}")]
    ConvertionToString(
        #[source]
        #[from]
        std::string::FromUtf8Error,
    ),
    #[error("Could not parse the value as float: {0}")]
    ConvertionToFloat(
        #[source]
        #[from]
        std::num::ParseFloatError,
    ),
}

#[derive(Debug, Error)]
/// Errors that could occur while deserializeing a boolean
pub enum DeserializeBoolError {
    #[error("Could not convert the value to UTF-8: {0}")]
    ConvertionToString(
        #[source]
        #[from]
        std::string::FromUtf8Error,
    ),
    #[error("Could not parse the value as boolean: {0}")]
    ConvertionToFloat(
        #[source]
        #[from]
        std::str::ParseBoolError,
    ),
}

#[derive(Debug, Error)]
/// Errors that could occur while deserializeing a char
pub enum DeserializeCharError {
    #[error("Could not convert the value to UTF-8: {0}")]
    ConvertionToString(
        #[source]
        #[from]
        std::string::FromUtf8Error,
    ),
    #[error("Could not parse the value as character: {0}")]
    ConvertionToChar(
        #[source]
        #[from]
        std::char::ParseCharError,
    ),
}

macro_rules! impl_stringly_types {
    ( $( ( $type:ty, $error:ty ) ),* $(,)? ) => {
        $(
            impl FromMultiPartPart<'_> for $type {
                type Error = $error;

                fn from_bytes(body: &'_ [u8]) -> Result<Self, Self::Error> {
                    let value = String::from_bytes(body)?.parse()?;
                    Ok(value)
                }
            }
        )*
    };
}

impl_stringly_types! {
    (bool, DeserializeBoolError),
    (char, DeserializeCharError),
    (f32, DeserializeFloatError),
    (f64, DeserializeFloatError),
    (u8, DeserializeIntError),
    (u16, DeserializeIntError),
    (u32, DeserializeIntError),
    (u64, DeserializeIntError),
    (u128, DeserializeIntError),
    (i8, DeserializeIntError),
    (i16, DeserializeIntError),
    (i32, DeserializeIntError),
    (i64, DeserializeIntError),
    (i128, DeserializeIntError),
}

impl<'a> FromMultiPartPart<'a> for &'a [u8] {
    type Error = std::convert::Infallible;

    fn from_bytes(body: &'a [u8]) -> Result<Self, Self::Error> {
        Ok(body)
    }
}

impl FromMultiPartPart<'_> for Vec<u8> {
    type Error = std::convert::Infallible;

    fn from_bytes(body: &[u8]) -> Result<Self, Self::Error> {
        Ok(body.to_vec())
    }
}

impl<'a, T: FromMultiPartPart<'a>> FromMultiPartPart<'a> for Option<T> {
    type Error = T::Error;

    fn from_bytes(body: &'a [u8]) -> Result<Self, Self::Error> {
        T::from_bytes(body).map(Some)
    }

    fn handle_absent_value(_key: &str) -> Result<Self, poem::Error> {
        Ok(None)
    }
}

#[cfg(feature = "bytes")]
impl FromMultiPartPart<'_> for bytes::Bytes {
    type Error = std::convert::Infallible;

    fn from_bytes(body: &[u8]) -> Result<Self, Self::Error> {
        Ok(bytes::Bytes::copy_from_slice(body))
    }
}

#[cfg(feature = "json")]
#[derive(Debug)]
/// Newtype around a value with [`serde::Deserialize`] used to deserialize json values
pub struct Json<T>(pub T);

#[cfg(feature = "json")]
impl<'de, 'a: 'de, T: serde::Deserialize<'de>> FromMultiPartPart<'a> for Json<T> {
    type Error = serde_json::Error;

    fn from_bytes(body: &'a [u8]) -> Result<Json<T>, Self::Error> {
        let data = serde_json::from_slice(body)?;
        Ok(Json(data))
    }
}
