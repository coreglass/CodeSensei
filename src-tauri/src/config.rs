// OpenCode 配置管理
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

// ===== 配置数据结构 =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenCodeConfig {
    /// OpenCode Server 地址
    pub server_url: String,

    /// 认证用户名
    pub username: String,

    /// 认证密码（可选）
    pub password: Option<String>,

    /// 默认 Provider ID
    pub default_provider: Option<String>,

    /// 默认 Model ID
    pub default_model: Option<String>,
}

impl Default for OpenCodeConfig {
    fn default() -> Self {
        Self {
            server_url: "http://localhost:4096".to_string(),
            username: "opencode".to_string(),
            password: None,
            default_provider: None,
            default_model: None,
        }
    }
}

// ===== 配置管理器 =====

pub struct ConfigManager {
    config_path: PathBuf,
}

impl ConfigManager {
    /// 创建配置管理器
    pub fn new() -> Result<Self, String> {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("CodeSensei");

        // 确保配置目录存在
        fs::create_dir_all(&config_dir)
            .map_err(|e| format!("无法创建配置目录: {}", e))?;

        let config_path = config_dir.join("opencode-config.json");

        Ok(Self { config_path })
    }

    /// 读取配置
    pub fn load_config(&self) -> OpenCodeConfig {
        if !self.config_path.exists() {
            // 配置文件不存在，返回默认配置
            let default_config = OpenCodeConfig::default();
            // 自动保存默认配置
            let _ = self.save_config(&default_config);
            return default_config;
        }

        fs::read_to_string(&self.config_path)
            .ok()
            .and_then(|content| serde_json::from_str(&content).ok())
            .unwrap_or_default()
    }

    /// 保存配置
    pub fn save_config(&self, config: &OpenCodeConfig) -> Result<(), String> {
        let content = serde_json::to_string_pretty(config)
            .map_err(|e| format!("序列化配置失败: {}", e))?;

        fs::write(&self.config_path, content)
            .map_err(|e| format!("写入配置文件失败: {}", e))?;

        println!("✅ OpenCode 配置已保存到: {:?}", self.config_path);
        Ok(())
    }

    /// 更新 Server URL
    pub fn update_server_url(&self, url: String) -> Result<(), String> {
        let mut config = self.load_config();
        config.server_url = url.trim().trim_end_matches('/').to_string();
        self.save_config(&config)
    }

    /// 更新认证信息
    pub fn update_auth(&self, username: String, password: Option<String>) -> Result<(), String> {
        let mut config = self.load_config();
        config.username = username;
        config.password = password;
        self.save_config(&config)
    }

    /// 更新默认 Provider
    pub fn update_provider(&self, provider: Option<String>, model: Option<String>) -> Result<(), String> {
        let mut config = self.load_config();
        config.default_provider = provider;
        config.default_model = model;
        self.save_config(&config)
    }

    /// 获取配置文件路径（用于调试）
    pub fn config_path(&self) -> &PathBuf {
        &self.config_path
    }
}

// ===== 全局单例 =====

use std::sync::Mutex;

// 使用简单的全局状态管理
pub static CONFIG_MANAGER: Mutex<Option<ConfigManager>> = Mutex::new(None);

/// 初始化全局配置管理器
pub fn init_config_manager() -> Result<(), String> {
    let manager = ConfigManager::new()?;
    CONFIG_MANAGER.lock().unwrap().replace(manager);
    Ok(())
}

/// 获取全局配置
pub fn get_config() -> OpenCodeConfig {
    CONFIG_MANAGER
        .lock()
        .unwrap()
        .as_ref()
        .map(|manager| manager.load_config())
        .unwrap_or_default()
}

/// 保存全局配置
pub fn save_config(config: &OpenCodeConfig) -> Result<(), String> {
    CONFIG_MANAGER
        .lock()
        .unwrap()
        .as_ref()
        .ok_or_else(|| "配置管理器未初始化".to_string())?
        .save_config(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = OpenCodeConfig::default();
        assert_eq!(config.server_url, "http://localhost:4096");
        assert_eq!(config.username, "opencode");
        assert!(config.password.is_none());
    }

    #[test]
    fn test_config_serialization() {
        let config = OpenCodeConfig {
            server_url: "http://localhost:4096".to_string(),
            username: "opencode".to_string(),
            password: Some("test123".to_string()),
            default_provider: Some("openai".to_string()),
            default_model: Some("gpt-4".to_string()),
        };

        let json = serde_json::to_string(&config).unwrap();
        let parsed: OpenCodeConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.server_url, config.server_url);
        assert_eq!(parsed.username, config.username);
        assert_eq!(parsed.password, config.password);
    }
}
