//! Utility map used to build the request object

use std::collections::HashMap;

use poem::http::StatusCode;
use thiserror::Error;

use crate::part::FromMultiPartPart;

/// Error that could occur when getting data from a [`MultiPartMap`]
#[derive(Debug, Error)]
pub enum MultiPartMapError<E> {
    #[error("Failed to decode field `{0}`: {1:}")]
    DecodeError(String, #[source] E),
}

impl<E> poem::error::ResponseError for MultiPartMapError<E> {
    fn status(&self) -> poem::http::StatusCode {
        StatusCode::BAD_REQUEST
    }
}

/// A map containing a parsed multipart request
#[derive(Debug)]
pub struct MultiPartMap {
    map: HashMap<String, Vec<u8>>,
}

impl MultiPartMap {
    /// Create a new [`MultiPartMap`]
    pub(crate) async fn new(
        mut body: poem::web::Multipart,
    ) -> Result<Self, poem::error::ParseMultipartError> {
        let mut map = HashMap::new();

        while let Ok(Some(field)) = body.next_field().await {
            let name = match field.name() {
                Some(name) => name.to_string(),
                None => continue,
            };

            let value = field.bytes().await?;

            map.insert(name, value);
        }

        Ok(Self { map })
    }

    /// Get the value behind the key in the multipart map. Returns a [`MultiPartMapError`] is the
    /// key could not be found in the map, or the value in behind the key could not be decoded.
    pub fn get<'a, S: FromMultiPartPart<'a>>(&'a self, key: &str) -> Result<S, poem::Error> {
        let value = if let Some(value) = self.map.get(key) {
            S::from_bytes(value).map_err(poem::error::BadRequest)?
        } else {
            S::handle_absent_value(key)?
        };

        Ok(value)
    }
}
