# OpenCode 集成方案

## 一、架构设计

### 当前架构（Claude）
```
前端（Vue）
  ↓ Tauri IPC
Rust 后端
  ↓ tokio::process
Node.js 插件
  ↓ Claude Agent SDK
Claude API
```

### 新架构（OpenCode）
```
前端（Vue）
  ↓ Tauri IPC
Rust 后端
  ↓ HTTP (reqwest)
OpenCode Server
  ↓ AI Provider
OpenAI/Anthropic/Claude/etc
```

**优势：**
- ✅ 完全开源，无依赖风险
- ✅ 支持多种 AI Provider（OpenAI、Anthropic、本地模型等）
- ✅ HTTP 通信更简单可靠
- ✅ 无需 Node.js 中间层
- ✅ 可复用 OpenCode 的强大功能

---

## 二、核心 API 集成

### 2.1 会话管理

```rust
// 创建会话
POST http://localhost:4096/session
Body: { "title": "CodeSensei Session" }
Response: { "id": "session-uuid", "title": "...", ... }

// 发送消息
POST http://localhost:4096/session/{id}/message
Body: {
  "parts": [
    { "type": "text", "text": "用户输入的内容" }
  ]
}
Response: {
  "info": { "id": "msg-id", ... },
  "parts": [
    { "type": "text", "text": "AI 响应" }
  ]
}
```

### 2.2 项目集成

OpenCode 天然支持项目概念，我们可以：
1. 让 OpenCode 管理项目
2. 或者在 CodeSensei 中管理项目，通过 API 告诉 OpenCode

### 2.3 文件操作

OpenCode 提供了丰富的文件 API：
- `GET /file?path=<path>` - 列出文件
- `GET /file/content?path=<path>` - 读取文件
- OpenCode 会自动处理文件读写（通过工具调用）

---

## 三、实现计划

### 阶段 1：基础集成（1-2天）

**目标：** 替换 Claude 为 OpenCode，保持现有功能

#### 步骤 1：添加配置
```rust
// src-tauri/src/config.rs
pub struct OpenCodeConfig {
    pub server_url: String,        // http://localhost:4096
    pub password: Option<String>,  // Basic Auth 密码
    pub username: String,          // 默认 "opencode"
}
```

#### 步骤 2：实现 OpenCode 客户端
```rust
// src-tauri/src/opencode.rs
use reqwest::Client;

pub struct OpenCodeClient {
    client: Client,
    base_url: String,
    auth: Option<String>,
}

impl OpenCodeClient {
    pub async fn create_session(&self, title: &str) -> Result<Session, Error> {
        // POST /session
    }

    pub async fn send_message(&self, session_id: &str, message: &str) -> Result<Message, Error> {
        // POST /session/:id/message
    }

    pub async fn get_file_content(&self, path: &str) -> Result<String, Error> {
        // GET /file/content?path=...
    }
}
```

#### 步骤 3：修改 Tauri 命令
```rust
// src-tauri/src/main.rs
#[tauri::command]
async fn update_requirement_with_opencode(
    req: RequirementUpdateRequest,
) -> Result<AgentResponse, String> {
    let client = OpenCodeClient::new(config);

    // 创建会话
    let session = client.create_session("Requirement Update").await?;

    // 发送消息
    let prompt = format!("根据用户需求更新需求文档：{}", req.user_input);
    let response = client.send_message(&session.id, &prompt).await?;

    Ok(AgentResponse {
        success: true,
        message: "需求文档已更新".to_string(),
        document_content: Some(response.text),
    })
}
```

### 阶段 2：功能增强（2-3天）

**新增功能：**
1. **流式输出**
   - 使用 SSE (Server-Sent Events)
   - `GET /event` 端点
   - 实时显示 AI 响应

2. **工具调用可视化**
   - 监听 OpenCode 的工具调用事件
   - 显示正在读取/写入的文件
   - 显示执行的操作

3. **会话历史**
   - 利用 OpenCode 的 Session 管理
   - 支持多轮对话
   - 会话持久化

### 阶段 3：高级功能（可选）

1. **Agent 模式**
   - 使用 OpenCode 的 Agent 系统
   - `GET /agent` 列出可用 agents
   - 支持不同场景的专用 agents

2. **多 AI Provider**
   - 让用户选择 AI Provider
   - OpenAI、Anthropic、本地模型等
   - 在设置中配置

3. **项目管理增强**
   - 与 OpenCode 项目同步
   - 共享 VCS 信息
   - 智能上下文感知

---

## 四、配置界面更新

### 设置对话框新增项

```vue
<el-form-item label="OpenCode Server">
  <el-input v-model="form.serverUrl" placeholder="http://localhost:4096" />
</el-form-item>

<el-form-item label="Server Password (可选)">
  <el-input v-model="form.serverPassword" type="password" />
</el-form-item>

<el-form-item label="AI Provider">
  <el-select v-model="form.provider">
    <el-option label="OpenAI" value="openai" />
    <el-option label="Anthropic" value="anthropic" />
    <el-option label="OpenAI Compatible" value="openai-compatible" />
  </el-select>
</el-form-item>

<el-form-item label="Model">
  <el-select v-model="form.model">
    <el-option label="GPT-4" value="gpt-4" />
    <el-option label="GPT-4o" value="gpt-4o" />
    <el-option label="Claude 3.5 Sonnet" value="claude-3-5-sonnet-20241022" />
  </el-select>
</el-form-item>
```

---

## 五、迁移清单

### 需要删除的文件/依赖
- ❌ `plugins/claude-agent/index.cjs` - 整个插件目录
- ❌ `@anthropic-ai/claude-agent-sdk` npm 依赖
- ❌ `src-tauri/src/claude_node.rs` - Node.js 桥接代码
- ❌ `scripts/copy-plugins.cjs` - 插件复制脚本

### 需要新增的文件
- ✅ `src-tauri/src/opencode.rs` - OpenCode 客户端
- ✅ `src-tauri/src/config.rs` - 配置管理（新增 OpenCode 配置）

### 需要修改的文件
- 🔄 `src-tauri/Cargo.toml` - 移除 uuid 依赖（如果不用），保留 reqwest
- 🔄 `src-tauri/src/main.rs` - 修改 AI 相关命令
- 🔄 `src-tauri/src/claude.rs` - 重命名为 `ai_config.rs` 或删除
- 🔄 `src/components/SettingsDialog.vue` - 添加 OpenCode 配置
- 🔄 `src/api/tauri.js` - 可能需要调整 API 调用

---

## 六、测试计划

### 本地测试
1. 启动 OpenCode Server
   ```bash
   opencode serve --port 4096
   ```

2. 测试 API 连接
   ```bash
   curl http://localhost:4096/global/health
   ```

3. 测试创建会话
   ```bash
   curl -X POST http://localhost:4096/session \
     -H "Content-Type: application/json" \
     -d '{"title": "Test Session"}'
   ```

4. 测试发送消息
   ```bash
   curl -X POST http://localhost:4096/session/{id}/message \
     -H "Content-Type: application/json" \
     -d '{"parts": [{"type": "text", "text": "Hello"}]}'
   ```

### 集成测试
1. ✅ 配置 OpenCode Server 地址
2. ✅ 创建项目
3. ✅ 测试对话模式
4. ✅ 测试需求文档模式
5. ✅ 测试创建文件模式
6. ✅ 测试错误处理
7. ✅ 测试认证（如果设置了密码）

---

## 七、部署说明

### 用户端部署

用户需要：
1. 安装 OpenCode
   ```bash
   npm install -g @opencode/opencode
   ```

2. 启动 OpenCode Server
   ```bash
   opencode serve --port 4096
   ```

3. 配置 AI Provider
   - 在 OpenCode 中配置 OpenAI/Anthropic API Key
   - 或使用本地模型（通过 OpenAI Compatible API）

### 应用端配置
- 在 CodeSensei 设置中输入 OpenCode Server 地址
- 如果设置了密码，输入密码
- 选择 AI Provider 和模型

---

## 八、优势对比

| 特性 | Claude (旧方案) | OpenCode (新方案) |
|------|----------------|------------------|
| 开源 | ❌ 否 | ✅ 是 |
| AI Provider | 仅 Anthropic | 多种支持 |
| 部署方式 | 云 API | 本地/云端 |
| 成本 | 按使用付费 | 灵活（可用本地模型） |
| 功能扩展 | 受限于 API | 完全可控 |
| 数据隐私 | 发送到 Anthropic | 可本地处理 |
| 工具调用 | 固定工具集 | 自定义工具 |
| 项目管理 | 简单 | 强大（VCS、多项目） |

---

## 九、风险与注意事项

### 需要注意
1. **OpenCode Server 必须运行**
   - 用户需要先启动 OpenCode Server
   - 应用需要检测 Server 是否可用
   - 提供友好的错误提示

2. **API 兼容性**
   - OpenCode API 可能会更新
   - 需要关注版本变化
   - 考虑版本兼容性检测

3. **认证管理**
   - Basic Auth 密码存储
   - AI Provider API Key 由 OpenCode 管理
   - 无需在 CodeSensei 中存储敏感信息

### 优势
1. ✅ **零依赖风险** - OpenCode 是开源的
2. ✅ **成本可控** - 可使用本地模型
3. ✅ **功能强大** - OpenCode 持续更新
4. ✅ **社区支持** - 活跃的开源社区

---

## 十、下一步行动

建议按以下顺序执行：

1. **立即开始**（基础集成）
   - [ ] 实现 `opencode.rs` 客户端
   - [ ] 修改 Tauri 命令
   - [ ] 更新设置界面
   - [ ] 本地测试

2. **短期**（1周内）
   - [ ] 添加流式输出
   - [ ] 完善错误处理
   - [ ] 添加 Server 状态检测
   - [ ] 编写文档

3. **中期**（2-4周）
   - [ ] 会话历史管理
   - [ ] 工具调用可视化
   - [ ] 多 AI Provider 支持
   - [ ] 性能优化

---

## 十一、代码示例

### 完整的 OpenCode 客户端实现

```rust
// src-tauri/src/opencode.rs
use reqwest::{Client, header};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MessagePart {
    #[serde(rename = "type")]
    pub part_type: String,
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub info: MessageInfo,
    pub parts: Vec<MessagePart>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageInfo {
    pub id: String,
    #[serde(default)]
    pub role: String,
}

pub struct OpenCodeClient {
    client: Client,
    base_url: String,
    auth_header: Option<String>,
}

impl OpenCodeClient {
    pub fn new(server_url: String, password: Option<String>) -> Self {
        let mut client = Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .expect("Failed to create HTTP client");

        let auth_header = password.map(|p| {
            let credentials = format!("opencode:{}", p);
            let encoded = base64::encode(credentials);
            format!("Basic {}", encoded)
        });

        Self {
            client,
            base_url: server_url,
            auth_header,
        }
    }

    pub async fn health_check(&self) -> Result<bool, String> {
        let url = format!("{}/global/health", self.base_url);
        let response = self.client.get(&url)
            .send()
            .await
            .map_err(|e| format!("Failed to connect: {}", e))?;

        Ok(response.status().is_success())
    }

    pub async fn create_session(&self, title: &str) -> Result<Session, String> {
        let url = format!("{}/session", self.base_url);

        let body = serde_json::json!({ "title": title });

        let mut request = self.client.post(&url).json(&body);

        if let Some(auth) = &self.auth_header {
            request = request.header(header::AUTHORIZATION, auth);
        }

        let response = request.send().await
            .map_err(|e| format!("Failed to create session: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Server returned status: {}", response.status()));
        }

        response.json::<Session>().await
            .map_err(|e| format!("Failed to parse response: {}", e))
    }

    pub async fn send_message(
        &self,
        session_id: &str,
        message: &str,
    ) -> Result<Message, String> {
        let url = format!("{}/session/{}/message", self.base_url, session_id);

        let body = serde_json::json!({
            "parts": [
                { "type": "text", "text": message }
            ]
        });

        let mut request = self.client.post(&url).json(&body);

        if let Some(auth) = &self.auth_header {
            request = request.header(header::AUTHORIZATION, auth);
        }

        let response = request.send().await
            .map_err(|e| format!("Failed to send message: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Server returned status: {}", response.status()));
        }

        response.json::<Message>().await
            .map_err(|e| format!("Failed to parse response: {}", e))
    }

    pub async fn delete_session(&self, session_id: &str) -> Result<bool, String> {
        let url = format!("{}/session/{}", self.base_url, session_id);

        let mut request = self.client.delete(&url);

        if let Some(auth) = &self.auth_header {
            request = request.header(header::AUTHORIZATION, auth);
        }

        let response = request.send().await
            .map_err(|e| format!("Failed to delete session: {}", e))?;

        Ok(response.status().is_success())
    }
}
```

---

## 附录：资源链接

- [OpenCode Server 官方文档](https://opencode.ai/docs/server/)
- [OpenCode 中文文档](https://opencodecn.com/docs/server)
- [OpenCode GitHub](https://github.com/sst/opencode)
- [OpenAPI 规范](http://localhost:4096/doc) (本地运行后访问)

---

**文档版本:** v1.0
**最后更新:** 2025-02-08
**作者:** CodeSensei Team
