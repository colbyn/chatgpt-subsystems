use std::{rc::Rc, cell::RefCell};

use serde::{Deserialize, Serialize};
use tokio_stream::StreamExt;

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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    pub role: Role,
    pub content: String,
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

impl ResponseFormat {
    pub fn json_object() -> Self {
        Self { r#type: ResponseType::JsonObject }
    }
    pub fn text() -> Self {
        Self { r#type: ResponseType::Text }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatRequest {
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
    pub n: Option<i32>,
    /// The maximum number of tokens allowed for the generated answer.
    ///
    /// By default, the number of tokens the model can
    /// return will be (4096 - prompt tokens).
    pub max_tokens: Option<i32>,
    /// An alternative to sampling with temperature, called nucleus sampling, where the model considers the results of
    /// the tokens with `topP` probability mass.
    ///
    /// So 0.1 means only the tokens comprising the top 10% probability mass are considered.
    pub top_p: Option<f32>,
    /// Number between -2.0 and 2.0.
    ///
    /// Positive values penalize new tokens based on their existing frequency in the text
    /// so far, decreasing the model's likelihood to repeat the same line verbatim.
    pub frequency_penalty: Option<f32>,
    /// Number between -2.0 and 2.0.
    ///
    /// Positive values penalize new tokens based on whether they appear in the text so far,
    /// increasing the model's likelihood to talk about new topics.
    pub presence_penalty: Option<f32>,
    /// Include the log probabilities on the `logprobs` most likely tokens, as well the chosen tokens.
    ///
    /// For example, if `logprobs` is 5, the API will return a list of the 5 most likely tokens.
    /// The API will always return the `logprob` of the sampled token, so there may be up to
    /// `logprobs+1` elements in the response. The maximum value for `logprobs` is 5.
    pub logprobs: Option<i32>,
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
}


impl Default for ChatRequest {
    fn default() -> Self {
        Self {
            messages: Vec::default(),
            model: String::from("gpt-3.5-turbo-1106"),
            stream: Some(true),
            temperature: None,
            n: None,
            max_tokens: Some(4096),
            top_p: None,
            frequency_penalty: None,
            presence_penalty: None,
            logprobs: None,
            response_format: None,
            stop: None,
        }
    }
}

impl ChatRequest {
    pub async fn invoke<L: FnMut(&str) -> ()>(
        &self,
        api_key: &str,
        logger: Rc<RefCell<L>>,
        timeout: std::time::Duration
    ) -> Result<Vec<CompletionChunk>, Box<dyn std::error::Error>> {
        let url = "https://api.openai.com/v1/chat/completions";
        let client = reqwest::ClientBuilder::new()
            .timeout(timeout)
            .build()
            .unwrap();
        let response_stream = client
            .post(url)
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&self)
            .send()
            .await?;
        if !response_stream.status().is_success() {
            println!("[CHAR-GPT] FAILED\n```{}\n```", serde_json::to_string_pretty(&self).unwrap())
        }
        assert!(response_stream.status().is_success());
        let response_stream = response_stream.bytes_stream();
        tokio::pin!(response_stream);
        let mut results: Vec<CompletionChunk> = Vec::default();
        let logger = logger.clone();
        while let Some(item) = response_stream.next().await {
            let chunk = item?;
            let text = String::from_utf8(chunk.to_vec())?;
            for line in text.lines() {
                let logger = logger.clone();
                if line.starts_with("data: ") {
                    let json_part = &line["data: ".len()..];
                    if let Ok(response) = serde_json::from_str::<CompletionChunk>(json_part) {
                        results.push(response.clone());
                        let msg = response.choices
                            .iter()
                            .filter_map(|x| x.delta.content.clone())
                            .collect::<String>();
                        let mut logger = logger.borrow_mut();
                        logger(&msg);
                    }
                }
            }
        }
        Ok(results)
    }
}

impl Message {
    pub fn user(content: impl Into<String>) -> Self {
        Self { role: Role::User, content: content.into() }
    }
    pub fn assistant(content: impl Into<String>) -> Self {
        Self { role: Role::Assistant, content: content.into() }
    }
    pub fn system(content: impl Into<String>) -> Self {
        Self { role: Role::System, content: content.into() }
    }
}

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
    pub index: i64,
    pub delta: ChatResponseDelta,
    pub finish_reason: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatResponseDelta {
    pub content: Option<String>,
}

