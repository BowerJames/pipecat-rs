use url::Url;
use serde_json;

pub enum FrameMeta {
    
}

#[derive(Debug, Clone, PartialEq)]
pub enum Frame {
    DataFrame(DataFrame),
    SystemFrame(SystemFrame),
}

#[derive(Debug, Clone, PartialEq)]
pub enum SystemFrame {
    StartUp,
    Stop,
    Shutdown
}

#[derive(Debug, Clone, PartialEq)]
pub enum DataFrame {
    InputTextFrame(String),
    InputTextDeltaFrame(TextDelta),
    OutputTextFrame(String),
    OutputTextDeltaFrame(TextDelta),
    LLMContextFrame(LLMContext),
    LLMResponseFrame(LLMResponse),
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextDelta {
    pub content: String,
    pub finished: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LLMContext {
    pub conversation: Vec<ConversationItem>,
    tools: Vec<Tool>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConversationItem {
    SystemMessage(Vec<ContentPart>),
    UserMessage(Vec<ContentPart>),
    AssistantMessage(Vec<ContentPart>),
    FunctionCallMessage(FunctionCall)
}

#[derive(Debug, Clone, PartialEq)]
pub enum ContentPart {
    TextPart(String),
    ImagePart(Image),
    AudioPart(Audio),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Image {
    ImageURL(Url),
    ImageBase64(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Audio {
    AudioURL(Url),
    AudioBase64(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
    pub result: FunctionCallResult,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FunctionCallResult {
    Success(FunctionCallResultSuccess),
    Error(String),
    Pending
}

#[derive(Debug, Clone, PartialEq)]
pub enum FunctionCallResultSuccess {
    StringResult(String),
    JsonResult(serde_json::Value),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LLMResponse {
    assistant_message: ConversationItem,
    function_calls: Vec<FunctionCall>,
}