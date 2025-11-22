use crate::config::Config;
use crate::infrastructure::mcp::{JsonRpcError, JsonRpcRequest, JsonRpcResponse, McpServer};
use anyhow::Result;
use tracing::{error, info};

/// MCPリクエストハンドラー
pub struct RequestHandler {
    server: McpServer,
    config: Config,
}

impl RequestHandler {
    pub fn new(server: McpServer) -> Self {
        Self {
            server,
            config: Config::from_env(),
        }
    }

    pub fn with_config(server: McpServer, config: Config) -> Self {
        Self { server, config }
    }

    /// JSON-RPCリクエストを処理
    pub async fn handle_jsonrpc_request(&self, request: JsonRpcRequest) -> Result<JsonRpcResponse> {
        let id = request.id.clone();

        match request.method.as_str() {
            "initialize" => {
                info!("Handling initialize request");
                Ok(JsonRpcResponse {
                    jsonrpc: self.config.jsonrpc_version().to_string(),
                    id,
                    result: Some(serde_json::json!({
                        "protocolVersion": "2024-11-05",
                        "capabilities": {
                            "tools": {}
                        },
                        "serverInfo": {
                            "name": "google-gemini-image-creator",
                            "version": "0.1.0"
                        }
                    })),
                    error: None,
                })
            }
            "tools/list" => {
                info!("Handling tools/list request");
                let tools = self.server.list_tools();
                Ok(JsonRpcResponse {
                    jsonrpc: self.config.jsonrpc_version().to_string(),
                    id,
                    result: Some(serde_json::json!({
                        "tools": tools
                    })),
                    error: None,
                })
            }
            "tools/call" => {
                let params = request
                    .params
                    .ok_or_else(|| anyhow::anyhow!("Missing params for tools/call"))?;

                let name = params
                    .get("name")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing 'name' in params"))?;

                let arguments = params
                    .get("arguments")
                    .cloned()
                    .unwrap_or(serde_json::json!({}));

                info!("Handling tools/call request: {}", name);

                match self.server.call_tool(name, &arguments).await {
                    Ok(result) => Ok(JsonRpcResponse {
                        jsonrpc: self.config.jsonrpc_version().to_string(),
                        id,
                        result: Some(serde_json::json!({
                            "content": result.content,
                            "isError": result.is_error
                        })),
                        error: None,
                    }),
                    Err(e) => {
                        error!("Tool call failed: {}", e);
                        Ok(JsonRpcResponse {
                            jsonrpc: self.config.jsonrpc_version().to_string(),
                            id,
                            result: None,
                            error: Some(JsonRpcError {
                                code: self.config.jsonrpc_error_codes.internal_error,
                                message: format!("Internal error: {}", e),
                                data: None,
                            }),
                        })
                    }
                }
            }
            _ => Ok(JsonRpcResponse {
                jsonrpc: self.config.jsonrpc_version().to_string(),
                id,
                result: None,
                error: Some(JsonRpcError {
                    code: self.config.jsonrpc_error_codes.method_not_found,
                    message: format!("Method not found: {}", request.method),
                    data: None,
                }),
            }),
        }
    }
}
