use google_gemini_image_creator::infrastructure::mcp::McpServer;

#[test]
fn test_list_tools() {
    let server = McpServer::new("test-key".to_string());
    let tools = server.list_tools();
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].name, "generate_image");
}
