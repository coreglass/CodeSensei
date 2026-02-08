# Code Sensei - OpenCode Edition

> A teaching-oriented programming assistant powered by OpenCode

## What is Code Sensei?

Code Sensei is a **teaching-oriented programming assistant** that helps developers learn programming through AI-powered guidance. Unlike typical code generators, Code Sensei focuses on **education and best practices**.

## ✨ Key Features

- 🎓 **Teaching-First Approach**: Explains *why* code is written a certain way
- 📝 **Requirements Management**: AI-assisted requirement document generation
- 📁 **Project Management**: Create, organize, and manage coding projects
- 🔄 **Iterative Development**: AI helps modify existing code rather than rewriting
- 🎯 **Multiple Modes**:
  - **Chat Mode**: Q&A for programming concepts
  - **Requirements Mode**: Generate and update requirement docs
  - **File Creation Mode**: AI-assisted file creation/modification
- 🔧 **Built-in Code Editor**: Monaco Editor (VS Code's editor)
- 🌳 **File Tree**: Navigate project structure easily
- 💾 **Auto-save**: Never lose your work

## 🏗️ Architecture

```
Frontend (Vue 3)
    ↓ Tauri IPC
Backend (Rust)
    ↓ HTTP
OpenCode Server
    ↓ AI Provider
OpenAI / Anthropic / Local Models
```

**Why OpenCode?**
- ✅ **Open Source**: No vendor lock-in
- ✅ **Flexible AI Providers**: Support for OpenAI, Anthropic, local models, etc.
- ✅ **HTTP API**: Simple, reliable communication
- ✅ **Active Community**: Regular updates and improvements

## 🛠️ Tech Stack

- **Frontend**: Vue 3 + Element Plus + Monaco Editor
- **Backend**: Tauri 2.x + Rust
- **AI**: OpenCode (HTTP API)
- **Storage**: JSON file-based

## 🚀 Quick Start

### Prerequisites

- Node.js >= 18
- Rust >= 1.70
- npm or yarn

### Installation

```bash
# Install dependencies
npm install

# Run in development mode
npm run tauri dev
```

### Setup OpenCode Server

**1. Install OpenCode:**
```bash
npm install -g @opencode/opencode
```

**2. Start the Server:**
```bash
opencode serve --port 4096
```

**3. Configure AI Provider:**

Open OpenCode TUI and configure your preferred AI provider:
```bash
opencode
# Press Ctrl+M to open model selector
# Choose provider (OpenAI, Anthropic, etc.)
# Enter API Key
```

**4. Configure Code Sensei:**

1. Open Code Sensei
2. Click **Settings** (top right)
3. Enter OpenCode Server URL: `http://localhost:4096`
4. Click **Test Connection**
5. Save configuration

## 📖 Usage Guide

### Creating a Project

1. Click **Create New Project**
2. Fill in project details:
   - Project name
   - Description
   - Root directory
3. Click **Create**

### Using AI Features

#### Requirements Mode
1. Open a project
2. Switch to **Requirements** tab
3. Describe what you want to build
4. AI generates a structured requirement document

#### File Creation Mode
1. Switch to **Create Files** tab
2. Describe what file you want to create/modify
3. AI analyzes the project and makes changes
4. File tree updates automatically

#### Chat Mode
1. Switch to **Chat** tab
2. Ask questions about code, architecture, best practices
3. AI provides detailed explanations

## 🏗️ Development

### Project Structure

```
Code Sensei/
├── src/                    # Vue frontend
│   ├── views/             # Page components
│   ├── components/        # Reusable components
│   └── api/               # Tauri API wrapper
├── src-tauri/             # Rust backend
│   ├── src/
│   │   ├── main.rs        # Tauri commands
│   │   ├── opencode.rs    # OpenCode client
│   │   └── config.rs      # Configuration management
│   └── Cargo.toml         # Rust dependencies
├── OPENCODE_TEST_GUIDE.md # Testing guide
└── OPENCODE_INTEGRATION.md # Integration documentation
```

### Building

```bash
# Development
npm run tauri dev

# Build for production
npm run tauri build

# Output
# Windows: src-tauri/target/release/bundle/nsis/Code Sensei_0.1.0_x64-setup.exe
```

## 🧪 Testing

See [OPENCODE_TEST_GUIDE.md](OPENCODE_TEST_GUIDE.md) for detailed testing instructions.

## 📚 Documentation

- [OpenCode Integration Guide](OPENCODE_INTEGRATION.md) - How OpenCode is integrated
- [Test Guide](OPENCODE_TEST_GUIDE.md) - Step-by-step testing instructions
- [OpenCode Official Docs](https://opencode.ai/docs/server/)

## 🗺️ Roadmap

### Phase 1: Core Features ✅
- ✅ Tauri + Vue 3 setup
- ✅ Project management
- ✅ File operations (create, delete, rename, move)
- ✅ AI-powered features (via OpenCode)
- ✅ Requirements document generation

### Phase 2: Enhanced Features (In Progress)
- [ ] Streaming output (SSE)
- [ ] Tool call visualization
- [ ] Session history
- [ ] Multiple AI provider support in UI

### Phase 3: Advanced Features
- [ ] Code analysis
- [ ] Refactoring suggestions
- [ ] Test generation
- [ ] Custom agents

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## 📄 License

MIT License - see LICENSE file for details

## 🙏 Acknowledgments

- [OpenCode](https://github.com/sst/opencode) - The amazing open-source AI coding agent
- [Tauri](https://tauri.app/) - Rust-based desktop app framework
- [Vue.js](https://vuejs.org/) - Progressive JavaScript framework
- [Element Plus](https://element-plus.org/) - Vue 3 UI library
- [Monaco Editor](https://microsoft.github.io/monaco-editor/) - VS Code's editor

## 📞 Support

If you encounter any issues:
1. Check [OPENCODE_TEST_GUIDE.md](OPENCODE_TEST_GUIDE.md)
2. Review OpenCode logs
3. Check Code Sensei backend logs
4. Open an issue on GitHub

---

**Made with ❤️ by the Code Sensei Team**
