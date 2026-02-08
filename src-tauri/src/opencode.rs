// OpenCode Server HTTP 客户端
use reqwest::{Client, header};
use serde::{Deserialize, Serialize};
use std::time::Duration;

// ===== 业务结构 =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub provider_id: Option<String>,
    #[serde(default)]
    pub model_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessagePart {
    #[serde(rename = "type")]
    pub part_type: String,
    pub text: Option<String>,
    #[serde(default)]
    pub reasoning: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageInfo {
    pub id: String,
    #[serde(default)]
    pub role: String,
    #[serde(default)]
    pub created: String,
    #[serde(default)]
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub info: MessageInfo,
    pub parts: Vec<MessagePart>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SendMessageRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    pub parts: Vec<MessagePart>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub healthy: bool,
    pub version: String,
}

// ===== AI Provider 相关结构 =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provider {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub display_name: String,
    #[serde(default)]
    pub homepage: String,
    /// models 可能是字符串数组或对象（map），使用 Value 灵活处理
    #[serde(default)]
    pub models: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderListResponse {
    #[serde(default)]
    pub all: Vec<Provider>,
    #[serde(default)]
    pub default: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigProvidersResponse {
    #[serde(default)]
    pub providers: Vec<Provider>,
}

// ===== 客户端实现 =====

pub struct OpenCodeClient {
    client: Client,
    base_url: String,
    auth_header: Option<String>,
}

impl OpenCodeClient {
    pub fn new(server_url: String, username: String, password: Option<String>) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(120))
            .connect_timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to create HTTP client");

        // 构建 Basic Auth header
        let auth_header = if let Some(pwd) = password {
            let credentials = format!("{}:{}", username, pwd);
            use base64::Engine;
            let encoded = base64::engine::general_purpose::STANDARD.encode(credentials);
            Some(format!("Basic {}", encoded))
        } else {
            None
        };

        Self {
            client,
            base_url: server_url.trim_end_matches('/').to_string(),
            auth_header,
        }
    }

    /// 检查服务器健康状态
    pub async fn health_check(&self) -> Result<HealthResponse, String> {
        let url = format!("{}/global/health", self.base_url);

        let mut request = self.client.get(&url);

        if let Some(auth) = &self.auth_header {
            request = request.header(header::AUTHORIZATION, auth);
        }

        let response = request
            .send()
            .await
            .map_err(|e| format!("无法连接到 OpenCode Server: {}\n请检查 Server 是否运行，地址是否正确", e))?;

        if !response.status().is_success() {
            return Err(format!("Server 返回错误状态: {}", response.status()));
        }

        response
            .json::<HealthResponse>()
            .await
            .map_err(|e| format!("解析响应失败: {}", e))
    }

    /// 获取所有可用的 AI Providers
    pub async fn get_providers(&self) -> Result<Vec<Provider>, String> {
        let url = format!("{}/provider", self.base_url);

        let mut request = self.client.get(&url);

        if let Some(auth) = &self.auth_header {
            request = request.header(header::AUTHORIZATION, auth);
        }

        let response = request
            .send()
            .await
            .map_err(|e| format!("获取 Providers 失败: {}", e))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "无法读取错误信息".to_string());
            return Err(format!("Server 返回错误 ({}): {}", status, error_text));
        }

        // 先获取原始文本，以便在解析失败时查看内容
        let response_text = response
            .text()
            .await
            .map_err(|e| format!("读取响应文本失败: {}", e))?;

        // 尝试解析，如果失败则显示原始响应
        match serde_json::from_str::<ProviderListResponse>(&response_text) {
            Ok(result) => Ok(result.all),
            Err(e) => {
                // 打印实际收到的 JSON，方便调试
                eprintln!("解析 Providers 失败，收到的响应:\n{}", response_text);
                Err(format!("解析 Providers 失败: {}\n响应内容: {}", e, response_text))
            }
        }
    }

    /// 获取配置文件中的 Providers（包含模型列表）
    pub async fn get_config_providers(&self) -> Result<Vec<Provider>, String> {
        let url = format!("{}/config/providers", self.base_url);

        let mut request = self.client.get(&url);

        if let Some(auth) = &self.auth_header {
            request = request.header(header::AUTHORIZATION, auth);
        }

        let response = request
            .send()
            .await
            .map_err(|e| format!("获取配置 Providers 失败: {}", e))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "无法读取错误信息".to_string());
            return Err(format!("Server 返回错误 ({}): {}", status, error_text));
        }

        // 先获取原始文本，以便在解析失败时查看内容
        let response_text = response
            .text()
            .await
            .map_err(|e| format!("读取响应文本失败: {}", e))?;

        // 尝试解析，如果失败则显示原始响应
        match serde_json::from_str::<ConfigProvidersResponse>(&response_text) {
            Ok(result) => Ok(result.providers),
            Err(e) => {
                // 打印实际收到的 JSON，方便调试
                eprintln!("解析配置 Providers 失败，收到的响应:\n{}", response_text);
                Err(format!("解析配置 Providers 失败: {}\n响应内容: {}", e, response_text))
            }
        }
    }

    /// 创建新会话
    pub async fn create_session(
        &self,
        title: &str,
        provider_id: Option<String>,
        model_id: Option<String>,
    ) -> Result<Session, String> {
        let url = format!("{}/session", self.base_url);

        let mut body = serde_json::Map::new();
        body.insert("title".to_string(), serde_json::Value::String(title.to_string()));

        if let Some(provider) = provider_id {
            body.insert("providerId".to_string(), serde_json::Value::String(provider));
        }

        if let Some(model) = model_id {
            body.insert("modelId".to_string(), serde_json::Value::String(model));
        }

        let mut request = self.client.post(&url).json(&body);

        if let Some(auth) = &self.auth_header {
            request = request.header(header::AUTHORIZATION, auth);
        }

        let response = request
            .send()
            .await
            .map_err(|e| format!("创建会话失败: {}", e))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "无法读取错误信息".to_string());
            return Err(format!("Server 返回错误 ({}): {}", status, error_text));
        }

        response
            .json::<Session>()
            .await
            .map_err(|e| format!("解析会话响应失败: {}", e))
    }

    /// 发送消息到会话
    pub async fn send_message(
        &self,
        session_id: &str,
        message: &str,
        agent: Option<String>,
        model: Option<String>,
    ) -> Result<Message, String> {
        let url = format!("{}/session/{}/message", self.base_url, session_id);

        let body = SendMessageRequest {
            message_id: None,
            agent,
            model,
            parts: vec![MessagePart {
                part_type: "text".to_string(),
                text: Some(message.to_string()),
                reasoning: None,
            }],
        };

        let mut request = self.client.post(&url).json(&body);

        if let Some(auth) = &self.auth_header {
            request = request.header(header::AUTHORIZATION, auth);
        }

        let response = request
            .send()
            .await
            .map_err(|e| format!("发送消息失败: {}", e))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "无法读取错误信息".to_string());
            return Err(format!("Server 返回错误 ({}): {}", status, error_text));
        }

        response
            .json::<Message>()
            .await
            .map_err(|e| format!("解析消息响应失败: {}", e))
    }

    /// 异步发送消息（不等待响应）
    pub async fn send_message_async(
        &self,
        session_id: &str,
        message: &str,
        agent: Option<String>,
        model: Option<String>,
    ) -> Result<(), String> {
        let url = format!("{}/session/{}/prompt_async", self.base_url, session_id);

        let body = SendMessageRequest {
            message_id: None,
            agent,
            model,
            parts: vec![MessagePart {
                part_type: "text".to_string(),
                text: Some(message.to_string()),
                reasoning: None,
            }],
        };

        let mut request = self.client.post(&url).json(&body);

        if let Some(auth) = &self.auth_header {
            request = request.header(header::AUTHORIZATION, auth);
        }

        let response = request
            .send()
            .await
            .map_err(|e| format!("发送异步消息失败: {}", e))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "无法读取错误信息".to_string());
            return Err(format!("Server 返回错误 ({}): {}", status, error_text));
        }

        Ok(())
    }

    /// 获取会话中的消息列表
    pub async fn get_messages(&self, session_id: &str, limit: Option<u32>) -> Result<Vec<Message>, String> {
        let url = if let Some(lim) = limit {
            format!("{}/session/{}/message?limit={}", self.base_url, session_id, lim)
        } else {
            format!("{}/session/{}/message", self.base_url, session_id)
        };

        let mut request = self.client.get(&url);

        if let Some(auth) = &self.auth_header {
            request = request.header(header::AUTHORIZATION, auth);
        }

        let response = request
            .send()
            .await
            .map_err(|e| format!("获取消息列表失败: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Server 返回错误: {}", response.status()));
        }

        response
            .json::<Vec<Message>>()
            .await
            .map_err(|e| format!("解析消息列表失败: {}", e))
    }

    /// 删除会话
    pub async fn delete_session(&self, session_id: &str) -> Result<bool, String> {
        let url = format!("{}/session/{}", self.base_url, session_id);

        let mut request = self.client.delete(&url);

        if let Some(auth) = &self.auth_header {
            request = request.header(header::AUTHORIZATION, auth);
        }

        let response = request
            .send()
            .await
            .map_err(|e| format!("删除会话失败: {}", e))?;

        Ok(response.status().is_success())
    }

    /// 获取可用的工作空间文件
    pub async fn list_files(&self, path: &str) -> Result<Vec<FileNode>, String> {
        let url = format!("{}/file?path={}", self.base_url,
            urlencoding::encode(path));

        let mut request = self.client.get(&url);

        if let Some(auth) = &self.auth_header {
            request = request.header(header::AUTHORIZATION, auth);
        }

        let response = request
            .send()
            .await
            .map_err(|e| format!("获取文件列表失败: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Server 返回错误: {}", response.status()));
        }

        response
            .json::<Vec<FileNode>>()
            .await
            .map_err(|e| format!("解析文件列表失败: {}", e))
    }

    /// 读取文件内容
    pub async fn read_file(&self, path: &str) -> Result<String, String> {
        let url = format!("{}/file/content?path={}", self.base_url,
            urlencoding::encode(path));

        let mut request = self.client.get(&url);

        if let Some(auth) = &self.auth_header {
            request = request.header(header::AUTHORIZATION, auth);
        }

        let response = request
            .send()
            .await
            .map_err(|e| format!("读取文件失败: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Server 返回错误: {}", response.status()));
        }

        #[derive(Deserialize)]
        struct FileContent {
            content: String,
        }

        let file_content: FileContent = response
            .json()
            .await
            .map_err(|e| format!("解析文件内容失败: {}", e))?;

        Ok(file_content.content)
    }
}

// ===== 辅助结构 =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileNode {
    pub path: String,
    #[serde(rename = "type")]
    pub node_type: String, // "file" or "directory"
}

// ===== 测试辅助函数 =====

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = OpenCodeClient::new(
            "http://localhost:4096".to_string(),
            "opencode".to_string(),
            None,
        );
        assert_eq!(client.base_url, "http://localhost:4096");
    }

    #[test]
    fn test_auth_header() {
        let client = OpenCodeClient::new(
            "http://localhost:4096".to_string(),
            "opencode".to_string(),
            Some("test123".to_string()),
        );
        assert!(client.auth_header.is_some());
        assert!(client.auth_header.unwrap().starts_with("Basic "));
    }
}
