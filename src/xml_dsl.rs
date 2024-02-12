use std::{path::Path, str::FromStr};

use crate::client::{self as api, ChatCompletionsRequestBuilder};

#[derive(Debug, Clone)]
pub struct PromptCollection {
    prompts: Vec<Prompt>,
}

#[derive(Debug, Clone)]
pub struct Prompt {
    pub name: Option<String>,
    pub configuration: api::ConfigurationBuilder,
    pub messages: Vec<api::Message>
}

impl PromptCollection {
    pub fn open(file_path: impl AsRef<Path>) -> Result<Self, Box<dyn std::error::Error>> {
        let source = std::fs::read_to_string(file_path.as_ref())?;
        Self::parse(source)
    } 
    pub fn parse(contents: impl AsRef<str>) -> Result<Self, Box<dyn std::error::Error>> {
        // let contents = std::fs::read_to_string(file_path.as_ref());
        let source = contents.as_ref();
        let html = scraper::Html::parse_fragment(source);
        let selector = scraper::Selector::parse("prompt").unwrap();
        let prompts = html
            .select(&selector)
            .filter_map(process_prompt_element)
            .collect::<Vec<_>>();
        Ok(PromptCollection { prompts })
    }
    pub fn get(&self, prompt_name: impl AsRef<str>) -> Option<Prompt> {
        let target = prompt_name.as_ref();
        for prompt in self.prompts.iter() {
            if let Some(name) = prompt.name.as_ref() {
                if name == &target {
                    return Some(prompt.clone());
                }
            }
        }
        None
    }
}

impl Prompt {
    pub fn open(file_path: impl AsRef<Path>, prompt_name: impl AsRef<str>) -> Result<Self, api::Error> {
        let prompt_name = prompt_name.as_ref();
        let collection = PromptCollection::open(file_path)?;
        let prompt = collection.get(prompt_name)
            .ok_or(Box::new(PromptNotFound(prompt_name.to_string())))?;
        Ok(prompt)
    }
    pub fn parse(contents: impl AsRef<str>, prompt_name: impl AsRef<str>) -> Result<Self, api::Error> {
        let prompt_name = prompt_name.as_ref();
        let collection = PromptCollection::parse(contents)?;
        let prompt = collection.get(prompt_name)
            .ok_or(Box::new(PromptNotFound(prompt_name.to_string())))?;
        Ok(prompt)
    }
    pub fn build_body(&self) -> Option<api::ChatCompletionsBody> {
        self.configuration.clone().build(self.messages.clone())
    }
    pub fn request_builder(&self) -> Option<ChatCompletionsRequestBuilder> {
        let body = self.build_body()?;
        let builder = ChatCompletionsRequestBuilder::default().with_body(body);
        Some(builder)
    }
}

#[derive(Debug, Clone)]
pub struct PromptNotFound(pub String);
impl std::fmt::Display for PromptNotFound {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Cannot find prompt: {:?}.", self.0)
    }
}
impl std::error::Error for PromptNotFound {}



//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――
// TODO
//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――
fn process_prompt_element(element: scraper::ElementRef) -> Option<Prompt> {
    let name = element.attr("name")
        .map(str::to_string);
    let model = element.attr("model")
        .map(str::to_string);
    let stream = element.attr("stream")
        .and_then(|x| bool::from_str(&x).ok());
    let temperature = element.attr("temperature")
        .and_then(|x| f32::from_str(&x).ok());
    let n: Option<usize> = element.attr("n")
        .and_then(|x| usize::from_str(&x).ok());
    let max_tokens = element.attr("max-tokens")
        .and_then(|x| usize::from_str(&x).ok());
    let top_p = element.attr("top-p")
        .and_then(|x| f32::from_str(&x).ok());
    let frequency_penalty = element.attr("frequency-penalty")
        .and_then(|x| f32::from_str(&x).ok());
    let presence_penalty = element.attr("presence-penalty")
        .and_then(|x| f32::from_str(&x).ok());
    let logprobs = element.attr("logprobs")
        .and_then(|x| bool::from_str(&x).ok());
    let top_logprobs = element.attr("top-logprobs")
        .and_then(|x| usize::from_str(&x).ok());
    let response_format = element
        .attr("response-format")
        .and_then(|x| {
            match x.to_lowercase().as_str() {
                "json-object" => Some(api::ResponseFormat::json_object()),
                "json_object" => Some(api::ResponseFormat::json_object()),
                "text" => Some(api::ResponseFormat::text()),
                _ => None
            }
        });
    // let stop = element.attr("stop").map(str::to_string);
    // - * -
    let mut configuration = api::ConfigurationBuilder::default();
    configuration.model = model;
    configuration.stream = stream;
    configuration.temperature = temperature;
    configuration.n = n;
    configuration.max_tokens = max_tokens;
    configuration.top_p = top_p;
    configuration.frequency_penalty = frequency_penalty;
    configuration.presence_penalty = presence_penalty;
    configuration.logprobs = logprobs;
    configuration.top_logprobs = top_logprobs;
    configuration.response_format = response_format;
    // - * -
    let message_selector = scraper::Selector::parse("message").unwrap();
    let messages = element
        .select(&message_selector)
        .map(|message_element| {
            let role = message_element.attr("role").unwrap_or("user");
            let role = api::Role::from(role).unwrap();
            let content = message_element.inner_html().trim().to_string();
            let content = unindent::unindent(&content);
            api::Message{role, content}
        })
        .collect::<Vec<_>>();
    // - * -
    let prompt = Prompt { name, configuration, messages };
    Some(prompt)
}
