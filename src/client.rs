use std::{cell::RefCell, rc::Rc, str::FromStr};
use serde::{Deserialize, Serialize};
use tokio_stream::StreamExt;

thread_local! {
    static RUNTIME: RefCell<tokio::runtime::Runtime> = RefCell::new(tokio::runtime::Runtime::new().unwrap());
}


//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――
// TODO
//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ConfigurationBuilder {
    /// ID of the model to use.
    pub model: Option<String>,
    /// If set, partial message deltas will be sent, like in ChatGPT.
    /// Tokens will be sent as data-only server-sent events as they become
    /// available, with the stream terminated by a data: [DONE] message.
    pub stream: Option<bool>,
    /// What sampling temperature to use, between 0 and 2.
    ///
    /// Higher values like 0.8 will make the output more random,
    /// while lower values like 0.2 will make it more focused and deterministic.
    pub temperature: Option<f32>,
    /// How many chat completion choices to generate for each input message.
    pub n: Option<usize>,
    /// The maximum number of tokens allowed for the generated answer.
    ///
    /// By default, the number of tokens the model can
    /// return will be (4096 - prompt tokens).
    pub max_tokens: Option<usize>,
    /// An alternative to sampling with temperature, called nucleus sampling, where
    /// the model considers the results of the tokens with `topP` probability mass.
    ///
    /// So `0.1` means only the tokens comprising the top 10% probability mass are
    /// considered.
    pub top_p: Option<f32>,
    /// Number between `-2.0` and `2.0.`
    ///
    /// Positive values penalize new tokens based on their existing frequency in the text
    /// so far, decreasing the model's likelihood to repeat the same line verbatim.
    pub frequency_penalty: Option<f32>,
    /// Number between -2.0 and 2.0.
    ///
    /// Positive values penalize new tokens based on whether they appear in the text so far,
    /// increasing the model's likelihood to talk about new topics.
    pub presence_penalty: Option<f32>,
    /// Whether to return log probabilities of the output tokens or not.
    /// 
    /// If true, returns the log probabilities of each output token returned
    /// in the `content` of `message`.
    /// 
    /// This option is currently **not available** on the `gpt-4-vision-preview` model.
    pub logprobs: Option<bool>,
    /// An integer between 0 and 5 specifying the number of most likely tokens to
    /// return at each token position, each with an associated log probability.
    /// 
    /// `logprobs` must be set to true if this parameter is used.
    pub top_logprobs: Option<usize>,
    /// An object specifying the format that the model must output.
    /// 
    /// Setting to `ChatCompletionsRequest.ResponseFormat.json` enables JSON mode,
    /// which "guarantees" the message the model generates is valid JSON.
    ///
    /// **Important:** when using JSON mode, you must also instruct the model to
    /// produce JSON yourself via a system or user message. Without this, the model
    /// may generate an unending stream of whitespace until the generation reaches
    /// the token limit, resulting in a long-running and seemingly "stuck" request.
    ///
    /// Also note that the message content may be partially cut off if `finish_reason="length"`,
    /// which indicates the generation exceeded max_tokens or the conversation exceeded the max
    /// context length.
    pub response_format: Option<ResponseFormat>,
    /// Up to 4 sequences where the API will stop generating further tokens.
    ///
    /// The returned text will not contain the stop sequence.
    pub stop: Option<Vec<String>>,
    /// This feature is in Beta.
    /// 
    /// If specified, our system will make a best effort to sample deterministically,
    /// such that repeated requests with the same seed and parameters should return
    /// the same result.
    /// 
    /// Determinism is not guaranteed, and you should refer to the system_fingerprint
    /// response parameter to monitor changes in the backend.
    pub seed: Option<isize>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ResponseType {
    Text,
    JsonObject,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub struct ResponseFormat {
    r#type: ResponseType
}

impl ConfigurationBuilder {
    pub fn with_model(mut self, model: impl AsRef<str>) -> Self {
        self.model = Some(model.as_ref().to_string());
        self
    }
    pub fn with_stream(mut self, stream: bool) -> Self {
        self.stream = Some(stream);
        self
    }
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }
    pub fn with_n(mut self, n: usize) -> Self {
        self.n = Some(n);
        self
    }
    pub fn with_max_tokens(mut self, max_tokens: usize) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }
    pub fn with_top_p(mut self, top_p: f32) -> Self {
        self.top_p = Some(top_p);
        self
    }
    pub fn with_frequency_penalty(mut self, frequency_penalty: f32) -> Self {
        self.frequency_penalty = Some(frequency_penalty);
        self
    }
    pub fn with_presence_penalty(mut self, presence_penalty: f32) -> Self {
        self.presence_penalty = Some(presence_penalty);
        self
    }
    pub fn with_logprobs(mut self, logprobs: bool) -> Self {
        self.logprobs = Some(logprobs);
        self
    }
    pub fn with_response_format(mut self, response_format: ResponseFormat) -> Self {
        self.response_format = Some(response_format);
        self
    }
    pub fn with_stop(mut self, stop: Vec<String>) -> Self {
        self.stop = Some(stop);
        self
    }
    pub fn build(self, messages: impl IntoIterator<Item=Message>) -> Option<ChatCompletionsBody> {
        let model = self.model.as_ref()?;
        let mut chat_request = ChatCompletionsBody::new(model, messages);
        chat_request.stream = self.stream.clone();
        chat_request.temperature = self.temperature.clone();
        chat_request.n = self.n.clone();
        chat_request.max_tokens = self.max_tokens.clone();
        chat_request.top_p = self.top_p.clone();
        chat_request.frequency_penalty = self.frequency_penalty.clone();
        chat_request.presence_penalty = self.presence_penalty.clone();
        chat_request.logprobs = self.logprobs.clone();
        chat_request.response_format = self.response_format.clone();
        chat_request.stop = self.stop.clone();
        Some(chat_request)
    }
}


//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――
// TODO
//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Role {
    #[serde(rename = "system")]
    System,
    #[serde(rename = "user")]
    User,
    #[serde(rename = "assistant")]
    Assistant,
}

impl Role {
    pub fn from(string: &str) -> Option<Self> {
        match string.to_lowercase().as_str() {
            "system" => Some(Self::System),
            "assistant" => Some(Self::Assistant),
            "user" => Some(Self::User),
            _ => None
        }
    }
}

impl ResponseFormat {
    pub fn json_object() -> Self {
        Self { r#type: ResponseType::JsonObject }
    }
    pub fn text() -> Self {
        Self { r#type: ResponseType::Text }
    }
}


//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――
// TODO
//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――
pub type Error = Box<dyn std::error::Error>;

#[derive(Debug, Clone)]
pub enum ApiError {
    /// # TODO
    APIConnectionError,
    /// # TODO
    APITimeoutError,
    /// # TODO
    InternalServerError,
    /// # 401 - Invalid Authentication
    AuthenticationError,
    /// # 400 - Bad Request Error
    BadRequestError,
    /// # 409 - Conflict Error
    ConflictError,
    /// # 404 - Not Found Error
    NotFoundError,
    /// # 403 - Permission Denied Error
    PermissionDeniedError,
    /// # 429 - Rate limit reached for requests
    RateLimitError,
    /// # 422 - Unprocessable Entity Error
    UnprocessableEntityError,
}

#[derive(Debug, Clone)]
pub struct RateLimitMetadata {
    /// In seconds.
    pub retry_after: usize,
    pub retry_after_ms: usize,
    pub ratelimit_limit_requests: usize,
    pub ratelimit_limit_tokens: usize,
    pub ratelimit_remaining_requests: usize,
    pub ratelimit_remaining_tokens: usize,
    pub ratelimit_reset_requests: String,
    pub ratelimit_reset_tokens: String,
}

#[derive(Debug, Clone)]
pub struct MissingHeader(String);

impl ApiError {
    pub(crate) fn from_code(status: impl Into<u16>) -> Option<Self> {
        match status.into() {
            400 => Some(ApiError::BadRequestError),
            401 => Some(ApiError::AuthenticationError),
            403 => Some(ApiError::PermissionDeniedError),
            404 => Some(ApiError::NotFoundError),
            409 => Some(ApiError::ConflictError),
            422 => Some(ApiError::UnprocessableEntityError),
            429 => Some(ApiError::RateLimitError),
            _ => None,
        }
    }
}

impl RateLimitMetadata {
    fn from_response(response: &reqwest::Response) -> Result<Self, Box<dyn std::error::Error>> {
        let retry_after = response
            .headers()
            .get("retry-after")
            .ok_or(MissingHeader(String::from("retry-after")))
            .map_err(Box::new)?
            .to_str()?
            .to_string();
        let retry_after_ms = response
            .headers()
            .get("retry-after-ms")
            .ok_or(MissingHeader(String::from("retry-after-ms")))
            .map_err(Box::new)?
            .to_str()?
            .to_string();
        let ratelimit_limit_requests = response
            .headers()
            .get("x-ratelimit-limit-requests")
            .ok_or(MissingHeader(String::from("x-ratelimit-limit-requests")))
            .map_err(Box::new)?
            .to_str()?
            .to_string();
        let ratelimit_limit_tokens = response
            .headers()
            .get("x-ratelimit-limit-tokens")
            .ok_or(MissingHeader(String::from("x-ratelimit-limit-tokens")))
            .map_err(Box::new)?
            .to_str()?
            .to_string();
        let ratelimit_remaining_requests = response
            .headers()
            .get("x-ratelimit-remaining-requests")
            .ok_or(MissingHeader(String::from("x-ratelimit-remaining-requests")))
            .map_err(Box::new)?
            .to_str()?
            .to_string();
        let ratelimit_remaining_tokens = response
            .headers()
            .get("x-ratelimit-remaining-tokens")
            .ok_or(MissingHeader(String::from("x-ratelimit-remaining-tokens")))
            .map_err(Box::new)?
            .to_str()?
            .to_string();
        let ratelimit_reset_requests = response
            .headers()
            .get("x-ratelimit-reset-requests")
            .ok_or(MissingHeader(String::from("x-ratelimit-reset-requests")))
            .map_err(Box::new)?
            .to_str()?
            .to_string();
        let ratelimit_reset_tokens = response
            .headers()
            .get("x-ratelimit-reset-tokens")
            .ok_or(MissingHeader(String::from("x-ratelimit-reset-tokens")))
            .map_err(Box::new)?
            .to_str()?
            .to_string();
        Ok(RateLimitMetadata {
            retry_after: usize::from_str(&retry_after)?,
            retry_after_ms: usize::from_str(&retry_after_ms)?,
            ratelimit_limit_requests: usize::from_str(&ratelimit_limit_requests)?,
            ratelimit_limit_tokens: usize::from_str(&ratelimit_limit_tokens)?,
            ratelimit_remaining_requests: usize::from_str(&ratelimit_remaining_requests)?,
            ratelimit_remaining_tokens: usize::from_str(&ratelimit_remaining_tokens)?,
            ratelimit_reset_requests,
            ratelimit_reset_tokens,
        })
    }
}

impl std::fmt::Display for MissingHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Missing header: '{}'.", self.0)
    }
}
impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            ApiError::APIConnectionError => "api connection error",
            ApiError::APITimeoutError => "api timeout error",
            ApiError::InternalServerError => "internal server error",
            ApiError::AuthenticationError => "authentication error",
            ApiError::BadRequestError => "bad request error",
            ApiError::ConflictError => "conflict error",
            ApiError::NotFoundError => "not found error",
            ApiError::PermissionDeniedError => "permission denied error",
            ApiError::RateLimitError => "rate limit error",
            ApiError::UnprocessableEntityError => "unprocessable entity error",
        };
        write!(f, "{label}")
    }
}

impl std::error::Error for MissingHeader {}
impl std::error::Error for ApiError {}

//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――
// TODO
//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatCompletionsBody {
    pub messages: Vec<Message>,
    /// ID of the model to use.
    pub model: String,
    /// If set, partial message deltas will be sent, like in ChatGPT.
    /// Tokens will be sent as data-only server-sent events as they become
    /// available, with the stream terminated by a data: [DONE] message.
    pub stream: Option<bool>,
    /// What sampling temperature to use, between 0 and 2.
    ///
    /// Higher values like 0.8 will make the output more random,
    /// while lower values like 0.2 will make it more focused and deterministic.
    pub temperature: Option<f32>,
    /// How many chat completion choices to generate for each input message.
    pub n: Option<usize>,
    /// The maximum number of tokens allowed for the generated answer.
    ///
    /// By default, the number of tokens the model can
    /// return will be (4096 - prompt tokens).
    pub max_tokens: Option<usize>,
    /// An alternative to sampling with temperature, called nucleus sampling, where
    /// the model considers the results of the tokens with `topP` probability mass.
    ///
    /// So `0.1` means only the tokens comprising the top 10% probability mass are
    /// considered.
    pub top_p: Option<f32>,
    /// Number between `-2.0` and `2.0.`
    ///
    /// Positive values penalize new tokens based on their existing frequency in the text
    /// so far, decreasing the model's likelihood to repeat the same line verbatim.
    pub frequency_penalty: Option<f32>,
    /// Number between -2.0 and 2.0.
    ///
    /// Positive values penalize new tokens based on whether they appear in the text so far,
    /// increasing the model's likelihood to talk about new topics.
    pub presence_penalty: Option<f32>,
    /// Whether to return log probabilities of the output tokens or not.
    /// 
    /// If true, returns the log probabilities of each output token returned
    /// in the `content` of `message`.
    /// 
    /// This option is currently **not available** on the `gpt-4-vision-preview` model.
    pub logprobs: Option<bool>,
    /// An integer between 0 and 5 specifying the number of most likely tokens to
    /// return at each token position, each with an associated log probability.
    /// 
    /// `logprobs` must be set to true if this parameter is used.
    pub top_logprobs: Option<usize>,
    /// An object specifying the format that the model must output.
    /// 
    /// Setting to `ChatCompletionsRequest.ResponseFormat.json` enables JSON mode,
    /// which "guarantees" the message the model generates is valid JSON.
    ///
    /// **Important:** when using JSON mode, you must also instruct the model to
    /// produce JSON yourself via a system or user message. Without this, the model
    /// may generate an unending stream of whitespace until the generation reaches
    /// the token limit, resulting in a long-running and seemingly "stuck" request.
    ///
    /// Also note that the message content may be partially cut off if `finish_reason="length"`,
    /// which indicates the generation exceeded max_tokens or the conversation exceeded the max
    /// context length.
    pub response_format: Option<ResponseFormat>,
    /// Up to 4 sequences where the API will stop generating further tokens.
    ///
    /// The returned text will not contain the stop sequence.
    pub stop: Option<Vec<String>>,
    /// This feature is in Beta.
    /// 
    /// If specified, our system will make a best effort to sample deterministically,
    /// such that repeated requests with the same seed and parameters should return
    /// the same result.
    /// 
    /// Determinism is not guaranteed, and you should refer to the system_fingerprint
    /// response parameter to monitor changes in the backend.
    pub seed: Option<isize>,
}

impl ChatCompletionsBody {
    pub fn new(model: impl AsRef<str>, messages: impl IntoIterator<Item=Message>) -> Self {
        let model = model.as_ref().to_string();
        let messages = messages.into_iter().collect::<Vec<_>>();
        Self {
            messages,
            model,
            stream: None,
            temperature: None,
            n: None,
            max_tokens: None,
            top_p: None,
            frequency_penalty: None,
            presence_penalty: None,
            logprobs: None,
            top_logprobs: None,
            response_format: None,
            stop: None,
            seed: None,
        }
    }
    pub fn with_model(mut self, model: impl AsRef<str>) -> Self {
        self.model = model.as_ref().to_string();
        self
    }
    pub fn with_stream(mut self, stream: bool) -> Self {
        self.stream = Some(stream);
        self
    }
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }
    pub fn with_n(mut self, n: usize) -> Self {
        self.n = Some(n);
        self
    }
    pub fn with_max_tokens(mut self, max_tokens: usize) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }
    pub fn with_top_p(mut self, top_p: f32) -> Self {
        self.top_p = Some(top_p);
        self
    }
    pub fn with_frequency_penalty(mut self, frequency_penalty: f32) -> Self {
        self.frequency_penalty = Some(frequency_penalty);
        self
    }
    pub fn with_presence_penalty(mut self, presence_penalty: f32) -> Self {
        self.presence_penalty = Some(presence_penalty);
        self
    }
    pub fn with_logprobs(mut self, logprobs: bool) -> Self {
        self.logprobs = Some(logprobs);
        self
    }
    pub fn with_response_format(mut self, response_format: ResponseFormat) -> Self {
        self.response_format = Some(response_format);
        self
    }
    pub fn with_stop(mut self, stop: Vec<String>) -> Self {
        self.stop = Some(stop);
        self
    }
}

//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――
// TODO
//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――
#[derive(Debug, Clone)]
pub struct ApiEndpoint {
    pub api_key: String,
    pub api_url: String,
}

impl ApiEndpoint {
    pub fn open_ai_chat_completions(api_key: impl AsRef<str>) -> Self {
        let api_key = api_key.as_ref().to_string();
        let api_url = "https://api.openai.com/v1/chat/completions".to_string();
        ApiEndpoint { api_key, api_url }
    }
    pub fn octo_ai_chat_completions(api_key: impl AsRef<str>) -> Self {
        let api_key = api_key.as_ref().to_string();
        let api_url = "https://text.octoai.run/v1/chat/completions".to_string();
        ApiEndpoint { api_key, api_url }
    }
}

//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――
// TODO
//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――
pub struct ChatCompletionsRequest {
    pub api_endpoint: ApiEndpoint,
    pub body: ChatCompletionsBody,
    pub timeout: Option<std::time::Duration>,
    pub logger: Option<Rc<RefCell<dyn FnMut(&str) -> ()>>>,
}

#[derive(Clone, Default)]
pub struct ChatCompletionsRequestBuilder {
    pub api_endpoint: Option<ApiEndpoint>,
    pub body: Option<ChatCompletionsBody>,
    pub timeout: Option<std::time::Duration>,
    pub logger: Option<Rc<RefCell<dyn FnMut(&str) -> ()>>>,
}

impl ChatCompletionsRequestBuilder {
    pub fn with_api_endpoint(mut self, api_endpoint: ApiEndpoint) -> Self {
        self.api_endpoint = Some(api_endpoint);
        self
    }
    pub fn with_body(mut self, body: ChatCompletionsBody) -> Self {
        self.body = Some(body);
        self
    }
    pub fn with_timeout(mut self, timeout: std::time::Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }
    pub fn with_logger(mut self, logger: Rc<RefCell<dyn FnMut(&str) -> ()>>) -> Self {
        self.logger = Some(logger);
        self
    }
    pub fn with_logger_closure(mut self, logger: impl FnMut(&str) -> () + 'static) -> Self {
        let logger = Rc::new(RefCell::new(logger));
        self.logger = Some(logger);
        self
    }
    pub fn build(self) -> Option<ChatCompletionsRequest> {
        let api_endpoint = self.api_endpoint.clone()?;
        let body = self.body.clone()?;
        let timeout = self.timeout.clone();
        let logger = self.logger.clone();
        Some(ChatCompletionsRequest { api_endpoint, body, timeout, logger })
    }
}

//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――
// TODO
//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CompletionChunk {
    pub id: String,
    pub choices: Vec<ChatResponseChoice>,
    pub created: i64,
    pub model: String,
    pub system_fingerprint: Option<String>,
    pub object: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatResponseChoice {
    pub index: usize,
    pub delta: ChatResponseDelta,
    pub finish_reason: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatResponseDelta {
    pub content: Option<String>,
}

//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――
// TODO
//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――

pub struct ChatCompletionsStream {

}

impl ChatCompletionsRequest {
    pub async fn execute(&self) -> Result<ChatCompletionsResponse, Error> {
        let url = self.api_endpoint.api_url.as_str();
        let api_key = self.api_endpoint.api_key.as_str();
        let client = {
            if let Some(timeout) = self.timeout.as_ref() {
                reqwest::ClientBuilder::new()
                    .timeout(timeout.clone())
                    .build()
                    .unwrap()
            } else {
                reqwest::ClientBuilder::new().build().unwrap()
            }
        };
        let response = client
            .post(url)
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&self.body)
            .send()
            .await?;
        if let Some(error) = ApiError::from_code(response.status().as_u16()) {
            return Err(Box::new(error))
        }
        let rate_limit_metadata = RateLimitMetadata::from_response(&response).ok();
        let response = response.bytes_stream();
        tokio::pin!(response);
        let mut results: Vec<CompletionChunk> = Vec::default();
        while let Some(item) = response.next().await {
            let chunk = item?;
            let text = String::from_utf8(chunk.to_vec())?;
            for line in text.lines() {
                if line.starts_with("data: ") {
                    let json_part = &line["data: ".len()..];
                    if let Ok(response) = serde_json::from_str::<CompletionChunk>(json_part) {
                        results.push(response.clone());
                        let msg = response.choices
                            .iter()
                            .filter_map(|x| x.delta.content.clone())
                            .collect::<String>();
                        if let Some(logger) = self.logger.as_ref() {
                            let mut logger = logger.borrow_mut();
                            logger(&msg);
                        }
                    }
                }
            }
        }
        let output = results;
        Ok(ChatCompletionsResponse { rate_limit_metadata, output })
    }
    pub fn execute_blocking<L: FnMut(&str) -> ()>(&self) -> Result<ChatCompletionsResponse, Error> {
        RUNTIME.with(|rt| {
            rt.borrow().block_on(async {
                self.execute().await
            })
        })
    }
}

//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――
// TODO
//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――
#[derive(Debug, Clone)]
pub struct ChatCompletionsResponse {
    pub rate_limit_metadata: Option<RateLimitMetadata>,
    pub output: Vec<CompletionChunk>,
}

impl ChatCompletionsResponse {
    pub fn content(&self, index: usize) -> String {
        self.output
            .iter()
            .flat_map(|chunk| {
                chunk.choices
                    .iter()
                    .filter_map(|choice| {
                        if choice.index == index {
                            return choice.delta.content.clone()
                        }
                        None
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>()
            .join("")
    }
}
