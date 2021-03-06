use std::convert::TryFrom;
use std::ops::Deref;

use header::{HeaderName, HeaderValue, InvalidTypedHeader, TypedHeader};
use syntax::trim_whitespace_left;

pub const MAX_CONTENT_LENGTH: u64 = 9_999_999_999_999_999_999;

/// The `Content-Length` typed header as described by
/// [RFC7826](https://tools.ietf.org/html/rfc7826#section-18.17).
///
/// The RFC states that the content length of a request/response can be up to 19 digits long which
/// actually would require use of u64, but since the length of the buffer during decoding is of
/// type `usize`, this cannot be guaranteed on all platforms.
///
/// The default value for this header is 0.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ContentLength(usize);

impl Deref for ContentLength {
    type Target = usize;

    fn deref(&self) -> &usize {
        &self.0
    }
}

impl TryFrom<usize> for ContentLength {
    type Error = InvalidTypedHeader;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        if value as u64 > MAX_CONTENT_LENGTH {
            Err(InvalidTypedHeader)
        } else {
            Ok(ContentLength(value))
        }
    }
}

impl TypedHeader for ContentLength {
    /// Returns the statically assigned `HeaderName` for this header.
    fn header_name() -> &'static HeaderName {
        &HeaderName::ContentLength
    }

    /// Converts the `ContentLength` type to raw header values.
    ///
    /// # Examples
    ///
    /// ```
    /// # #![feature(try_from)]
    /// #
    /// use std::convert::TryFrom;
    ///
    /// use rtsp::*;
    /// use rtsp::header::types::ContentLength;
    ///
    /// let typed_header = ContentLength::try_from(10).unwrap();
    /// let raw_header = vec![HeaderValue::try_from("10").unwrap()];
    /// assert_eq!(typed_header.to_header_raw(), raw_header);
    /// ```
    fn to_header_raw(&self) -> Vec<HeaderValue> {
        // Unsafe Justification
        //
        // In order for this to be safe, we must ensure that `value` contains no unprintable
        // ASCII-US characters and that all linebreaks of the form `"\r\n"` are followed by a space
        // or tab. Since [`ContentLength`] serializes into a number, it satisfies the constraints.

        vec![unsafe { HeaderValue::from_str_unchecked(self.0.to_string().as_str()) }]
    }

    /// Converts the raw header values to the `ContentLength` header type. Based on the syntax
    /// provided by [RFC7826](https://tools.ietf.org/html/rfc7826#section-20), this header has the
    /// following syntax:
    ///
    /// ```text
    /// DIGIT = %x30-39 ; any US-ASCII digit "0".."9"
    /// Content-Length = "Content-Length" HCOLON 1*19DIGIT
    /// ```
    ///
    /// However, in order to allow 19 digits to be used, 64 bit integers need to be used
    /// (i.e. `u64`), but the length of the buffer during decoding is of type `usize`, so the
    /// actual allowed length may be significantly smaller depending on the architecture.
    ///
    /// The absence of a header value defaults the content length to a value of 0.
    ///
    /// # Examples
    ///
    /// ```
    /// # #![feature(try_from)]
    /// #
    /// use std::convert::TryFrom;
    ///
    /// use rtsp::*;
    /// use rtsp::header::types::ContentLength;
    ///
    /// let typed_header = ContentLength::try_from(0).unwrap();
    /// let raw_header: Vec<HeaderValue> = vec![];
    ///
    /// assert_eq!(
    ///     ContentLength::try_from_header_raw(&raw_header).unwrap(),
    ///     typed_header
    /// );
    ///
    /// let typed_header = ContentLength::try_from(10).unwrap();
    /// let raw_header = vec![HeaderValue::try_from("10").unwrap()];
    ///
    /// assert_eq!(
    ///     ContentLength::try_from_header_raw(&raw_header).unwrap(),
    ///     typed_header
    /// );
    ///
    /// let raw_header = vec![HeaderValue::try_from("invalid content length").unwrap()];
    ///
    /// assert!(ContentLength::try_from_header_raw(&raw_header).is_err());
    /// ```
    fn try_from_header_raw(header: &[HeaderValue]) -> Result<Self, InvalidTypedHeader> {
        if header.len() == 0 {
            Ok(ContentLength::default())
        } else if header.len() > 1 {
            Err(InvalidTypedHeader)
        } else {
            trim_whitespace_left(header[0].as_str())
                .parse::<usize>()
                .map_err(|_| InvalidTypedHeader)
                .and_then(|content_length| ContentLength::try_from(content_length))
        }
    }
}
