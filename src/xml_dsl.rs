use std::{path::Path, collections::HashMap, rc::Rc, cell::RefCell};

// use crate::types::{InputWord, WordModel};
use crate::api;

thread_local! {
    static RUNTIME: RefCell<tokio::runtime::Runtime> = RefCell::new(tokio::runtime::Runtime::new().unwrap());
}


//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――
// TODO
//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――
#[derive(Debug, Clone)]
pub struct Prompt {
    pub name: String,
    pub request: api::ChatRequest,
}

impl Prompt {
    pub async fn execute<L: FnMut(&str) -> ()>(&self, logger: Rc<RefCell<L>>) -> Result<String, Box<dyn std::error::Error>> {
        let outs = self.request.invoke(logger).await?;
        let result = outs
            .into_iter()
            .filter_map(|x| x.choices.first().and_then(|x| x.delta.content.clone()))
            .collect::<String>();
        Ok(result)
    }
    pub fn execute_blocking<L: FnMut(&str) -> ()>(&self, logger: Rc<RefCell<L>>) -> Result<String, Box<dyn std::error::Error>> {
        RUNTIME.with(|rt| {
            rt.borrow().block_on(async {
                self.execute(logger).await
            })
        })
    }
    pub fn parse_from(html_source: &str, prompt_name: &str, globals: &dyn liquid::ObjectView) -> Option<Prompt> {
        let template = liquid::ParserBuilder::with_stdlib()
            .build().unwrap()
            .parse(&html_source).unwrap();
        let html = template.render(&globals).unwrap();
        return internal_parse_html_dsl(&html).get(prompt_name).map(ToOwned::to_owned)
    }
    pub fn read_from(html_path: impl AsRef<Path>, prompt_name: &str, globals: &dyn liquid::ObjectView) -> Option<Prompt> {
        let html_source = std::fs::read_to_string(html_path.as_ref()).ok()?;
        Self::parse_from(&html_source, prompt_name, globals)
    }
    pub fn print_info(&self) {
        use colored::Colorize;
        // let contents = serde_json::to_string_pretty(&self.request).unwrap();
        println!("{}", format!("model: {}", self.request.model).truecolor(191, 122, 255));
        println!("{}", format!("stream: {:?}", self.request.stream).truecolor(191, 122, 255));
        println!("{}", format!("temperature: {:?}", self.request.temperature).truecolor(191, 122, 255));
        println!("{}", format!("n: {:?}", self.request.n).truecolor(191, 122, 255));
        println!("{}", format!("max_tokens: {:?}", self.request.max_tokens).truecolor(191, 122, 255));
        println!("{}", format!("top_p: {:?}", self.request.top_p).truecolor(191, 122, 255));
        println!("{}", format!("frequency_penalty: {:?}", self.request.frequency_penalty).truecolor(191, 122, 255));
        println!("{}", format!("presence_penalty: {:?}", self.request.presence_penalty).truecolor(191, 122, 255));
        println!("{}", format!("logprobs: {:?}", self.request.logprobs).truecolor(191, 122, 255));
        println!("{}", format!("response_format: {:?}", self.request.response_format).truecolor(191, 122, 255));
        println!("{}", format!("stop: {:?}", self.request.stop).truecolor(191, 122, 255));
        for message in self.request.messages.iter() {
            println!("{} {}", "  ─╼ ROLE:".truecolor(191, 122, 255), serde_json::to_string(&message.role).unwrap());
            for line in message.content.lines() {
                println!("{}", &format!("  │ {}", line).truecolor(191, 122, 255));
            }
        }
    }
}


//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――
// TODO
//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――
fn internal_parse_html_dsl(html: &str) -> HashMap<String, Prompt> {
    let fragment = scraper::Html::parse_fragment(html);
    let promp_selector = scraper::Selector::parse("prompt").unwrap();
    let message_selector = scraper::Selector::parse("message").unwrap();
    fragment
        .select(&promp_selector)
        .filter_map(|prompt_element| {
            let name = prompt_element.attr("name")?.to_string();
            let model = prompt_element.attr("model").unwrap_or("gpt-3.5-turbo").to_string();
            let temperature = prompt_element.attr("temperature")
                .and_then(|x| x.parse::<f32>().ok());
            let max_tokens = prompt_element.attr("max-tokens")
                .and_then(|x| x.parse::<i32>().ok());
            let top_p = prompt_element.attr("top-p")
                .and_then(|x| x.parse::<f32>().ok());
            let frequency_penalty = prompt_element.attr("frequency-penalty")
                .and_then(|x| x.parse::<f32>().ok());
            let presence_penalty = prompt_element.attr("presence-penalty")
                .and_then(|x| x.parse::<f32>().ok());
            let response_format = prompt_element.attr("response-format")
                .and_then(|x| {
                    match x.to_lowercase().as_str() {
                        "json-object" => Some(api::ResponseFormat::json_object()),
                        "json_object" => Some(api::ResponseFormat::json_object()),
                        "text" => Some(api::ResponseFormat::text()),
                        _ => None
                    }
                });
            let messages = prompt_element
                .select(&message_selector)
                .map(|message_element| {
                    let role = api::Role::from(message_element.attr("role").unwrap_or("user")).unwrap();
                    let content = message_element.inner_html().trim().to_string();
                    // let content = unindent::unindent(&content);
                    api::Message{role, content}
                })
                .collect::<Vec<_>>();
            let mut request = api::ChatRequest::default();
            request.messages = messages;
            request.model = model;
            request.temperature = temperature;
            request.max_tokens = max_tokens;
            request.top_p = top_p;
            request.frequency_penalty = frequency_penalty;
            request.presence_penalty = presence_penalty;
            request.response_format = response_format;
            Some((name.clone(), Prompt {name, request}))
        })
        .collect::<HashMap<_, _>>()
}