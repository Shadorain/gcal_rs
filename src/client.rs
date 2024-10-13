use reqwest::{
    header::{HeaderMap, HeaderValue},
    ClientBuilder, RequestBuilder, Response,
};

use super::{CalendarListClient, ClientError, ClientResult, EventClient, OToken, Sendable};

/// Client is a Google Calendar client. The access key must have already been fetched and the oauth
/// negotiation should have already been completed. The client itself only implements HTTP verbs
/// that accept Sendable implementations. You must use the decorated clients such as EventClient
/// and CalendarListClient to do transactional work.
#[derive(Debug, Clone)]
pub struct GCalClient {
    client: reqwest::Client,
    headers: Option<HeaderMap<HeaderValue>>,
    token: OToken,

    debug: bool,
}

impl GCalClient {
    /// Create a new client. Requires an access key.
    pub fn new(token: OToken) -> ClientResult<Self> {
        let client = ClientBuilder::new().gzip(true).https_only(true).build()?;

        Ok(Self {
            client,
            headers: None,
            token,
            debug: false,
        })
    }
    pub fn calendar_client(&self) -> CalendarListClient {
        CalendarListClient::new(self.clone())
    }
    pub fn event_client(&self) -> EventClient {
        EventClient::new(self.clone())
    }
    pub fn clients(self) -> (CalendarListClient, EventClient) {
        (
            CalendarListClient::new(self.clone()),
            EventClient::new(self),
        )
    }

    pub fn set_debug(&mut self) {
        self.debug = true
    }

    /// Perform a GET request.
    pub async fn get(
        &self,
        action: Option<String>,
        target: impl Sendable,
    ) -> ClientResult<Response> {
        self.send(self.client.get(self.get_url("GET", &target, action)?))
            .await
    }

    /// Perform a POST request.
    pub async fn post(
        &self,
        action: Option<String>,
        target: impl Sendable,
    ) -> ClientResult<Response> {
        self.send(
            self.client
                .post(self.get_url("POST", &target, action)?)
                .body(target.body_bytes()?),
        )
        .await
    }

    /// Perform a PUT request.
    pub async fn put(
        &self,
        action: Option<String>,
        target: impl Sendable,
    ) -> ClientResult<Response> {
        self.send(
            self.client
                .put(self.get_url("PUT", &target, action)?)
                .body(target.body_bytes()?),
        )
        .await
    }

    /// Perform a PATCH request.
    pub async fn patch(
        &self,
        action: Option<String>,
        target: impl Sendable,
    ) -> ClientResult<Response> {
        self.send(
            self.client
                .patch(self.get_url("PATCH", &target, action)?)
                .body(target.body_bytes()?),
        )
        .await
    }

    /// Perform a DELETE request.
    pub async fn delete(
        &self,
        action: Option<String>,
        target: impl Sendable,
    ) -> ClientResult<Response> {
        self.send(self.client.delete(self.get_url("DELETE", &target, action)?))
            .await
    }

    async fn send(&self, mut req: RequestBuilder) -> ClientResult<Response> {
        if let Some(headers) = &self.headers {
            req = req.headers(headers.clone())
        }

        let resp = self.set_bearer(req).send().await?;
        if resp.status() != 200 {
            if let Some(header) = resp.headers().get("WWW-Authenticate") {
                if header
                    .to_str()?
                    .starts_with(r#"Bearer error="invalid_token""#)
                {
                    return Err(ClientError::InvalidToken);
                }
            }
            Ok(resp)
        } else {
            Ok(resp)
        }
    }

    fn get_url(
        &self,
        method: &str,
        target: &impl Sendable,
        action: Option<String>,
    ) -> ClientResult<url::Url> {
        let url = target.url(action)?;

        if self.debug {
            eprintln!(
                "[{}] {} | {}",
                method,
                url,
                String::from_utf8(target.body_bytes()?).unwrap_or_default()
            );
        }

        Ok(url)
    }

    fn set_bearer(&self, req: RequestBuilder) -> RequestBuilder {
        req.header("Authorization", format!("Bearer {}", self.token.access))
    }
}
