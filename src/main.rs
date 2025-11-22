// ライブラリとして公開されているモジュールを使用
use google_gemini_image_creator::domain;
use google_gemini_image_creator::infrastructure;
use google_gemini_image_creator::presentation;

use anyhow::Result;
use infrastructure::mcp::McpServer;
use presentation::RequestHandler;
use std::env;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<()> {
    // ログの初期化
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    info!("Starting Google Gemini Image Creator MCP Server");

    // 環境変数からAPIキーを取得
    let api_key = env::var("GEMINI_API_KEY")
        .map_err(|_| anyhow::anyhow!("GEMINI_API_KEY environment variable is required"))?;

    // GeminiModelの環境変数を初期化
    domain::GeminiModel::init_from_env();

    // MCPサーバーの初期化
    let server = McpServer::new(api_key);
    let handler = RequestHandler::new(server);

    info!("MCP Server initialized");

    // MCPサーバーを起動（実際のMCPプロトコル実装に合わせて調整が必要）
    // ここでは標準入出力を使用したMCPサーバーの実装例を示す
    run_mcp_server(handler).await?;

    Ok(())
}

async fn run_mcp_server(handler: RequestHandler) -> Result<()> {
    use std::io::{self, BufRead, BufReader, Write};

    let stdin = io::stdin();
    let mut reader = BufReader::new(stdin.lock());
    let mut stdout = io::stdout();

    loop {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(0) => break, // EOF
            Ok(_) => {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }

                // JSON-RPCリクエストをパース
                match serde_json::from_str::<infrastructure::mcp::JsonRpcRequest>(trimmed) {
                    Ok(request) => match handler.handle_jsonrpc_request(request).await {
                        Ok(response) => {
                            let response_json = serde_json::to_string(&response)?;
                            writeln!(stdout, "{}", response_json)?;
                            stdout.flush()?;
                        }
                        Err(e) => {
                            error!("Error handling request: {}", e);
                            use google_gemini_image_creator::config::Config;
                            let config = Config::from_env();
                            let error_response = serde_json::json!({
                                "jsonrpc": config.jsonrpc_version(),
                                "error": {
                                    "code": config.jsonrpc_error_codes.internal_error,
                                    "message": format!("Internal error: {}", e)
                                },
                                "id": null
                            });
                            writeln!(stdout, "{}", error_response)?;
                            stdout.flush()?;
                        }
                    },
                    Err(e) => {
                        error!("Failed to parse request: {} - Input: {}", e, trimmed);
                        // パースエラーの場合もJSON-RPCエラーレスポンスを返す
                        use google_gemini_image_creator::config::Config;
                        let config = Config::from_env();
                        let error_response = serde_json::json!({
                            "jsonrpc": config.jsonrpc_version(),
                            "error": {
                                "code": config.jsonrpc_error_codes.parse_error,
                                "message": format!("Parse error: {}", e)
                            },
                            "id": null
                        });
                        writeln!(stdout, "{}", error_response)?;
                        stdout.flush()?;
                    }
                }
            }
            Err(e) => {
                error!("Error reading from stdin: {}", e);
                break;
            }
        }
    }

    Ok(())
}
