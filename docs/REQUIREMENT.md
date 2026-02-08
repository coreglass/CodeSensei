# Code Sensei 详细需求文档

## 项目概述

Code Sensei 是一款基于 Tauri + Vue 3 构建的教学型编程助手，旨在帮助编程学习者通过 AI 辅助的方式理解代码、学习编程概念，并提升实践能力。

## 核心功能

### 1. 项目管理
- **项目创建与编辑**
  - 支持创建本地项目或关联已有代码目录
  - 项目元数据管理（名称、描述、编程语言等）
  - 项目列表展示与快速切换

- **文件树导航**
  - 自动扫描项目文件结构
  - 支持文件夹展开/折叠
  - 智能过滤（自动跳过 node_modules、target 等大型目录）

### 2. 代码编辑
- **Monaco Editor 集成**
  - 语法高亮（支持 100+ 种编程语言）
  - 代码自动补全
  - 括号匹配、缩进引导
  - 多光标编辑

- **文件操作**
  - 创建、编辑、保存文件
  - 文件重命名、删除
  - 文件夹创建

### 3. AI 辅助功能（核心）
- **OpenCode Server 集成**
  - 通过 OpenCode 提供统一的 AI Provider 接口
  - 支持多种 AI 模型（OpenAI、Anthropic、智谱 AI、Moonshot 等）
  - 灵活的模型切换和配置

- **需求文档生成**
  - 根据用户描述自动生成结构化需求文档
  - 支持需求文档的迭代更新

- **代码生成与修改**
  - 基于需求文档生成项目代码
  - 智能代码修改建议
  - 代码风格一致性保证

### 4. 教学特性
- **分步引导**
  - 将复杂任务分解为可执行步骤
  - 实时进度反馈

- **代码解释**
  - 逐行代码注释
  - 核心概念讲解
  - 最佳实践说明

## 技术架构

### 前端技术栈
- **框架**: Vue 3 (Composition API)
- **UI 库**: Element Plus
- **编辑器**: Monaco Editor
- **路由**: Vue Router 5
- **构建工具**: Vite 6

### 后端技术栈
- **框架**: Tauri 2.x
- **语言**: Rust
- **HTTP 客户端**: reqwest
- **配置管理**: 自定义 ConfigManager

### AI 集成
- **OpenCode Server**: 统一的 AI Provider 接口
- **支持模型**:
  - OpenAI (GPT-4, GPT-4o)
  - Anthropic (Claude 3.5 Sonnet)
  - 智谱 AI (GLM-4)
  - Moonshot AI (Kimi 系列)
  - 其他兼容 OpenAI API 的服务

## 开发进度

### Phase 1: 基础框架 ✅
- [x] Tauri + Vue 3 项目搭建
- [x] 基础 UI 布局实现
- [x] 项目 CRUD 功能完成
- [x] Monaco Editor 集成
- [x] 文件树展示与操作

### Phase 2: AI 辅助功能 ✅
- [x] OpenCode Server HTTP 客户端实现
- [x] Provider 配置管理
- [x] 需求文档 AI 生成
- [x] 代码生成与修改功能
- [x] **实时对话过程显示**（异步轮询）
- [x] 实时进度反馈

### Phase 3: 优化与扩展 🚧
- [ ] 代码解释功能
- [ ] 学习路径推荐
- [ ] 代码质量分析
- [ ] UI/UX 优化
- [ ] 测试覆盖

## 最新更新 (2025-02-09)

### 修复问题

#### 1. OpenCode Provider 解析错误修复
**问题**: 刷新可用提供商时报错 "invalid type: map, expected a sequence"

**原因**: OpenCode API 返回的 `models` 字段是对象（包含模型详细信息），但代码期望的是字符串数组

**解决方案**:
- 修改 `src-tauri/src/opencode.rs` 中 `Provider` 结构体
  - 将 `models` 字段类型从 `Vec<String>` 改为 `serde_json::Value`
  - 支持灵活处理数组和对象两种格式

- 更新 `src/components/SettingsDialog.vue`
  - 添加 `getModelCount()` 辅助函数处理 models 计数
  - 更新 `currentModels` computed 属性支持对象格式
  - 更新 `getProviderHint()` 函数兼容两种格式

#### 2. Tauri 应用构建问题修复
**问题**: 直接运行 `cargo build --release` 编译的 exe 启动后显示 "localhost 拒绝连接"

**原因**: 使用 `cargo build` 不会执行前端资源构建步骤，导致应用内没有打包前端静态文件

**解决方案**:
- 使用 `cargo tauri build` 进行完整构建
  - 该命令会自动执行 `beforeBuildCommand: npm run build`
  - 正确将 dist 目录中的前端资源打包到应用中

**正确的构建流程**:
```bash
# 开发模式（带热重载）
npm run tauri dev

# 构建独立应用
npm run tauri build

# 或者手动分步执行
npm run build && cd src-tauri && cargo build --release
```

**构建产物位置**:
- 直接运行: `src-tauri/target/release/code-sensei.exe`
- NSIS 安装包: `src-tauri/target/release/bundle/nsis/Code Sensei_0.1.0_x64-setup.exe`
- MSI 安装包: `src-tauri/target/release/bundle/msi/Code Sensei_0.1.0_x64_en-US.msi`

#### 3. 实时显示 AI Agent 执行过程 ✨
**问题**: 用户反馈 AI Agent 执行时间过长，UI 显示超时错误，但后台任务实际已完成

**原因**:
1. HTTP 请求超时时间设置为 120 秒，复杂任务可能需要更长时间
2. 同步等待模式下，用户无法看到执行过程，体验不佳

**解决方案**:

**方案 1: 增加超时时间**（临时修复）
- 将 HTTP 请求超时从 120 秒增加到 **600 秒（10 分钟）**
- 连接超时从 10 秒增加到 30 秒
- 修改文件: `src-tauri/src/opencode.rs`

**方案 2: 异步 API + 实时轮询**（完整方案）✨
- **后端新增**:
  - `create_files_with_agent_async`: 异步发送消息，立即返回 session_id
  - `get_session_messages`: 轮询获取会话中的消息列表

- **前端实现**:
  - 使用异步 API 发起请求，立即返回
  - 每秒轮询一次，获取新的对话消息
  - 实时显示 AI Agent 与工具的对话过程
  - 自动检测任务完成状态
  - 完成后自动刷新文件树

**用户体验改进**:

*之前*:
```
⏳ 正在处理中... (等待 2-3 分钟)
❌ 调用 Claude Agent 失败: 超时
```

*现在*:
```
🚀 开始处理你的请求...
⏳ AI Agent 正在工作...

📝 用户: 创建一个 main.py 文件
🤖 助手: 好的，我来创建 main.py 文件...
🤖 助手: 我正在读取现有的文件...
🤖 助手: 我正在创建 main.py...

✅ 任务已完成！请查看上方对话了解详细执行过程。
```

**修改文件**:
- `src-tauri/src/opencode.rs`: 增加超时时间
- `src-tauri/src/main.rs`: 添加异步命令
- `src/api/tauri.js`: 添加异步 API 封装
- `src/views/Project.vue`: 实现轮询逻辑和实时显示

## 配置说明

### OpenCode Server 配置
1. **安装 OpenCode**: `npm install -g @opencode/opencode`
2. **启动 Server**: `opencode serve --port 4096`
3. **配置 Provider**: 在 OpenCode 中添加 AI Provider API Key
4. **在 Code Sensei 中配置**:
   - Server 地址: `http://localhost:4096`
   - 用户名: `opencode`
   - 密码: (可选，如果在 Server 中设置了)

### 项目目录结构
```
code-sensei/
├── src/                    # Vue 前端源码
│   ├── views/             # 页面组件
│   ├── components/        # 可复用组件
│   ├── api/               # Tauri API 封装
│   └── router/            # 路由配置
├── src-tauri/             # Rust 后端源码
│   ├── src/
│   │   ├── main.rs        # Tauri 入口 + Commands
│   │   ├── opencode.rs    # OpenCode 客户端
│   │   └── config.rs      # 配置管理
│   └── Cargo.toml         # Rust 依赖配置
├── docs/                  # 项目文档
└── dist/                  # 前端构建产物（运行 npm run build 生成）
```

## 待办事项

### 短期目标
- [ ] 完善错误处理和用户提示
- [ ] 添加更多 AI 模型支持测试
- [ ] 优化文件树性能（大型项目）
- [ ] 添加深色模式支持

### 中期目标
- [ ] 实现代码解释功能
- [ ] 添加学习路径推荐
- [ ] 实现代码质量分析
- [ ] 支持多项目对比学习

### 长期目标
- [ ] 建立学习进度追踪系统
- [ ] 添加练习题库
- [ ] 社区功能（分享项目、协作学习）
- [ ] 移动端适配

## 开发指南

### 环境要求
- Node.js >= 18
- Rust >= 1.70
- npm 或 yarn

### 本地开发
```bash
# 安装依赖
npm install

# 启动开发服务器
npm run tauri dev
```

### 构建
```bash
# 构建前端 + Rust
npm run tauri build

# 仅构建前端
npm run build

# 仅构建 Rust
cd src-tauri && cargo build --release
```

## 贡献指南

欢迎通过 Issue 或 Pull Request 贡献代码！

## 许可证

MIT

---

**最后更新**: 2025-02-09
