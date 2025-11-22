use crate::domain::models::{GeneratedImage, ImageGenerationRequest};

/// 画像生成リポジトリのトレイト
/// ドメイン層で定義し、インフラ層で実装する（依存関係の逆転）
#[async_trait::async_trait]
pub trait ImageGenerationRepository: Send + Sync {
    /// 画像を生成する
    async fn generate_image(
        &self,
        request: &ImageGenerationRequest,
    ) -> Result<GeneratedImage, ImageGenerationError>;
}

/// 画像生成エラー
#[derive(Debug, thiserror::Error)]
pub enum ImageGenerationError {
    #[error("API authentication error: {0}")]
    AuthenticationError(String),
    #[error("Rate limit exceeded: {0}")]
    RateLimitError(String),
    #[error("Invalid prompt: {0}")]
    InvalidPromptError(String),
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("API error: {0}")]
    ApiError(String),
    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl From<reqwest::Error> for ImageGenerationError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            Self::NetworkError(format!("Request timeout: {}", err))
        } else if err.is_connect() {
            Self::NetworkError(format!("Connection error: {}", err))
        } else {
            Self::NetworkError(format!("HTTP error: {}", err))
        }
    }
}
