use crate::domain::{
    GeminiModel, GeneratedImage, ImageGenerationError, ImageGenerationRepository,
    ImageGenerationRequest,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Gemini APIクライアント
pub struct GeminiClient {
    api_key: String,
    api_base_url: String,
    http_client: reqwest::Client,
}

impl GeminiClient {
    pub fn new(api_key: String) -> Self {
        let api_base_url = std::env::var("GEMINI_API_BASE_URL")
            .unwrap_or_else(|_| "https://generativelanguage.googleapis.com/v1beta".to_string());
        Self {
            api_key,
            api_base_url,
            http_client: reqwest::Client::new(),
        }
    }

    pub fn with_base_url(api_key: String, api_base_url: String) -> Self {
        Self {
            api_key,
            api_base_url,
            http_client: reqwest::Client::new(),
        }
    }

    /// URLを構築する（テスト用）
    #[doc(hidden)]
    pub fn build_url(&self, model: &GeminiModel) -> String {
        format!("{}/models/{}:generateContent", self.api_base_url, model)
    }
}

#[async_trait]
impl ImageGenerationRepository for GeminiClient {
    async fn generate_image(
        &self,
        request: &ImageGenerationRequest,
    ) -> Result<GeneratedImage, ImageGenerationError> {
        let url = format!(
            "{}/models/{}:generateContent",
            self.api_base_url, request.model
        );

        // Gemini APIのリクエストボディ
        let request_body = GeminiRequest {
            contents: vec![Content {
                parts: vec![Part {
                    text: request.prompt.clone(),
                }],
            }],
        };

        let response = self
            .http_client
            .post(&url)
            .query(&[("key", &self.api_key)])
            .json(&request_body)
            .send()
            .await?;

        let status = response.status();

        // エラーレスポンスの処理
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return match status.as_u16() {
                401 => Err(ImageGenerationError::AuthenticationError(
                    "Invalid API key".to_string(),
                )),
                429 => Err(ImageGenerationError::RateLimitError(
                    "Rate limit exceeded".to_string(),
                )),
                400 => Err(ImageGenerationError::InvalidPromptError(error_text)),
                _ => Err(ImageGenerationError::ApiError(format!(
                    "API returned status {}: {}",
                    status, error_text
                ))),
            };
        }

        let response_body: GeminiResponse = response.json().await?;

        // レスポンスから画像データを抽出
        let image_data = extract_image_data(&response_body)?;

        Ok(GeneratedImage::new(image_data, request.model.clone()))
    }
}

/// Gemini APIリクエストボディ
#[derive(Debug, Serialize)]
struct GeminiRequest {
    contents: Vec<Content>,
}

#[derive(Debug, Serialize)]
struct Content {
    parts: Vec<Part>,
}

#[derive(Debug, Serialize)]
struct Part {
    text: String,
}

/// Gemini APIレスポンスボディ
#[derive(Debug, Deserialize)]
struct GeminiResponse {
    candidates: Vec<Candidate>,
}

#[derive(Debug, Deserialize)]
struct Candidate {
    content: ResponseContent,
}

#[derive(Debug, Deserialize)]
struct ResponseContent {
    parts: Vec<ResponsePart>,
}

#[derive(Debug, Deserialize)]
struct ResponsePart {
    #[serde(rename = "inlineData")]
    inline_data: Option<InlineData>,
}

#[derive(Debug, Deserialize)]
struct InlineData {
    #[serde(rename = "mimeType")]
    #[allow(dead_code)]
    mime_type: Option<String>, // APIレスポンスに含まれる可能性があるが、現在は未使用（将来の拡張用）
    data: String, // base64エンコードされた画像データ
}

/// レスポンスから画像データを抽出
fn extract_image_data(response: &GeminiResponse) -> Result<Vec<u8>, ImageGenerationError> {
    let candidate = response
        .candidates
        .first()
        .ok_or_else(|| ImageGenerationError::ApiError("No candidates in response".to_string()))?;

    let part = candidate
        .content
        .parts
        .iter()
        .find(|p| p.inline_data.is_some())
        .ok_or_else(|| ImageGenerationError::ApiError("No image data in response".to_string()))?;

    let inline_data = part
        .inline_data
        .as_ref()
        .ok_or_else(|| ImageGenerationError::ApiError("No inline data".to_string()))?;

    // base64デコード
    use base64::Engine;
    base64::engine::general_purpose::STANDARD
        .decode(&inline_data.data)
        .map_err(|e| ImageGenerationError::ApiError(format!("Failed to decode base64: {}", e)))
}
