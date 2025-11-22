use std::env;

/// アプリケーション設定
pub struct Config {
    /// JSON-RPCバージョン
    pub jsonrpc_version: String,
    /// Gemini APIベースURL
    pub gemini_api_base_url: String,
    /// デフォルトGeminiモデル名
    pub gemini_default_model: String,
    /// 許可されたGeminiモデルリスト（カンマ区切り）
    pub gemini_allowed_models: Vec<String>,
    /// プロンプトの最大長
    pub max_prompt_length: usize,
    /// JSON-RPCエラーコード
    pub jsonrpc_error_codes: JsonRpcErrorCodes,
}

/// JSON-RPCエラーコード設定
pub struct JsonRpcErrorCodes {
    /// パースエラー
    pub parse_error: i32,
    /// 無効なリクエスト
    pub invalid_request: i32,
    /// メソッドが見つからない
    pub method_not_found: i32,
    /// 内部エラー
    pub internal_error: i32,
}

impl Default for JsonRpcErrorCodes {
    fn default() -> Self {
        Self {
            parse_error: -32700,
            invalid_request: -32600,
            method_not_found: -32601,
            internal_error: -32603,
        }
    }
}

impl Config {
    /// 環境変数から設定を読み込む
    pub fn from_env() -> Self {
        Self {
            jsonrpc_version: env::var("JSONRPC_VERSION").unwrap_or_else(|_| "2.0".to_string()),
            gemini_api_base_url: env::var("GEMINI_API_BASE_URL")
                .unwrap_or_else(|_| "https://generativelanguage.googleapis.com/v1beta".to_string()),
            gemini_default_model: env::var("GEMINI_DEFAULT_MODEL")
                .unwrap_or_else(|_| "gemini-2.5-flash-image".to_string()),
            gemini_allowed_models: env::var("GEMINI_ALLOWED_MODELS")
                .ok()
                .map(|s| {
                    s.split(',')
                        .map(|m| m.trim().to_string())
                        .filter(|m| !m.is_empty())
                        .collect()
                })
                .unwrap_or_default(),
            max_prompt_length: env::var("MAX_PROMPT_LENGTH")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(10000),
            jsonrpc_error_codes: JsonRpcErrorCodes::default(),
        }
    }

    /// JSON-RPCバージョンを取得
    pub fn jsonrpc_version(&self) -> &str {
        &self.jsonrpc_version
    }

    /// Gemini APIベースURLを取得
    pub fn gemini_api_base_url(&self) -> &str {
        &self.gemini_api_base_url
    }

    /// デフォルトGeminiモデル名を取得
    pub fn gemini_default_model(&self) -> &str {
        &self.gemini_default_model
    }

    /// 許可されたGeminiモデルリストを取得
    pub fn gemini_allowed_models(&self) -> &[String] {
        &self.gemini_allowed_models
    }

    /// プロンプトの最大長を取得
    pub fn max_prompt_length(&self) -> usize {
        self.max_prompt_length
    }
}
