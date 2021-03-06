//! RTSP Request Types
//!
//! This module contains structs related to RTSP requests, notably the `Request` type itself as well
//! as a builder to create requests. Typically, you will import the `rtsp::Request` type rather than
//! reaching into this module itself.

use std::convert::TryFrom;
use std::error::Error;
use std::fmt;
use std::mem::replace;

use header::{HeaderMap, HeaderName, HeaderValue, TypedHeader, TypedHeaderMap};
use method::Method;
use uri::RequestURIField;
use version::Version;

/// Represents an RTSP request.
///
/// An RTSP request consists of a header and a, potentially empty, body. The body component is
/// generic, enabling arbitrary types to represent the RTSP body.
///
/// This struct implements `PartialEq` but care should be taken when using it. Two requests can
/// be semantically equivalent but not be byte by byte. This will mainly occur due to extra spaces
/// in headers. Even when using a typed request, the same problem will occur.
///
/// Note that it is not necessary to ever set the `Content-Length` header as it will be forcibly
/// set during encoding even if it is already present.
#[derive(Clone, Eq, PartialEq)]
pub struct Request<B, H = HeaderMap>
where
    H: Default,
{
    /// The body component of the request. This is generic to support arbitrary content types.
    body: B,

    /// A header map that will either be `HeaderMap` or `TypedHeaderMap`.
    headers: H,

    /// The RTSP method to be applied to the resource. This can be any standardized RTSP method or
    /// an extension method.
    method: Method,

    /// The absolute RTSP request URI (including scheme, host, and port) for the target resource.
    /// IPv6 literals are supported.
    ///
    /// RTSP also supports specifying just `*` for the URI in the request line indicating that the
    /// request does not apply to a particular resource but to the server or proxy itself. This is
    /// only allowed when the request method does not necessarily apply to a resource.
    uri: RequestURIField,

    /// The protocol version that is being used.
    version: Version,
}

impl Request<()> {
    /// Constructs a new builder that uses untyped headers.
    pub fn builder() -> Builder {
        Builder::new()
    }

    /// Constructs a new builder that uses typed headers.
    pub fn typed_builder() -> Builder<TypedHeaderMap> {
        Builder::new()
    }

    /// A convenience function for quickly creating a new builder with the method set to
    /// `"DESCRIBE"` and the request URI field set to `uri`.
    pub fn describe<T>(uri: T) -> Builder
    where
        RequestURIField: TryFrom<T>,
    {
        let mut b = Builder::new();
        b.method(Method::Describe).uri(uri);
        b
    }

    /// A convenience function for quickly creating a new builder with the method set to
    /// `"GET_PARAMETER"` and the request URI field set to `uri`.
    pub fn get_parameter<T>(uri: T) -> Builder
    where
        RequestURIField: TryFrom<T>,
    {
        let mut b = Builder::new();
        b.method(Method::GetParameter).uri(uri);
        b
    }

    /// A convenience function for quickly creating a new builder with the method set to `"OPTIONS"`
    /// and the request URI field set to `uri`.
    pub fn options<T>(uri: T) -> Builder
    where
        RequestURIField: TryFrom<T>,
    {
        let mut b = Builder::new();
        b.method(Method::Options).uri(uri);
        b
    }

    /// A convenience function for quickly creating a new builder with the method set to `"PAUSE"`
    /// and the request URI set to `uri`.
    pub fn pause<T>(uri: T) -> Builder
    where
        RequestURIField: TryFrom<T>,
    {
        let mut b = Builder::new();
        b.method(Method::Pause).uri(uri);
        b
    }

    /// A convenience function for quickly creating a new builder with the method set to `"PLAY"`
    /// and the request URI set to `uri`.
    pub fn play<T>(uri: T) -> Builder
    where
        RequestURIField: TryFrom<T>,
    {
        let mut b = Builder::new();
        b.method(Method::Play).uri(uri);
        b
    }

    /// A convenience function for quickly creating a new builder with the method set to
    /// `"PLAY_NOTIFY"` and the request URI set to `uri`.
    pub fn play_notify<T>(uri: T) -> Builder
    where
        RequestURIField: TryFrom<T>,
    {
        let mut b = Builder::new();
        b.method(Method::PlayNotify).uri(uri);
        b
    }

    /// A convenience function for quickly creating a new builder with the method set to
    /// `"REDIRECT"` and the request URI set to `uri`.
    pub fn redirect<T>(uri: T) -> Builder
    where
        RequestURIField: TryFrom<T>,
    {
        let mut b = Builder::new();
        b.method(Method::Redirect).uri(uri);
        b
    }

    /// A convenience function for quickly creating a new builder with the method set to
    /// `"SET_PARAMETER"` and the request URI set to `uri`.
    pub fn set_parameter<T>(uri: T) -> Builder
    where
        RequestURIField: TryFrom<T>,
    {
        let mut b = Builder::new();
        b.method(Method::SetParameter).uri(uri);
        b
    }

    /// A convenience function for quickly creating a new builder with the method set to `"SETUP"`
    /// and the request URI set to `uri`.
    pub fn setup<T>(uri: T) -> Builder
    where
        RequestURIField: TryFrom<T>,
    {
        let mut b = Builder::new();
        b.method(Method::Setup).uri(uri);
        b
    }

    /// A convenience function for quickly creating a new builder with the method set to
    /// `"TEARDOWN"` and the request URI set to `uri`.
    pub fn teardown<T>(uri: T) -> Builder
    where
        RequestURIField: TryFrom<T>,
    {
        let mut b = Builder::new();
        b.method(Method::Teardown).uri(uri);
        b
    }
}

impl<B, H> Request<B, H>
where
    H: Default,
{
    /// Returns an immutable reference to the request body.
    pub fn body(&self) -> &B {
        &self.body
    }

    /// Returns a mutable reference to the request body. To Change the type of the body, use the
    /// `map` function.
    pub fn body_mut(&mut self) -> &mut B {
        &mut self.body
    }

    /// Returns an immutable reference to the request header map.
    pub fn headers(&self) -> &H {
        &self.headers
    }

    /// Returns a mutable reference to the request header map.
    pub fn headers_mut(&mut self) -> &mut H {
        &mut self.headers
    }

    /// Maps the body of this request to a new type `T` using the provided function.
    pub fn map<T, F>(self, mut mapper: F) -> Request<T, H>
    where
        F: FnMut(B) -> T,
    {
        Request {
            body: mapper(self.body),
            headers: self.headers,
            method: self.method,
            uri: self.uri,
            version: self.version,
        }
    }

    /// Returns an immutable reference to the request method.
    pub fn method(&self) -> &Method {
        &self.method
    }

    /// Returns a mutable reference to the request method.
    pub fn method_mut(&mut self) -> &mut Method {
        &mut self.method
    }

    /// Returns an immutable reference to the request URI.
    pub fn uri(&self) -> &RequestURIField {
        &self.uri
    }

    /// Returns a mutable reference to the request URI.
    pub fn uri_mut(&mut self) -> &mut RequestURIField {
        &mut self.uri
    }

    /// Returns a copy of the request version.
    pub fn version(&self) -> Version {
        self.version
    }

    /// Returns a mutable reference to the request version.
    pub fn version_mut(&mut self) -> &mut Version {
        &mut self.version
    }
}

impl<B> From<Request<B>> for Request<B, TypedHeaderMap> {
    /// Converts the request from using untyped headers to typed headers.
    fn from(value: Request<B>) -> Request<B, TypedHeaderMap> {
        Request {
            body: value.body,
            headers: value.headers.into(),
            method: value.method,
            uri: value.uri,
            version: value.version,
        }
    }
}

impl<B> From<Request<B, TypedHeaderMap>> for Request<B> {
    /// Converts the request from using typed headers to untyped headers.
    fn from(value: Request<B, TypedHeaderMap>) -> Request<B> {
        Request {
            body: value.body,
            headers: value.headers.into(),
            method: value.method,
            uri: value.uri,
            version: value.version,
        }
    }
}

impl<B, H> fmt::Debug for Request<B, H>
where
    B: fmt::Debug,
    H: fmt::Debug + Default,
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter
            .debug_struct("Request")
            .field("method", self.method())
            .field("uri", self.uri())
            .field("version", &self.version())
            .field("headers", self.headers())
            .field("body", self.body())
            .finish()
    }
}

/// Alias for a request using typed headers.
pub type TypedRequest<B> = Request<B, TypedHeaderMap>;

/// An RTSP request builder
///
/// This type can be used to construct a `Request` through a builder-like pattern.
#[derive(Clone, Debug)]
pub struct Builder<H = HeaderMap>
where
    H: Default,
{
    /// A stored error used when making a `Request`.
    pub(crate) error: Option<BuilderError>,

    /// A header map that will either be `HeaderMap` or `TypedHeaderMap`.
    pub(crate) headers: H,

    /// The RTSP method to be applied to the resource. This can be any standardized RTSP method or
    /// an extension method.
    pub(crate) method: Option<Method>,

    /// The absolute RTSP URI (including scheme, host, and port) for the target resource. IPv6
    /// literals are supported.
    ///
    /// RTSP also supports specifying just `*` for the URI in the request line indicating that the
    /// request does not apply to a particular resource but to the server or proxy itself. This is
    /// only allowed when the request method does not necessarily apply to a resource.
    pub(crate) uri: Option<RequestURIField>,

    /// The protocol version that is being used.
    pub(crate) version: Version,
}

impl<H> Builder<H>
where
    H: Default,
{
    /// Creates a new default instance of a `Builder` to construct a `Request`.
    pub fn new() -> Builder<H> {
        Builder::default()
    }

    /// Constructs a `Request` by using the given body. Note that this function does not consume
    /// the builder, allowing you to construct requests with different bodies with the same
    /// builder, but all of the fields will be reset.
    ///
    /// # Errors
    ///
    /// An error will be returned if part of the request is invalid.
    ///
    /// # Examples
    ///
    /// ```
    /// use rtsp::*;
    ///
    /// let request = Request::builder()
    ///     .method(Method::Setup)
    ///     .uri("rtsp://server.com")
    ///     .build(())
    ///     .unwrap();
    /// ```
    pub fn build<B>(&mut self, body: B) -> Result<Request<B, H>, BuilderError> {
        if let Some(error) = self.error {
            return Err(error);
        }

        if let Some(method) = replace(&mut self.method, None) {
            if let Some(uri) = replace(&mut self.uri, None) {
                Ok(Request {
                    body,
                    headers: replace(&mut self.headers, H::default()),
                    method,
                    uri,
                    version: self.version,
                })
            } else {
                Err(BuilderError::MissingRequestURI)
            }
        } else {
            Err(BuilderError::MissingMethod)
        }
    }

    /// Set the method for this request.
    ///
    /// # Errors
    ///
    /// An error will be stored if the given method is an invalid `Method`. Also, this does not have
    /// a default value and, as a result, it must be specified before `build` is called.
    ///
    /// # Examples
    ///
    /// ```
    /// use rtsp::*;
    ///
    /// let request = Request::builder()
    ///     .method(Method::Setup)
    ///     .uri("rtsp://server.com")
    ///     .build(())
    ///     .unwrap();
    /// ```
    pub fn method<T>(&mut self, method: T) -> &mut Self
    where
        Method: TryFrom<T>,
    {
        match Method::try_from(method) {
            Ok(method) => self.method = Some(method),
            Err(_) => self.error = Some(BuilderError::InvalidMethod),
        }

        self
    }

    /// Set the URI for this request.
    ///
    /// # Errors
    ///
    /// An error will be stored if the given URI is an invalid `URI`. Also, this does not have a
    /// default value and, as a result, it must be specified before `build` is called.
    ///
    /// # Examples
    ///
    /// ```
    /// use rtsp::*;
    ///
    /// let request = Request::builder()
    ///     .method(Method::Setup)
    ///     .uri("rtsp://server.com")
    ///     .build(())
    ///     .unwrap();
    /// ```
    pub fn uri<T>(&mut self, uri: T) -> &mut Self
    where
        RequestURIField: TryFrom<T>,
    {
        match RequestURIField::try_from(uri) {
            Ok(uri) => self.uri = Some(uri),
            Err(_) => self.error = Some(BuilderError::InvalidRequestURI),
        }

        self
    }

    /// Set the version for this request.
    ///
    /// The default value for the version is RTSP/2.0.
    ///
    /// # Errors
    ///
    /// An error will be stored if the given version is an invalid or unsupported `Version`.
    /// Currently the only supported version is RTSP/2.0.
    ///
    /// # Examples
    ///
    /// ```
    /// use rtsp::*;
    ///
    /// let request = Request::builder()
    ///     .method(Method::Setup)
    ///     .uri("rtsp://server.com")
    ///     .version(Version::RTSP20)
    ///     .build(())
    ///     .unwrap();
    /// ```
    pub fn version<T>(&mut self, version: T) -> &mut Self
    where
        Version: TryFrom<T>,
    {
        match Version::try_from(version) {
            Ok(version) if version == Version::RTSP20 => self.version = version,
            Ok(_) => self.error = Some(BuilderError::UnsupportedVersion),
            Err(_) => self.error = Some(BuilderError::InvalidVersion),
        }

        self
    }
}

impl Builder<HeaderMap> {
    /// Appends a header to this request.
    ///
    /// This function will append the provided key/value as a header to the internal `HeaderMap`
    /// being constructed. Essentially, this is equivalent to calling `HeaderMap::append`. Because
    /// of this, you are able to add a given header multiple times.
    ///
    /// By default, the request contains no headers.
    ///
    /// # Errors
    ///
    /// An error will be stored if the given name is an invalid `HeaderName`, or the given value is
    /// an invalid `HeaderValue`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rtsp::*;
    ///
    /// let request = Request::builder()
    ///     .method(Method::Play)
    ///     .uri("rtsp://server.com")
    ///     .header("CSeq", "835")
    ///     .header("Session", "ULExwZCXh2pd0xuFgkgZJW")
    ///     .build(())
    ///     .unwrap();
    /// ```
    pub fn header<K, V>(&mut self, key: K, value: V) -> &mut Self
    where
        HeaderName: TryFrom<K>,
        HeaderValue: TryFrom<V>,
    {
        match HeaderName::try_from(key) {
            Ok(key) => match HeaderValue::try_from(value) {
                Ok(value) => {
                    self.headers.append(key, value);
                }
                Err(_) => self.error = Some(BuilderError::InvalidHeaderValue),
            },
            Err(_) => self.error = Some(BuilderError::InvalidHeaderName),
        }

        self
    }

    /// Converts this builder into a builder that contains typed headers.
    pub fn into_typed(self) -> Builder<TypedHeaderMap> {
        Builder {
            error: self.error,
            headers: self.headers.into(),
            method: self.method,
            uri: self.uri,
            version: self.version,
        }
    }
}

impl Builder<TypedHeaderMap> {
    /// Sets a typed header for this request.
    ///
    /// By default, the request contains no headers.
    ///
    /// # Errors
    ///
    /// Since typed headers are used here, this function cannot produce an error for the builder.
    ///
    /// # Examples
    ///
    /// ```
    /// # #![feature(try_from)]
    /// #
    /// use std::convert::TryFrom;
    ///
    /// use rtsp::*;
    /// use rtsp::header::types::*;
    ///
    /// let request = Request::typed_builder()
    ///     .method(Method::Play)
    ///     .uri("rtsp://server.com")
    ///     .header(ContentLength::try_from(5).unwrap())
    ///     .build(())
    ///     .unwrap();
    /// ```
    pub fn header<H: TypedHeader>(&mut self, header: H) -> &mut Self {
        self.headers.set(header);
        self
    }

    /// Sets a raw header for this request. This is slightly different from the untyped builder's
    /// `header` function in that setting the raw value for a previously set header will end up
    /// overwriting it.
    ///
    /// By default, the request contains no headers.
    ///
    /// # Errors
    ///
    /// An error will be stored in the builder if the given name is an invalid `HeaderName`, or the
    /// given values contains an invalid `HeaderValue`.
    ///
    /// # Examples
    ///
    /// ```
    /// # #![feature(try_from)]
    /// #
    /// use std::convert::TryFrom;
    ///
    /// use rtsp::*;
    ///
    /// let request = Request::typed_builder()
    ///     .method(Method::Play)
    ///     .uri("rtsp://server.com")
    ///     .header_raw(HeaderName::ContentLength, vec![HeaderValue::try_from("5").unwrap()])
    ///     .build(())
    ///     .unwrap();
    /// ```
    pub fn header_raw(&mut self, name: HeaderName, value: Vec<HeaderValue>) -> &mut Self {
        self.headers.set_raw(name, value);
        self
    }

    /// Converts this builder into a builder that contains untyped headers.
    pub fn into_untyped(self) -> Builder<HeaderMap> {
        Builder {
            error: self.error,
            headers: self.headers.into(),
            method: self.method,
            uri: self.uri,
            version: self.version,
        }
    }
}

impl<H> Default for Builder<H>
where
    H: Default,
{
    #[inline]
    fn default() -> Self {
        Builder {
            error: None,
            headers: H::default(),
            method: None,
            uri: None,
            version: Version::default(),
        }
    }
}

/// An error type for when the request builder encounters an error.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum BuilderError {
    /// This error indicates that a given header name was invalid.
    InvalidHeaderName,

    /// This error indicates that a given header value was invalid.
    InvalidHeaderValue,

    /// This error indicates that the given method was invalid.
    InvalidMethod,

    /// This error indicates that the given request URI was invalid.
    InvalidRequestURI,

    /// This error indicates that the version was invalid.
    InvalidVersion,

    /// This error indicates that a method was not specified.
    MissingMethod,

    /// This error indicates that a request URI was not specified.
    MissingRequestURI,

    /// This error indicates that the version was unsupported. The only supported version is
    /// RTSP 2.0.
    UnsupportedVersion,
}

impl fmt::Display for BuilderError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(self.description())
    }
}

impl Error for BuilderError {
    fn description(&self) -> &str {
        use self::BuilderError::*;

        match self {
            &InvalidHeaderName => "invalid RTSP header name",
            &InvalidHeaderValue => "invalid RTSP header value",
            &InvalidMethod => "invalid RTSP method",
            &InvalidRequestURI => "invalid RTSP request URI",
            &InvalidVersion => "invalid RTSP version",
            &MissingMethod => "missing RTSP method",
            &MissingRequestURI => "missing RTSP request URI",
            &UnsupportedVersion => "unsupported RTSP version",
        }
    }
}
