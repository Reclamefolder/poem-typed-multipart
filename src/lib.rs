//! This crate provides a typed extractor for multipart/form-data requests in [`poem`], in a
//! rust-native manner.
//!
//! # Usage
//!
//! To use this crate, either implement [`FromMultiPart`] for your structure, or use the
//! [`poem_typed_multipart_macro::FromMultiPart`] macro. Then use the [`TypedMultiPart`] extractor
//! in your endpoint.
//!
//! ```
//! use poem_typed_multipart::{TypedMultiPart, FromMultiPart}
//!
//! #[derive(Debug, FromMultiPart)]
//! struct Body {
//!     id: u32,
//!     title: String
//! }
//!
//! #[poem::handler]
//! fn hello(TypedMultiPart(body): TypedMultiPart<Body>) -> String {
//!     println!("{body:?}");
//!     format!("Hello {}", body.title)
//! }
//! ```
//!
//! ## Extending this crate to your types
//!
//! All types implementing [`part::FromMultiPartPart`] can be used to can be extracted using this
//! crate. Check the existing implementation on details how to implement these yourself.
//!
//! # Features
//!
//! - `json`: Extract json values using the [`part::Json`] type.
//! - `bytes`: Support for [`bytes::Bytes`].

use map::MultiPartMap;
use poem::FromRequest;

#[cfg(feature = "derive")]
pub use poem_typed_multipart_macro::FromMultiPart;

pub mod map;
pub mod part;

/// Trait implement indicating that the implement can be parsed from a multipart request
pub trait FromMultiPart
where
    Self: Sized,
{
    /// Try to get the implementer based on the provided multipart request
    fn decode(map: MultiPartMap) -> Result<Self, poem::Error>;
}

/// Extractor used to get value `T` from the request. This consumes the request
pub struct TypedMultiPart<T: FromMultiPart>(pub T);

impl<'a, T: FromMultiPart> FromRequest<'a> for TypedMultiPart<T> {
    async fn from_request(
        req: &'a poem::Request,
        body: &mut poem::RequestBody,
    ) -> Result<Self, poem::Error> {
        let body = poem::web::Multipart::from_request(req, body).await?;
        let map = MultiPartMap::new(body).await?;

        let value = T::decode(map)?;

        Ok(Self(value))
    }
}
