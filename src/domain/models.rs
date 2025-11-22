use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::OnceLock;

/// Gemini画像生成モデル（環境変数から設定可能）
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GeminiModel(String);

/// デフォルトモデル名（環境変数から読み取る）
pub(crate) static DEFAULT_MODEL: OnceLock<String> = OnceLock::new();

/// 許可されたモデルリスト（環境変数から読み取る、空の場合はすべて許可）
pub(crate) static ALLOWED_MODELS: OnceLock<Vec<String>> = OnceLock::new();

impl GeminiModel {
    /// 環境変数からデフォルトモデル名を初期化
    pub fn init_default() {
        let default_model = std::env::var("GEMINI_DEFAULT_MODEL")
            .unwrap_or_else(|_| "gemini-2.5-flash-image".to_string());
        DEFAULT_MODEL.set(default_model).ok();
    }

    /// 環境変数から許可されたモデルリストを初期化
    pub fn init_allowed_models() {
        let allowed = std::env::var("GEMINI_ALLOWED_MODELS")
            .ok()
            .map(|s| {
                s.split(',')
                    .map(|m| m.trim().to_string())
                    .filter(|m| !m.is_empty())
                    .collect()
            })
            .unwrap_or_default();
        ALLOWED_MODELS.set(allowed).ok();
    }

    /// 環境変数を初期化（両方）
    pub fn init_from_env() {
        Self::init_default();
        Self::init_allowed_models();
    }

    /// モデル名を取得
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// モデル名が許可されているかチェック
    pub fn is_allowed(&self) -> bool {
        if let Some(allowed) = ALLOWED_MODELS.get() {
            allowed.is_empty() || allowed.contains(&self.0)
        } else {
            true // 許可リストが初期化されていない場合はすべて許可
        }
    }
}

impl Default for GeminiModel {
    fn default() -> Self {
        let model_name = DEFAULT_MODEL
            .get()
            .cloned()
            .unwrap_or_else(|| "gemini-2.5-flash-image".to_string());
        Self(model_name)
    }
}

impl fmt::Display for GeminiModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<&str> for GeminiModel {
    type Error = ModelParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let model = Self(value.to_string());

        // 許可されたモデルリストが設定されている場合は検証
        if !model.is_allowed() {
            return Err(ModelParseError::InvalidModel(format!(
                "Model '{}' is not in the allowed list",
                value
            )));
        }

        Ok(model)
    }
}

impl From<String> for GeminiModel {
    fn from(value: String) -> Self {
        Self(value)
    }
}

/// 画像生成リクエスト
#[derive(Debug, Clone)]
pub struct ImageGenerationRequest {
    pub prompt: String,
    pub model: GeminiModel,
}

impl ImageGenerationRequest {
    pub fn new(prompt: String) -> Self {
        Self {
            prompt,
            model: GeminiModel::default(),
        }
    }

    pub fn with_model(mut self, model: GeminiModel) -> Self {
        self.model = model;
        self
    }

    /// プロンプトの検証
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.prompt.trim().is_empty() {
            return Err(ValidationError::EmptyPrompt);
        }

        // プロンプトの長さ制限（環境変数MAX_PROMPT_LENGTHから読み取る）
        let max_length = std::env::var("MAX_PROMPT_LENGTH")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(10000);
        if self.prompt.len() > max_length {
            return Err(ValidationError::PromptTooLong(self.prompt.len()));
        }

        Ok(())
    }
}

/// 生成された画像データ
#[derive(Debug, Clone)]
pub struct GeneratedImage {
    pub data: Vec<u8>,
    pub model: GeminiModel,
    pub generated_at: chrono::DateTime<chrono::Utc>,
}

impl GeneratedImage {
    pub fn new(data: Vec<u8>, model: GeminiModel) -> Self {
        Self {
            data,
            model,
            generated_at: chrono::Utc::now(),
        }
    }
}

/// モデルパースエラー
#[derive(Debug, thiserror::Error)]
pub enum ModelParseError {
    #[error("Invalid model: {0}")]
    InvalidModel(String),
}

/// バリデーションエラー
#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Prompt cannot be empty")]
    EmptyPrompt,
    #[error("Prompt too long: {0} characters (max: 10000)")]
    PromptTooLong(usize),
}
