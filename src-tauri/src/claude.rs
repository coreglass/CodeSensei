// ===== API Key 管理 =====

pub fn get_api_key() -> Result<String, String> {
    std::env::var("ANTHROPIC_API_KEY")
        .map_err(|_| "ANTHROPIC_API_KEY environment variable not found".to_string())
}

pub fn save_api_key(_api_key: String) -> Result<(), String> {
    // Claude Code API Key 存储在 Claude 的配置文件中
    // 这里我们不需要手动保存，因为 Claude Code 会管理自己的配置
    Ok(())
}
