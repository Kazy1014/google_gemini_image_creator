use crate::application::GenerateImageUseCase;
use crate::domain::{GeminiModel, ImageGenerationRequest};
use crate::infrastructure::gemini::GeminiClient;
use crate::infrastructure::mcp::types::{CallToolResult, Content, Tool};
use anyhow::Result;
use std::sync::Arc;
use tracing::{error, info};

/// MCPサーバー
pub struct McpServer {
    use_case: Arc<GenerateImageUseCase<GeminiClient>>,
    default_model: String,
    allowed_models: Vec<String>,
}

impl McpServer {
    pub fn new(api_key: String) -> Self {
        // 環境変数から設定を読み取る
        GeminiModel::init_from_env();

        let default_model = std::env::var("GEMINI_DEFAULT_MODEL")
            .unwrap_or_else(|_| "gemini-2.5-flash-image".to_string());

        let allowed_models = std::env::var("GEMINI_ALLOWED_MODELS")
            .ok()
            .map(|s| {
                s.split(',')
                    .map(|m| m.trim().to_string())
                    .filter(|m| !m.is_empty())
                    .collect()
            })
            .unwrap_or_default();

        let client = GeminiClient::new(api_key);
        let use_case = Arc::new(GenerateImageUseCase::new(client));
        Self {
            use_case,
            default_model,
            allowed_models,
        }
    }

    /// MCPツールのリストを取得
    pub fn list_tools(&self) -> Vec<Tool> {
        // 許可されたモデルリストが設定されている場合はenumとして、そうでない場合は文字列として
        let model_schema = if self.allowed_models.is_empty() {
            serde_json::json!({
                "type": "string",
                "description": "Gemini model name to use (can be restricted via GEMINI_ALLOWED_MODELS environment variable)",
                "default": self.default_model
            })
        } else {
            serde_json::json!({
                "type": "string",
                "description": "Gemini model name to use",
                "enum": self.allowed_models,
                "default": self.default_model
            })
        };

        vec![Tool {
            name: "generate_image".to_string(),
            description: Some(
                "Generate images from text prompts using Google Gemini's Banana (image generation feature).".to_string(),
            ),
            input_schema: Some(serde_json::json!({
                "type": "object",
                "properties": {
                    "prompt": {
                        "type": "string",
                        "description": "Text prompt for image generation"
                    },
                    "model": model_schema
                },
                "required": ["prompt"]
            })),
        }]
    }

    /// ツール呼び出しを処理
    pub async fn call_tool(
        &self,
        name: &str,
        arguments: &serde_json::Value,
    ) -> Result<CallToolResult> {
        match name {
            "generate_image" => self.handle_generate_image(arguments).await,
            _ => Err(anyhow::anyhow!("Unknown tool: {}", name)),
        }
    }

    async fn handle_generate_image(&self, arguments: &serde_json::Value) -> Result<CallToolResult> {
        info!("Handling generate_image request");

        // 引数のパース
        let prompt = arguments
            .get("prompt")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: prompt"))?
            .to_string();

        let model = arguments
            .get("model")
            .and_then(|v| v.as_str())
            .map(GeminiModel::try_from)
            .transpose()
            .map_err(|e| anyhow::anyhow!("Invalid model: {}", e))?
            .unwrap_or_else(|| GeminiModel::from(self.default_model.clone()));

        let request = ImageGenerationRequest::new(prompt).with_model(model);

        // ユースケースを実行
        let image = self.use_case.execute(request).await.map_err(|e| {
            error!("Image generation failed: {}", e);
            anyhow::anyhow!("Image generation failed: {}", e)
        })?;

        // 結果をbase64エンコードして返す
        use base64::Engine;
        let base64_data = base64::engine::general_purpose::STANDARD.encode(&image.data);

        Ok(CallToolResult {
            content: vec![Content::Text {
                text: format!(
                    r#"{{
                        "image_data": "{}",
                        "model": "{}",
                        "generated_at": "{}",
                        "size_bytes": {}
                    }}"#,
                    base64_data,
                    image.model,
                    image.generated_at.to_rfc3339(),
                    image.data.len()
                ),
            }],
            is_error: false,
        })
    }
}
