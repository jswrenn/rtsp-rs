//! RTSP Header Name

use ascii::AsciiString;
use std::convert::TryFrom;
use std::error::Error;
use std::fmt;
use std::hash::{Hash, Hasher};

use syntax::is_token;

macro_rules! standard_headers {
    (
        $(
            $(#[$docs:meta])*
            ($variant:ident, $name:expr, $canonical_name:expr);
        )+
    ) => {
        /// An RTSP header name.
        ///
        /// All standardized header names are supported with an ASCII encoded extension that is
        /// always lowercase.
        #[derive(Clone, Eq, Hash, PartialEq)]
        #[non_exhaustive]
        pub enum HeaderName {
        $(
            $(#[$docs])*
            $variant,
        )+

            /// A header name that is not one of the standardized header names. This is encoded
            /// using ASCII-US and is always lowercase.
            Extension(ExtensionHeaderName)
        }

        impl HeaderName {
            pub fn as_str(&self) -> &str {
                use self::HeaderName::*;

                match *self {
                $(
                    $variant => $name,
                )+
                    Extension(ref extension) => extension.as_str()
                }
            }

            pub fn canonical_name(&self) -> &str {
                use self::HeaderName::*;

                match *self {
                $(
                    $variant => $canonical_name,
                )+
                    Extension(ref extension) => extension.canonical_name()
                }
            }
        }

        #[test]
        fn test_standard_header_as_str() {
        $(
            let header_name = HeaderName::$variant;
            assert_eq!(header_name.as_str(), $name);
        )+
        }

        #[test]
        fn test_standard_header_canonical_name() {
        $(
            let header_name = HeaderName::$variant;
            assert_eq!(header_name.canonical_name(), $canonical_name);
        )+
        }

        #[test]
        fn test_standard_header_name_equality() {
        $(
            let header_name = HeaderName::$variant;
            assert_eq!(header_name.as_str(), header_name.canonical_name().to_lowercase().as_str());
        )+
        }
    }
}

impl HeaderName {
    /// A helper function that creates a new `HeaderName` instance with the given header name
    /// extension. It first checks to see if the header name is valid, and if not, it will return an
    /// error.
    ///
    /// It is assumed that `name_lower` is the lowercase equivalent of `name`. Breaking this
    /// constraint can lead to invalid header names.
    fn extension(name: &[u8], name_lower: &[u8]) -> Result<HeaderName, InvalidHeaderName> {
        if !is_token(name) {
            return Err(InvalidHeaderName);
        }

        debug_assert_eq!(name.to_ascii_lowercase().as_slice(), name_lower);
        debug_assert!(is_token(name_lower));

        // Unsafe Justification
        //
        // We need to make sure that `name` and `name_lower` are valid ASCII-US strings. The
        // [`syntax::is_token`] function ensures that it is a proper subset, but it is a constraint
        // of this function that `name_lower` *must* be the lowercase equivalent of `name`.

        let name = unsafe { AsciiString::from_ascii_unchecked(name) };
        let name_lower = unsafe { AsciiString::from_ascii_unchecked(name_lower) };
        Ok(HeaderName::Extension(ExtensionHeaderName(name, name_lower)))
    }
}

impl AsRef<[u8]> for HeaderName {
    fn as_ref(&self) -> &[u8] {
        self.as_str().as_bytes()
    }
}

impl AsRef<str> for HeaderName {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Debug for HeaderName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.canonical_name())
    }
}

impl fmt::Display for HeaderName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.canonical_name())
    }
}

/// Performs equality checking of a `HeaderName` with a `str`. This check is case insensitive.
///
/// # Examples
///
/// ```
/// # #![feature(try_from)]
/// #
/// use std::convert::TryFrom;
///
/// use rtsp::HeaderName;
///
/// assert_eq!(HeaderName::try_from("eXtEnSiOn").unwrap(), *"exTENSION");
/// ```
impl PartialEq<str> for HeaderName {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other.to_ascii_lowercase()
    }
}

impl PartialEq<HeaderName> for str {
    fn eq(&self, other: &HeaderName) -> bool {
        self.to_ascii_lowercase() == other.as_str()
    }
}

/// Performs equality checking of a `HeaderName` with a `&str`. This check is case insensitive.
///
/// # Examples
///
/// ```
/// # #![feature(try_from)]
/// #
/// use std::convert::TryFrom;
///
/// use rtsp::HeaderName;
///
/// assert_eq!(HeaderName::try_from("extension").unwrap(), "extension");
/// ```
impl<'a> PartialEq<&'a str> for HeaderName {
    fn eq(&self, other: &&'a str) -> bool {
        self.as_str() == (*other).to_ascii_lowercase()
    }
}

impl<'a> PartialEq<HeaderName> for &'a str {
    fn eq(&self, other: &HeaderName) -> bool {
        self.to_ascii_lowercase() == other.as_str()
    }
}

impl PartialEq<String> for HeaderName {
    fn eq(&self, other: &String) -> bool {
        self.as_str() == other.as_str()
    }
}

impl PartialEq<HeaderName> for String {
    fn eq(&self, other: &HeaderName) -> bool {
        self.as_str() == other.as_str()
    }
}

impl<'a> PartialEq<&'a HeaderName> for HeaderName {
    fn eq(&self, other: &&'a HeaderName) -> bool {
        *self == **other
    }
}

impl<'a> PartialEq<HeaderName> for &'a HeaderName {
    fn eq(&self, other: &HeaderName) -> bool {
        *other == *self
    }
}

impl<'a> From<&'a HeaderName> for HeaderName {
    fn from(src: &'a HeaderName) -> HeaderName {
        src.clone()
    }
}

/// Provides a fallible conversion from a byte slice to a `HeaderName`. Note that you cannot do the
/// following:
///
/// ```compile_fail
/// let allow = HeaderName::try_from(b"Allow").unwrap();
/// ```
///
/// This is because `b"Allow"` is of type `&[u8; 5]` and so it must be converted to `&[u8]` in order
/// to perform the conversion. Another `TryFrom` implementation from `&[u8, N: usize]` will be
/// provided once constant generics land on nightly.
impl<'a> TryFrom<&'a [u8]> for HeaderName {
    type Error = InvalidHeaderName;

    /// Converts a `&[u8]` to an RTSP header name. The header name must contain only valid token
    /// characters. Since valid token characters includes only a subset of the ASCII-US character
    /// set, care should be taken when converting a UTF-8 encoded string to a header name.
    ///
    /// The conversion is case insensitive, but the header name is converted to lowercase.
    ///
    /// # Examples
    ///
    /// ```
    /// # #![feature(try_from)]
    /// #
    /// use std::convert::TryFrom;
    ///
    /// use rtsp::HeaderName;
    ///
    /// let content_length = HeaderName::try_from(&b"Content-Length"[..]).unwrap();
    /// assert_eq!(content_length, HeaderName::ContentLength);
    ///
    /// let referrer = HeaderName::try_from(&b"ReFeRrEr"[..]).unwrap();
    /// assert_eq!(referrer, HeaderName::Referrer);
    ///
    /// let extension = HeaderName::try_from(&b"Ext"[..]).unwrap();
    /// assert_eq!(extension.as_str(), "ext");
    ///
    /// let error = HeaderName::try_from(&b""[..]);
    /// assert!(error.is_err());
    /// ```
    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        use self::HeaderName::*;

        if value.is_empty() || value.len() > super::MAX_HEADER_NAME_LENGTH {
            return Err(InvalidHeaderName);
        }

        let value_lower = value.to_ascii_lowercase();

        match value_lower.len() {
            3 => match value_lower.as_slice() {
                b"via" => Ok(Via),
                _ => HeaderName::extension(value, value_lower.as_slice()),
            },
            4 => match value_lower.as_slice() {
                b"cseq" => Ok(CSeq),
                b"date" => Ok(Date),
                b"from" => Ok(From),
                b"mtag" => Ok(MTag),
                _ => HeaderName::extension(value, value_lower.as_slice()),
            },
            5 => match value_lower.as_slice() {
                b"allow" => Ok(Allow),
                b"range" => Ok(Range),
                b"scale" => Ok(Scale),
                b"speed" => Ok(Speed),
                _ => HeaderName::extension(value, value_lower.as_slice()),
            },
            6 => match value_lower.as_slice() {
                b"accept" => Ok(Accept),
                b"public" => Ok(Public),
                b"server" => Ok(Server),
                _ => HeaderName::extension(value, value_lower.as_slice()),
            },
            7 => match value_lower.as_slice() {
                b"expires" => Ok(Expires),
                b"require" => Ok(Require),
                b"session" => Ok(Session),
                _ => HeaderName::extension(value, value_lower.as_slice()),
            },
            8 => match value_lower.as_slice() {
                b"if-match" => Ok(IfMatch),
                b"location" => Ok(Location),
                b"referrer" => Ok(Referrer),
                b"rtp-info" => Ok(RTPInfo),
                _ => HeaderName::extension(value, value_lower.as_slice()),
            },
            9 => match value_lower.as_slice() {
                b"bandwidth" => Ok(Bandwidth),
                b"blocksize" => Ok(Blocksize),
                b"supported" => Ok(Supported),
                b"timestamp" => Ok(Timestamp),
                b"transport" => Ok(Transport),
                _ => HeaderName::extension(value, value_lower.as_slice()),
            },
            10 => match value_lower.as_slice() {
                b"connection" => Ok(Connection),
                b"seek-style" => Ok(SeekStyle),
                b"user-agent" => Ok(UserAgent),
                _ => HeaderName::extension(value, value_lower.as_slice()),
            },
            11 => match value_lower.as_slice() {
                b"media-range" => Ok(MediaRange),
                b"retry-after" => Ok(RetryAfter),
                b"unsupported" => Ok(Unsupported),
                _ => HeaderName::extension(value, value_lower.as_slice()),
            },
            12 => match value_lower.as_slice() {
                b"content-base" => Ok(ContentBase),
                b"content-type" => Ok(ContentType),
                _ => HeaderName::extension(value, value_lower.as_slice()),
            },
            13 => match value_lower.as_slice() {
                b"accept-ranges" => Ok(AcceptRanges),
                b"authorization" => Ok(Authorization),
                b"cache-control" => Ok(CacheControl),
                b"if-none-match" => Ok(IfNoneMatch),
                b"last-modified" => Ok(LastModified),
                b"notify-reason" => Ok(NotifyReason),
                b"proxy-require" => Ok(ProxyRequire),
                _ => HeaderName::extension(value, value_lower.as_slice()),
            },
            14 => match value_lower.as_slice() {
                b"content-length" => Ok(ContentLength),
                b"request-status" => Ok(RequestStatus),
                _ => HeaderName::extension(value, value_lower.as_slice()),
            },
            15 => match value_lower.as_slice() {
                b"accept-encoding" => Ok(AcceptEncoding),
                b"accept-language" => Ok(AcceptLanguage),
                b"proxy-supported" => Ok(ProxySupported),
                _ => HeaderName::extension(value, value_lower.as_slice()),
            },
            16 => match value_lower.as_slice() {
                b"content-encoding" => Ok(ContentEncoding),
                b"content-language" => Ok(ContentLanguage),
                b"content-location" => Ok(ContentLocation),
                b"media-properties" => Ok(MediaProperties),
                b"terminate-reason" => Ok(TerminateReason),
                b"www-authenticate" => Ok(WWWAuthenticate),
                _ => HeaderName::extension(value, value_lower.as_slice()),
            },
            17 => match value_lower.as_slice() {
                b"if-modified-since" => Ok(IfModifiedSince),
                _ => HeaderName::extension(value, value_lower.as_slice()),
            },
            18 => match value_lower.as_slice() {
                b"accept-credentials" => Ok(AcceptCredentials),
                b"pipelined-requests" => Ok(PipelinedRequests),
                b"proxy-authenticate" => Ok(ProxyAuthenticate),
                _ => HeaderName::extension(value, value_lower.as_slice()),
            },
            19 => match value_lower.as_slice() {
                b"authentication-info" => Ok(AuthenticationInfo),
                b"proxy-authorization" => Ok(ProxyAuthorization),
                _ => HeaderName::extension(value, value_lower.as_slice()),
            },
            22 => match value_lower.as_slice() {
                b"connection-credentials" => Ok(ConnectionCredentials),
                _ => HeaderName::extension(value, value_lower.as_slice()),
            },
            25 => match value_lower.as_slice() {
                b"proxy-authentication-info" => Ok(ProxyAuthenticationInfo),
                _ => HeaderName::extension(value, value_lower.as_slice()),
            },
            _ => HeaderName::extension(value, value_lower.as_slice()),
        }
    }
}

impl<'a> TryFrom<&'a str> for HeaderName {
    type Error = InvalidHeaderName;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        HeaderName::try_from(value.as_bytes())
    }
}

/// A wrapper type used to avoid users creating extension header names that are actually
/// standardized header names.
#[derive(Clone, Eq, PartialEq)]
pub struct ExtensionHeaderName(AsciiString, AsciiString);

impl fmt::Debug for ExtensionHeaderName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Display for ExtensionHeaderName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Hash for ExtensionHeaderName {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.1.hash(state);
    }
}

/// Performs equality checking of a `ExtensionHeaderName` with a `str`. This check is case
/// insensitive.
///
/// # Examples
///
/// ```
/// # #![feature(try_from)]
/// #
/// use std::convert::TryFrom;
///
/// use rtsp::HeaderName;
///
/// match HeaderName::try_from("extension").unwrap() {
///     HeaderName::Extension(extension) => assert_eq!(extension, *"eXtEnSiOn"),
///     _ => panic!("expected extension header name")
/// }
/// ```
impl PartialEq<str> for ExtensionHeaderName {
    fn eq(&self, other: &str) -> bool {
        self.1 == other.to_ascii_lowercase()
    }
}

/// Performs equality checking of a `ExtensionHeaderName` with a `&str`. This check is case
/// insensitive.
///
/// # Examples
///
/// ```
/// # #![feature(try_from)]
/// #
/// use std::convert::TryFrom;
///
/// use rtsp::HeaderName;
///
/// match HeaderName::try_from("extension").unwrap() {
///     HeaderName::Extension(extension) => assert_eq!(extension, "eXtEnSiOn"),
///     _ => panic!("expected extension header name")
/// }
/// ```
impl<'a> PartialEq<&'a str> for ExtensionHeaderName {
    fn eq(&self, other: &&'a str) -> bool {
        self.1 == (*other).to_ascii_lowercase()
    }
}

impl ExtensionHeaderName {
    /// Returns a `&str` representation of the extension header name. The returned string is
    /// lowercase even if the extension header name originally was a non-lowercase header name.
    ///
    /// # Examples
    ///
    /// ```
    /// # #![feature(try_from)]
    /// #
    /// use std::convert::TryFrom;
    ///
    /// use rtsp::HeaderName;
    ///
    /// match HeaderName::try_from("ExTeNsIoN").unwrap() {
    ///     HeaderName::Extension(extension) => assert_eq!(extension.as_str(), "extension"),
    ///     _ => panic!("expected extension header name")
    /// }
    /// ```
    pub fn as_str(&self) -> &str {
        self.1.as_str()
    }

    /// Returns a `&str` representation of the extension header name. The returned string is
    /// is of the same case as when it was created.
    ///
    /// # Examples
    ///
    /// ```
    /// # #![feature(try_from)]
    /// #
    /// use std::convert::TryFrom;
    ///
    /// use rtsp::HeaderName;
    ///
    /// match HeaderName::try_from("ExTeNsIoN").unwrap() {
    ///     HeaderName::Extension(extension) => assert_eq!(extension.canonical_name(), "ExTeNsIoN"),
    ///     _ => panic!("expected extension header name")
    /// }
    /// ```
    pub fn canonical_name(&self) -> &str {
        self.0.as_str()
    }
}

/// A possible error value when converting to a `HeaderName` from a `&[u8]` or `&str`.
///
/// This error indicates that the header name was of size 0 or contained invalid token characters.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct InvalidHeaderName;

impl fmt::Display for InvalidHeaderName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

impl Error for InvalidHeaderName {
    fn description(&self) -> &str {
        "invalid RTSP header name"
    }
}

standard_headers! {
    /// Accept
    /// [[RFC7826, Section 18.1](https://tools.ietf.org/html/rfc7826#section-18.1)]
    (Accept, "accept", "Accept");

    /// Accept-Credentials
    /// [[RFC7826, Section 18.2](https://tools.ietf.org/html/rfc7826#section-18.2)]
    (AcceptCredentials, "accept-credentials", "Accept-Credentials");

    /// Accept-Encoding
    /// [[RFC7826, Section 18.3](https://tools.ietf.org/html/rfc7826#section-18.3)]
    (AcceptEncoding, "accept-encoding", "Accept-Encoding");

    /// Accept-Language
    /// [[RFC7826, Section 18.4](https://tools.ietf.org/html/rfc7826#section-18.4)]
    (AcceptLanguage, "accept-language", "Accept-Language");

    /// Accept-Ranges
    /// [[RFC7826, Section 18.5](https://tools.ietf.org/html/rfc7826#section-18.5)]
    (AcceptRanges, "accept-ranges", "Accept-Ranges");

    /// Allow
    /// [[RFC7826, Section 18.6](https://tools.ietf.org/html/rfc7826#section-18.6)]
    (Allow, "allow", "Allow");

    /// Authentication-Info
    /// [[RFC7826, Section 18.7](https://tools.ietf.org/html/rfc7826#section-18.7)]
    (AuthenticationInfo, "authentication-info", "Authentication-Info");

    /// Authorization
    /// [[RFC7826, Section 18.8](https://tools.ietf.org/html/rfc7826#section-18.8)]
    (Authorization, "authorization", "Authorization");

    /// Bandwidth
    /// [[RFC7826, Section 18.9](https://tools.ietf.org/html/rfc7826#section-18.9)]
    (Bandwidth, "bandwidth", "Bandwidth");

    /// Blocksize
    /// [[RFC7826, Section 18.10](https://tools.ietf.org/html/rfc7826#section-18.10)]
    (Blocksize, "blocksize", "Blocksize");

    /// Cache-Control
    /// [[RFC7826, Section 18.11](https://tools.ietf.org/html/rfc7826#section-18.11)]
    (CacheControl, "cache-control", "Cache-Control");

    /// Connection
    /// [[RFC7826, Section 18.12](https://tools.ietf.org/html/rfc7826#section-18.12)]
    (Connection, "connection", "Connection");

    /// Connection-Credentials
    /// [[RFC7826, Section 18.13](https://tools.ietf.org/html/rfc7826#section-18.13)]
    (ConnectionCredentials, "connection-credentials", "Connection-Credentials");

    /// Content-Base
    /// [[RFC7826, Section 18.14](https://tools.ietf.org/html/rfc7826#section-18.14)]
    (ContentBase, "content-base", "Content-Base");

    /// Content-Encoding
    /// [[RFC7826, Section 18.15](https://tools.ietf.org/html/rfc7826#section-18.15)]
    (ContentEncoding, "content-encoding", "Content-Encoding");

    /// Content-Language
    /// [[RFC7826, Section 18.16](https://tools.ietf.org/html/rfc7826#section-18.16)]
    (ContentLanguage, "content-language", "Content-Language");

    /// Content-Length
    /// [[RFC7826, Section 18.17](https://tools.ietf.org/html/rfc7826#section-18.17)]
    (ContentLength, "content-length", "Content-Length");

    /// Content-Location
    /// [[RFC7826, Section 18.18](https://tools.ietf.org/html/rfc7826#section-18.18)]
    (ContentLocation, "content-location", "Content-Location");

    /// Content-Type
    /// [[RFC7826, Section 18.19](https://tools.ietf.org/html/rfc7826#section-18.19)]
    (ContentType, "content-type", "Content-Type");

    /// CSeq
    /// [[RFC7826, Section 18.20](https://tools.ietf.org/html/rfc7826#section-18.20)]
    (CSeq, "cseq", "CSeq");

    /// Date
    /// [[RFC7826, Section 18.21](https://tools.ietf.org/html/rfc7826#section-18.21)]
    (Date, "date", "Date");

    /// Expires
    /// [[RFC7826, Section 18.22](https://tools.ietf.org/html/rfc7826#section-18.22)]
    (Expires, "expires", "Expires");

    /// Date
    /// [[RFC7826, Section 18.23](https://tools.ietf.org/html/rfc7826#section-18.23)]
    (From, "from", "From");

    /// If-Match
    /// [[RFC7826, Section 18.24](https://tools.ietf.org/html/rfc7826#section-18.24)]
    (IfMatch, "if-match", "If-Match");

    /// If-Modified-Since
    /// [[RFC7826, Section 18.25](https://tools.ietf.org/html/rfc7826#section-18.25)]
    (IfModifiedSince, "if-modified-since", "If-Modified-Since");

    /// If-None-Match
    /// [[RFC7826, Section 18.26](https://tools.ietf.org/html/rfc7826#section-18.26)]
    (IfNoneMatch, "if-none-match", "If-None-Match");

    /// Last-Modified
    /// [[RFC7826, Section 18.27](https://tools.ietf.org/html/rfc7826#section-18.27)]
    (LastModified, "last-modified", "Last-Modified");

    /// Location
    /// [[RFC7826, Section 18.28](https://tools.ietf.org/html/rfc7826#section-18.28)]
    (Location, "location", "Location");

    /// Media-Properties
    /// [[RFC7826, Section 18.29](https://tools.ietf.org/html/rfc7826#section-18.29)]
    (MediaProperties, "media-properties", "Media-Properties");

    /// Media-Range
    /// [[RFC7826, Section 18.30](https://tools.ietf.org/html/rfc7826#section-18.30)]
    (MediaRange, "media-range", "Media-Range");

    /// MTag
    /// [[RFC7826, Section 18.31](https://tools.ietf.org/html/rfc7826#section-18.31)]
    (MTag, "mtag", "MTag");

    /// Notify-Reason
    /// [[RFC7826, Section 18.32](https://tools.ietf.org/html/rfc7826#section-18.32)]
    (NotifyReason, "notify-reason", "Notify-Reason");

    /// Pipelined-Requests
    /// [[RFC7826, Section 18.33](https://tools.ietf.org/html/rfc7826#section-18.33)]
    (PipelinedRequests, "pipelined-requests", "Pipelined-Requests");

    /// Proxy-Authenticate
    /// [[RFC7826, Section 18.34](https://tools.ietf.org/html/rfc7826#section-18.34)]
    (ProxyAuthenticate, "proxy-authenticate", "Proxy-Authenticate");

    /// Proxy-Authentication-Info
    /// [[RFC7826, Section 18.35](https://tools.ietf.org/html/rfc7826#section-18.35)]
    (ProxyAuthenticationInfo, "proxy-authentication-info", "Proxy-Authentication-Info");

    /// Proxy-Authorization
    /// [[RFC7826, Section 18.36](https://tools.ietf.org/html/rfc7826#section-18.36)]
    (ProxyAuthorization, "proxy-authorization", "Proxy-Authorization");

    /// Proxy-Require
    /// [[RFC7826, Section 18.37](https://tools.ietf.org/html/rfc7826#section-18.37)]
    (ProxyRequire, "proxy-require", "Proxy-Require");

    /// Proxy-Supported
    /// [[RFC7826, Section 18.38](https://tools.ietf.org/html/rfc7826#section-18.38)]
    (ProxySupported, "proxy-supported", "Proxy-Supported");

    /// Public
    /// [[RFC7826, Section 18.39](https://tools.ietf.org/html/rfc7826#section-18.39)]
    (Public, "public", "Public");

    /// Range
    /// [[RFC7826, Section 18.40](https://tools.ietf.org/html/rfc7826#section-18.40)]
    (Range, "range", "Range");

    /// Referrer
    /// [[RFC7826, Section 18.41](https://tools.ietf.org/html/rfc7826#section-18.41)]
    (Referrer, "referrer", "Referrer");

    /// Request-Status
    /// [[RFC7826, Section 18.42](https://tools.ietf.org/html/rfc7826#section-18.42)]
    (RequestStatus, "request-status", "Request-Status");

    /// Require
    /// [[RFC7826, Section 18.43](https://tools.ietf.org/html/rfc7826#section-18.43)]
    (Require, "require", "Require");

    /// Retry-After
    /// [[RFC7826, Section 18.44](https://tools.ietf.org/html/rfc7826#section-18.44)]
    (RetryAfter, "retry-after", "Retry-After");

    /// RTP-Info
    /// [[RFC7826, Section 18.45](https://tools.ietf.org/html/rfc7826#section-18.45)]
    (RTPInfo, "rtp-info", "RTP-Info");

    /// Scale
    /// [[RFC7826, Section 18.46](https://tools.ietf.org/html/rfc7826#section-18.46)]
    (Scale, "scale", "Scale");

    /// Seek-Style
    /// [[RFC7826, Section 18.47](https://tools.ietf.org/html/rfc7826#section-18.47)]
    (SeekStyle, "seek-style", "Seek-Style");

    /// Server
    /// [[RFC7826, Section 18.48](https://tools.ietf.org/html/rfc7826#section-18.48)]
    (Server, "server", "Server");

    /// Session
    /// [[RFC7826, Section 18.49](https://tools.ietf.org/html/rfc7826#section-18.49)]
    (Session, "session", "Session");

    /// Speed
    /// [[RFC7826, Section 18.50](https://tools.ietf.org/html/rfc7826#section-18.50)]
    (Speed, "speed", "Speed");

    /// Supported
    /// [[RFC7826, Section 18.51](https://tools.ietf.org/html/rfc7826#section-18.51)]
    (Supported, "supported", "Supported");

    /// Terminate-Reason
    /// [[RFC7826, Section 18.52](https://tools.ietf.org/html/rfc7826#section-18.52)]
    (TerminateReason, "terminate-reason", "Terminate-Reason");

    /// Timestamp
    /// [[RFC7826, Section 18.53](https://tools.ietf.org/html/rfc7826#section-18.53)]
    (Timestamp, "timestamp", "Timestamp");

    /// Transport
    /// [[RFC7826, Section 18.54](https://tools.ietf.org/html/rfc7826#section-18.54)]
    (Transport, "transport", "Transport");

    /// Unsupported
    /// [[RFC7826, Section 18.55](https://tools.ietf.org/html/rfc7826#section-18.55)]
    (Unsupported, "unsupported", "Unsupported");

    /// User-Agent
    /// [[RFC7826, Section 18.56](https://tools.ietf.org/html/rfc7826#section-18.56)]
    (UserAgent, "user-agent", "User-Agent");

    /// Via
    /// [[RFC7826, Section 18.57](https://tools.ietf.org/html/rfc7826#section-18.57)]
    (Via, "via", "Via");

    /// WWW-Authenticate
    /// [[RFC7826, Section 18.58](https://tools.ietf.org/html/rfc7826#section-18.58)]
    (WWWAuthenticate, "www-authenticate", "WWW-Authenticate");
}
