use vibagent::tools::{Tool, WebSearchTool};
use anyhow::Result;

#[tokio::test]
async fn test_web_search_tool_instantiation() {
    let tool = WebSearchTool;
    assert_eq!(tool.name(), "web_search");
    assert!(tool.description().contains("Brave Search"));
}

// Note: Full integration testing would require a mock server or a real API key.
// In a real project, we would use a crate like `mockito` to mock the API response.
