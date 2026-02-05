use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;
use tauri::{Emitter, AppHandle};
use tokio::process::Command as TokioCommand;

// ===== 业务结构 =====

#[derive(Debug, Serialize, Deserialize)]
pub struct RequirementUpdateRequest {
    pub user_input: String,
    pub project_path: String,
    pub current_content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateFilesRequest {
    pub user_input: String,
    pub project_path: String,
    pub requirement_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentResponse {
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_modified: Option<String>,
    #[serde(rename = "document_content", skip_serializing_if = "Option::is_none")]
    pub document_content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

// ===== Claude Agent Node.js 集成 =====

pub struct ClaudeAgentNode {
    plugin_path: PathBuf,
}

impl ClaudeAgentNode {
    pub fn new() -> Self {
        let plugin_path = std::env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .join("plugins")
            .join("claude-agent")
            .join("index.cjs");

        Self { plugin_path }
    }

    /// 使用 Node.js 插件更新需求文档
    pub async fn update_requirement(
        &self,
        app: &AppHandle,
        req: RequirementUpdateRequest,
    ) -> Result<AgentResponse, String> {
        // 输出发送给 Node.js 的消息
        println!("========== 发送给 Claude Agent (Node.js) 的消息 ==========");
        println!("用户输入: {}", req.user_input);
        println!("项目路径: {}", req.project_path);
        println!("===========================================================");

        // 发送事件到前端
        let _ = app.emit("claude-message", serde_json::json!({
            "mode": "requirement",
            "user_input": req.user_input,
            "project_path": req.project_path
        }));

        // 准备参数
        let args = serde_json::json!({
            "prompt": req.user_input,
            "projectPath": req.project_path,
            "currentContent": req.current_content
        });

        // 调用 Node.js 插件
        let result = self.call_plugin("updateRequirement", &args).await?;

        let response: AgentResponse = serde_json::from_value(result)
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        Ok(response)
    }

    /// 使用 Node.js 插件创建/修改文件
    pub async fn create_files(
        &self,
        app: &AppHandle,
        req: CreateFilesRequest,
    ) -> Result<AgentResponse, String> {
        // 输出发送给 Node.js 的消息
        println!("========== 发送给 Claude Agent (Node.js) 的消息 ==========");
        println!("用户输入: {}", req.user_input);
        println!("项目路径: {}", req.project_path);
        println!("需求文档: {}", req.requirement_path);
        println!("==============================================================");

        // 发送事件到前端
        let _ = app.emit("claude-message", serde_json::json!({
            "mode": "create",
            "user_input": req.user_input,
            "project_path": req.project_path,
            "requirement_path": req.requirement_path
        }));

        // 发送进度事件
        let _ = app.emit("agent-progress", serde_json::json!({
            "stage": "analyzing",
            "message": "正在分析项目结构和需求..."
        }));

        // 准备参数
        let args = serde_json::json!({
            "prompt": req.user_input,
            "projectPath": req.project_path,
            "requirementPath": req.requirement_path
        });

        // 调用 Node.js 插件
        let result = self.call_plugin("createFiles", &args).await?;

        let response: AgentResponse = serde_json::from_value(result)
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        Ok(response)
    }

    /// 调用 Node.js 插件的辅助方法
    async fn call_plugin(&self, function: &str, args: &Value) -> Result<Value, String> {
        use std::io::Write;

        // 创建临时文件传递参数
        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join(format!("claude_agent_{}.json", uuid::Uuid::new_v4()));

        // 写入参数到临时文件
        let args_json = serde_json::to_string(args).unwrap();
        let mut file = std::fs::File::create(&temp_file)
            .map_err(|e| format!("Failed to create temp file: {}", e))?;
        file.write_all(args_json.as_bytes())
            .map_err(|e| format!("Failed to write temp file: {}", e))?;

        // 构建调用脚本
        let plugin_path = self.plugin_path.display().to_string().replace('\\', "/");
        let temp_path = temp_file.display().to_string().replace('\\', "/");

        let script = format!(
            "const plugin = require('{}'); const args = require('{}'); plugin[\"{}\"](args).then(r => console.log(JSON.stringify(r))).catch(e => console.error(JSON.stringify({{error: e.message}})));",
            plugin_path, temp_path, function
        );

        // 执行 Node.js 脚本
        let output = TokioCommand::new("node")
            .arg("-e")
            .arg(&script)
            .output()
            .await
            .map_err(|e| format!("Failed to execute Node.js: {}", e))?;

        // 清理临时文件
        let _ = std::fs::remove_file(&temp_file);

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Node.js execution failed: {}", error));
        }

        // 解析输出
        let stdout = String::from_utf8_lossy(&output.stdout);
        let result: Value = serde_json::from_str(&stdout)
            .map_err(|e| format!("Failed to parse Node.js output: {} - Output: {}", e, stdout))?;

        Ok(result)
    }
}
