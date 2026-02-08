// Prevents additional console window on Windows in release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod opencode;

use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use serde_json;
use tauri::{Manager, Emitter};
use config::*;
use opencode::OpenCodeClient;

#[derive(Clone)]
struct AppState {
    projects_dir: PathBuf,
}

// ===== 数据模型 =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub description: String,
    pub language: String,
    pub created_at: i64,
    pub updated_at: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileNode {
    pub name: String,
    pub path: String,
    pub is_file: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<FileNode>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub description: String,
    pub status: String,
    pub order: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

// ===== Tauri Commands =====

#[tauri::command]
fn scan_projects(state: tauri::State<'_, AppState>) -> Result<Vec<Project>, String> {
    let projects_dir = &state.projects_dir;

    if !projects_dir.exists() {
        fs::create_dir_all(projects_dir)
            .map_err(|e| format!("Failed to create projects directory: {}", e))?;
        return Ok(vec![]);
    }

    let mut projects = Vec::new();

    let entries = fs::read_dir(projects_dir)
        .map_err(|e| format!("Failed to read projects directory: {}", e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let path = entry.path();

        if path.is_dir() {
            let meta_file = path.join("project.json");
            if meta_file.exists() {
                let content = fs::read_to_string(&meta_file)
                    .map_err(|e| format!("Failed to read project.json: {}", e))?;

                let project: Project = serde_json::from_str(&content)
                    .map_err(|e| format!("Failed to parse project.json: {}", e))?;

                projects.push(project);
            }
        }
    }

    projects.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(projects)
}

#[tauri::command]
fn create_project(
    state: tauri::State<'_, AppState>,
    name: String,
    description: String,
    root_path: Option<String>,
) -> Result<Project, String> {
    let id = chrono::Utc::now().timestamp_millis().to_string();
    let project_dir = state.projects_dir.join(&id);

    // 创建项目目录结构
    fs::create_dir_all(&project_dir)
        .map_err(|e| format!("Failed to create project directory: {}", e))?;

    // 如果没有指定 root_path，创建默认目录结构
    if root_path.is_none() {
        fs::create_dir_all(project_dir.join("src"))
            .map_err(|e| format!("Failed to create src directory: {}", e))?;

        fs::create_dir_all(project_dir.join("docs"))
            .map_err(|e| format!("Failed to create docs directory: {}", e))?;
    }

    let now = chrono::Utc::now().timestamp();

    let project = Project {
        id: id.clone(),
        name: name.clone(),
        description: description.clone(),
        language: "Python".to_string(),
        created_at: now,
        updated_at: now,
        root_path: root_path.clone(),
    };

    // 保存项目元数据
    let meta_file = project_dir.join("project.json");
    let content = serde_json::to_string_pretty(&project)
        .map_err(|e| format!("Failed to serialize project: {}", e))?;

    fs::write(&meta_file, content)
        .map_err(|e| format!("Failed to write project.json: {}", e))?;

    // 如果没有 root_path，创建初始需求文档
    if root_path.is_none() {
        let requirement_file = project_dir.join("requirement.md");
        let initial_requirement = format!("# {} 需求文档\n\n## 项目描述\n{}\n\n## 功能需求\n\n## 技术栈\n\n", name, description);
        fs::write(&requirement_file, initial_requirement)
            .map_err(|e| format!("Failed to create requirement.md: {}", e))?;
    }

    Ok(project)
}

#[tauri::command]
fn delete_project(state: tauri::State<'_, AppState>, project_id: String) -> Result<(), String> {
    let project_dir = state.projects_dir.join(&project_id);

    if project_dir.exists() {
        fs::remove_dir_all(&project_dir)
            .map_err(|e| format!("Failed to delete project directory: {}", e))?;
    }

    Ok(())
}

#[tauri::command]
fn read_file(state: tauri::State<'_, AppState>, project_id: String, file_type: String) -> Result<String, String> {
    let project_dir = state.projects_dir.join(&project_id);

    let file_path = match file_type.as_str() {
        "requirement" => project_dir.join("requirement.md"),
        "chat" => project_dir.join("chat.json"),
        "tasks" => project_dir.join("tasks.json"),
        _ => return Err(format!("Unknown file type: {}", file_type)),
    };

    if !file_path.exists() {
        return Ok(String::new());
    }

    fs::read_to_string(&file_path)
        .map_err(|e| format!("Failed to read file: {}", e))
}

#[tauri::command]
fn write_file(
    state: tauri::State<'_, AppState>,
    project_id: String,
    file_type: String,
    content: String,
) -> Result<(), String> {
    let project_dir = state.projects_dir.join(&project_id);

    let file_path = match file_type.as_str() {
        "requirement" => project_dir.join("requirement.md"),
        "chat" => project_dir.join("chat.json"),
        "tasks" => project_dir.join("tasks.json"),
        _ => return Err(format!("Unknown file type: {}", file_type)),
    };

    fs::write(&file_path, content)
        .map_err(|e| format!("Failed to write file: {}", e))
}

#[tauri::command]
fn get_project_files(state: tauri::State<'_, AppState>, project_id: String) -> Result<Vec<FileNode>, String> {
    let project_dir = state.projects_dir.join(&project_id);

    // 读取项目元数据
    let meta_file = project_dir.join("project.json");
    if !meta_file.exists() {
        return Ok(vec![]);
    }

    let content = fs::read_to_string(&meta_file)
        .map_err(|e| format!("Failed to read project.json: {}", e))?;
    let project: Project = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse project.json: {}", e))?;

    // 确定要扫描的根目录
    let scan_dir = if let Some(ref root_path) = project.root_path {
        PathBuf::from(root_path)
    } else {
        project_dir.join("src")
    };

    if !scan_dir.exists() {
        return Ok(vec![]);
    }

    // 构建文件树
    let file_tree = build_file_tree(&scan_dir, &scan_dir)
        .map_err(|e| format!("Failed to build file tree: {}", e))?;

    Ok(file_tree)
}

fn build_file_tree(dir: &PathBuf, base: &PathBuf) -> std::io::Result<Vec<FileNode>> {
    // 更保守的限制，防止卡顿
    build_file_tree_with_limit(dir, base, 0, 10, 1000)
}

// 带限制的文件树构建，避免扫描过深或过多文件
fn build_file_tree_with_limit(
    dir: &PathBuf,
    base: &PathBuf,
    current_depth: u32,
    max_depth: u32,
    max_files: usize,
) -> std::io::Result<Vec<FileNode>> {
    let mut nodes = Vec::new();
    let mut file_count = 0;

    // 应该跳过的常见大型目录
    let skip_dirs = [
        "node_modules",
        ".git",
        "target",
        "debug",
        "release",
        "build",
        "dist",
        ".vscode",
        ".idea",
        "vendor",
        "venv",
        ".venv",
        "__pycache__",
        ".next",
        ".nuxt",
        "coverage",
    ];

    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return Ok(Vec::new()), // 无权限的目录直接跳过
    };

    for entry in entries {
        // 检查文件数量限制
        if file_count >= max_files {
            break;
        }

        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue, // 跳过无法访问的条目
        };

        let path = entry.path();
        let name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        // 跳过隐藏文件和目录（以.开头）
        if name.starts_with('.') && name != ".gitignore" && name != ".env" {
            continue;
        }

        // 跳过常见的大型目录
        if skip_dirs.contains(&name.as_str()) {
            continue;
        }

        let relative_path = path.strip_prefix(base)
            .map(|p| p.to_str().unwrap_or("").replace('\\', "/"))
            .unwrap_or(String::new());

        if path.is_dir() {
            // 检查深度限制
            if current_depth >= max_depth {
                // 深度超限，添加一个占位节点
                nodes.push(FileNode {
                    name: format!("{} (深度限制)", name),
                    path: relative_path,
                    is_file: false,
                    children: Some(Vec::new()),
                });
                continue;
            }

            // 递归扫描子目录
            match build_file_tree_with_limit(&path, base, current_depth + 1, max_depth, max_files) {
                Ok(children) => {
                    nodes.push(FileNode {
                        name,
                        path: relative_path,
                        is_file: false,
                        children: Some(children),
                    });
                }
                Err(_) => {
                    // 无法访问的子目录，跳过
                    continue;
                }
            }
        } else {
            // 跳过大型二进制文件
            if let Some(ext) = path.extension() {
                let ext_str = ext.to_str().unwrap_or("");
                let skip_extensions = ["dll", "exe", "so", "dylib", "bin", "pdb", "o", "a", "lib"];
                if skip_extensions.contains(&ext_str) {
                    continue;
                }
            }

            nodes.push(FileNode {
                name,
                path: relative_path,
                is_file: true,
                children: None,
            });
            file_count += 1;
        }
    }

    nodes.sort_by(|a, b| {
        // 文件夹排在前面
        if !a.is_file && b.is_file {
            return std::cmp::Ordering::Less;
        }
        if a.is_file && !b.is_file {
            return std::cmp::Ordering::Greater;
        }
        a.name.cmp(&b.name)
    });

    Ok(nodes)
}

#[tauri::command]
fn get_source_file(
    state: tauri::State<'_, AppState>,
    project_id: String,
    relative_path: String,
) -> Result<String, String> {
    let project_dir = state.projects_dir.join(&project_id);

    // 读取项目元数据以确定根目录
    let meta_file = project_dir.join("project.json");
    let content_dir = if meta_file.exists() {
        let meta_content = fs::read_to_string(&meta_file)
            .map_err(|e| format!("Failed to read project.json: {}", e))?;
        let project: Project = serde_json::from_str(&meta_content)
            .map_err(|e| format!("Failed to parse project.json: {}", e))?;

        if let Some(ref root_path) = project.root_path {
            PathBuf::from(root_path)
        } else {
            project_dir.join("src")
        }
    } else {
        project_dir.join("src")
    };

    let file_path = content_dir.join(&relative_path);

    fs::read_to_string(&file_path)
        .map_err(|e| format!("Failed to read source file: {}", e))
}

#[tauri::command]
fn save_source_file(
    state: tauri::State<'_, AppState>,
    project_id: String,
    relative_path: String,
    content: String,
) -> Result<(), String> {
    let project_dir = state.projects_dir.join(&project_id);

    // 读取项目元数据以确定根目录
    let meta_file = project_dir.join("project.json");
    let content_dir = if meta_file.exists() {
        let meta_content = fs::read_to_string(&meta_file)
            .map_err(|e| format!("Failed to read project.json: {}", e))?;
        let project: Project = serde_json::from_str(&meta_content)
            .map_err(|e| format!("Failed to parse project.json: {}", e))?;

        if let Some(ref root_path) = project.root_path {
            PathBuf::from(root_path)
        } else {
            project_dir.join("src")
        }
    } else {
        project_dir.join("src")
    };

    let file_path = content_dir.join(&relative_path);

    // 确保父目录存在
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory: {}", e))?;
    }

    fs::write(&file_path, content)
        .map_err(|e| format!("Failed to write source file: {}", e))
}

#[tauri::command]
fn create_file(state: tauri::State<'_, AppState>, project_id: String, relative_path: String, content: String) -> Result<(), String> {
    let project_dir = state.projects_dir.join(&project_id);

    // 读取项目元数据
    let meta_file = project_dir.join("project.json");
    let content_dir = if meta_file.exists() {
        let meta_content = fs::read_to_string(&meta_file)
            .map_err(|e| format!("Failed to read project.json: {}", e))?;
        let project: Project = serde_json::from_str(&meta_content)
            .map_err(|e| format!("Failed to parse project.json: {}", e))?;

        if let Some(ref root_path) = project.root_path {
            PathBuf::from(root_path)
        } else {
            project_dir.join("src")
        }
    } else {
        project_dir.join("src")
    };

    let file_path = content_dir.join(&relative_path);

    // 确保父目录存在
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory: {}", e))?;
    }

    fs::write(&file_path, content)
        .map_err(|e| format!("Failed to create file: {}", e))
}

#[tauri::command]
fn create_folder(state: tauri::State<'_, AppState>, project_id: String, relative_path: String) -> Result<(), String> {
    let project_dir = state.projects_dir.join(&project_id);

    // 读取项目元数据
    let meta_file = project_dir.join("project.json");
    let content_dir = if meta_file.exists() {
        let meta_content = fs::read_to_string(&meta_file)
            .map_err(|e| format!("Failed to read project.json: {}", e))?;
        let project: Project = serde_json::from_str(&meta_content)
            .map_err(|e| format!("Failed to parse project.json: {}", e))?;

        if let Some(ref root_path) = project.root_path {
            PathBuf::from(root_path)
        } else {
            project_dir.join("src")
        }
    } else {
        project_dir.join("src")
    };

    let folder_path = content_dir.join(&relative_path);

    // 如果文件夹已存在，直接返回成功
    if folder_path.exists() {
        return Ok(());
    }

    fs::create_dir_all(&folder_path)
        .map_err(|e| format!("Failed to create folder: {}", e))
}

#[tauri::command]
fn rename_file(state: tauri::State<'_, AppState>, project_id: String, old_path: String, new_path: String) -> Result<(), String> {
    let project_dir = state.projects_dir.join(&project_id);

    // 读取项目元数据
    let meta_file = project_dir.join("project.json");
    let content_dir = if meta_file.exists() {
        let meta_content = fs::read_to_string(&meta_file)
            .map_err(|e| format!("Failed to read project.json: {}", e))?;
        let project: Project = serde_json::from_str(&meta_content)
            .map_err(|e| format!("Failed to parse project.json: {}", e))?;

        if let Some(ref root_path) = project.root_path {
            PathBuf::from(root_path)
        } else {
            project_dir.join("src")
        }
    } else {
        project_dir.join("src")
    };

    let old_file_path = content_dir.join(&old_path);
    let new_file_path = content_dir.join(&new_path);

    // 确保新路径的父目录存在
    if let Some(parent) = new_file_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory: {}", e))?;
    }

    fs::rename(&old_file_path, &new_file_path)
        .map_err(|e| format!("Failed to rename: {}", e))
}

#[tauri::command]
fn delete_file(state: tauri::State<'_, AppState>, project_id: String, relative_path: String) -> Result<(), String> {
    let project_dir = state.projects_dir.join(&project_id);

    // 读取项目元数据
    let meta_file = project_dir.join("project.json");
    let content_dir = if meta_file.exists() {
        let meta_content = fs::read_to_string(&meta_file)
            .map_err(|e| format!("Failed to read project.json: {}", e))?;
        let project: Project = serde_json::from_str(&meta_content)
            .map_err(|e| format!("Failed to parse project.json: {}", e))?;

        if let Some(ref root_path) = project.root_path {
            PathBuf::from(root_path)
        } else {
            project_dir.join("src")
        }
    } else {
        project_dir.join("src")
    };

    let file_path = content_dir.join(&relative_path);

    if file_path.is_dir() {
        fs::remove_dir_all(&file_path)
            .map_err(|e| format!("Failed to delete directory: {}", e))?;
    } else {
        fs::remove_file(&file_path)
            .map_err(|e| format!("Failed to delete file: {}", e))?;
    }

    Ok(())
}

#[tauri::command]
fn move_file(state: tauri::State<'_, AppState>, project_id: String, source: String, target: String) -> Result<(), String> {
    let project_dir = state.projects_dir.join(&project_id);

    // 读取项目元数据
    let meta_file = project_dir.join("project.json");
    let content_dir = if meta_file.exists() {
        let meta_content = fs::read_to_string(&meta_file)
            .map_err(|e| format!("Failed to read project.json: {}", e))?;
        let project: Project = serde_json::from_str(&meta_content)
            .map_err(|e| format!("Failed to parse project.json: {}", e))?;

        if let Some(ref root_path) = project.root_path {
            PathBuf::from(root_path)
        } else {
            project_dir.join("src")
        }
    } else {
        project_dir.join("src")
    };

    let source_path = content_dir.join(&source);
    let target_path = content_dir.join(&target);

    // 确保目标路径的父目录存在
    if let Some(parent) = target_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory: {}", e))?;
    }

    // 尝试直接重命名（在同一文件系统内）
    if let Err(_) = fs::rename(&source_path, &target_path) {
        // 如果重命名失败（可能跨设备），则复制后删除
        if source_path.is_dir() {
            copy_dir_recursive(&source_path, &target_path)
                .map_err(|e| format!("Failed to copy directory: {}", e))?;
        } else {
            fs::copy(&source_path, &target_path)
                .map_err(|e| format!("Failed to copy file: {}", e))?;
        }

        // 删除源文件
        if source_path.is_dir() {
            fs::remove_dir_all(&source_path)
                .map_err(|e| format!("Failed to remove source directory: {}", e))?;
        } else {
            fs::remove_file(&source_path)
                .map_err(|e| format!("Failed to remove source file: {}", e))?;
        }
    }

    Ok(())
}

fn copy_dir_recursive(source: &PathBuf, target: &PathBuf) -> std::io::Result<()> {
    fs::create_dir_all(target)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let target_path = target.join(entry.file_name());

        if source_path.is_dir() {
            copy_dir_recursive(&source_path, &target_path)?;
        } else {
            fs::copy(&source_path, &target_path)?;
        }
    }
    Ok(())
}

// ===== Claude Agent Commands =====

/// 使用 OpenCode 更新需求文档
#[tauri::command]
async fn update_requirement_with_agent(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    project_id: String,
    user_input: String,
) -> Result<AgentResponse, String> {
    println!("========== 使用 OpenCode 更新需求文档 ==========");
    println!("项目 ID: {}", project_id);
    println!("用户输入: {}", user_input);

    // 1. 获取项目元数据
    let app_project_dir = state.projects_dir.join(&project_id);
    let meta_file = app_project_dir.join("project.json");

    let project: Project = if meta_file.exists() {
        let meta_content = fs::read_to_string(&meta_file)
            .map_err(|e| format!("无法读取 project.json: {}", e))?;
        serde_json::from_str(&meta_content)
            .map_err(|e| format!("无法解析 project.json: {}", e))?
    } else {
        return Err("项目不存在".to_string());
    };

    // 2. 确定需求文档的保存位置
    let requirement_path = if let Some(ref root_path) = project.root_path {
        PathBuf::from(root_path).join("requirement.md")
    } else {
        app_project_dir.join("requirement.md")
    };

    let requirement_path_display = requirement_path.display().to_string();

    // 3. 读取现有的需求文档内容
    let current_requirement = if requirement_path.exists() {
        fs::read_to_string(&requirement_path).unwrap_or_default()
    } else {
        String::new()
    };

    // 4. 获取 OpenCode 配置
    let config = get_config();
    println!("OpenCode Server: {}", config.server_url);

    // 5. 创建 OpenCode 客户端
    let client = OpenCodeClient::new(
        config.server_url.clone(),
        config.username.clone(),
        config.password.clone(),
    );

    // 6. 检查服务器连接
    println!("检查 OpenCode Server 连接...");
    let health = client.health_check().await
        .map_err(|e| format!("无法连接到 OpenCode Server: {}\n请检查 Server 是否运行，地址是否正确", e))?;
    println!("Server 版本: {}", health.version);

    // 7. 构建提示词
    let prompt = if current_requirement.is_empty() {
        format!(
            "你是 Code Sensei 的需求文档编辑助手。

## 用户需求
{}

## 任务
请根据用户需求创建需求文档。

## 输出格式
严格按照 Markdown 格式输出完整的需求文档，包含：
- 项目描述
- 功能需求
- 技术栈
- 其他必要章节

请直接输出需求文档内容，不要有其他说明。",
            user_input
        )
    } else {
        format!(
            "你是 Code Sensei 的需求文档编辑助手。

## 用户需求
{}

## 当前需求文档内容
```markdown{}
```

## 任务
请根据用户需求更新需求文档。保持文档结构清晰，使用 Markdown 格式。

请直接输出更新后的完整需求文档内容，不要有其他说明。",
            user_input, current_requirement
        )
    };

    // 8. 创建会话
    println!("创建 OpenCode 会话...");
    let session = client.create_session(
        "需求文档更新",
        config.default_provider.clone(),
        config.default_model.clone(),
    ).await
        .map_err(|e| format!("创建会话失败: {}", e))?;

    println!("会话 ID: {}", session.id);

    // 发送进度事件
    let _ = app.emit("agent-progress", serde_json::json!({
        "stage": "processing",
        "message": "正在生成需求文档..."
    }));

    // 9. 发送消息并获取响应
    println!("发送消息到 OpenCode...");
    let response = client.send_message(&session.id, &prompt, None, None).await
        .map_err(|e| format!("发送消息失败: {}", e))?;

    // 10. 提取响应文本
    let response_text = response.parts
        .iter()
        .filter_map(|part| part.text.as_ref())
        .cloned()
        .collect::<Vec<_>>()
        .join("\n");

    if response_text.is_empty() {
        return Err("AI 返回了空响应".to_string());
    }

    println!("收到响应，长度: {} 字符", response_text.len());

    // 11. 保存到需求文档
    if let Some(parent) = requirement_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("无法创建目录: {}", e))?;
    }

    fs::write(&requirement_path, &response_text)
        .map_err(|e| format!("无法保存需求文档: {}", e))?;

    println!("需求文档已保存到: {}", requirement_path_display);

    // 12. 删除临时会话
    let _ = client.delete_session(&session.id);

    // 13. 发送事件通知前端
    let _ = app.emit("requirement-updated", serde_json::json!({
        "project_id": project_id,
        "file_path": requirement_path_display
    }));

    println!("============================================");

    Ok(AgentResponse {
        success: true,
        message: "需求文档已更新".to_string(),
        file_modified: Some(requirement_path_display),
        document_content: Some(response_text),
        error: None,
    })
}

/// 使用 OpenCode 创建/修改文件（异步版本）
#[tauri::command]
async fn create_files_with_agent_async(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    project_id: String,
    user_input: String,
) -> Result<String, String> {
    println!("========== 使用 OpenCode 创建/修改文件（异步）==========");
    println!("项目 ID: {}", project_id);
    println!("用户输入: {}", user_input);

    // 1. 获取项目元数据
    let app_project_dir = state.projects_dir.join(&project_id);
    let meta_file = app_project_dir.join("project.json");

    let project: Project = if meta_file.exists() {
        let meta_content = fs::read_to_string(&meta_file)
            .map_err(|e| format!("无法读取 project.json: {}", e))?;
        serde_json::from_str(&meta_content)
            .map_err(|e| format!("无法解析 project.json: {}", e))?
    } else {
        return Err("项目不存在".to_string());
    };

    // 2. 确定项目根目录
    let project_root = if let Some(ref root_path) = project.root_path {
        PathBuf::from(root_path)
    } else {
        app_project_dir.clone()
    };

    let project_root_str = project_root.display().to_string();

    // 3. 读取需求文档（如果存在）
    let requirement_path = project_root.join("requirement.md");
    let requirement_content = if requirement_path.exists() {
        fs::read_to_string(&requirement_path).unwrap_or_default()
    } else {
        String::new()
    };

    // 4. 获取 OpenCode 配置
    let config = get_config();
    println!("OpenCode Server: {}", config.server_url);

    // 5. 创建 OpenCode 客户端
    let client = OpenCodeClient::new(
        config.server_url.clone(),
        config.username.clone(),
        config.password.clone(),
    );

    // 6. 检查服务器连接
    println!("检查 OpenCode Server 连接...");
    let health = client.health_check().await
        .map_err(|e| format!("无法连接到 OpenCode Server: {}\n请检查 Server 是否运行，地址是否正确", e))?;
    println!("Server 版本: {}", health.version);

    // 发送进度事件
    let _ = app.emit("agent-progress", serde_json::json!({
        "stage": "analyzing",
        "message": "正在分析项目结构和需求..."
    }));

    // 7. 构建提示词
    let prompt = if requirement_content.is_empty() {
        format!(
            "你是 Code Sensei 的代码生成助手。

## 项目路径
{}

## 用户需求
{}

## 任务
根据用户需求在项目中创建或修改文件。

## 工作原则
- 先用 Read 工具读取现有文件，了解项目结构
- 优先修改现有文件，避免创建不必要的文件
- 保持代码风格一致
- 确保代码可以运行

请简要说明你修改了哪些文件。",
            project_root_str, user_input
        )
    } else {
        format!(
            "你是 Code Sensei 的代码生成助手。

## 项目路径
{}

## 需求文档内容
```markdown{}
```

## 用户需求
{}

## 任务
根据需求文档和用户需求，在项目中创建或修改文件。

## 工作原则
- 先用 Read 工具读取现有文件，了解项目结构
- 优先修改现有文件，避免创建不必要的文件
- 保持代码风格一致
- 确保代码可以运行

请简要说明你修改了哪些文件。",
            project_root_str, requirement_content, user_input
        )
    };

    // 8. 创建会话
    println!("创建 OpenCode 会话...");
    let session = client.create_session(
        "代码生成",
        config.default_provider.clone(),
        config.default_model.clone(),
    ).await
        .map_err(|e| format!("创建会话失败: {}", e))?;

    let session_id = session.id.clone();
    println!("会话 ID: {}", session_id);

    // 9. 异步发送消息（立即返回）
    println!("异步发送消息到 OpenCode...");
    client.send_message_async(&session_id, &prompt, None, None).await
        .map_err(|e| format!("发送消息失败: {}", e))?;

    println!("消息已异步发送，会话 ID: {}", session_id);

    // 发送事件通知前端开始轮询
    let _ = app.emit("agent-task-started", serde_json::json!({
        "project_id": project_id,
        "session_id": session_id
    }));

    println!("============================================");

    // 返回会话 ID，前端可以用它来轮询结果
    Ok(session_id)
}

/// 获取会话中的消息列表（用于轮询）
#[tauri::command]
async fn get_session_messages(session_id: String, limit: Option<u32>) -> Result<Vec<opencode::Message>, String> {
    let config = get_config();
    let client = OpenCodeClient::new(
        config.server_url.clone(),
        config.username.clone(),
        config.password.clone(),
    );

    client.get_messages(&session_id, limit).await
}

/// 使用 OpenCode 创建/修改文件（同步版本，保留用于简单任务）
#[tauri::command]
async fn create_files_with_agent(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    project_id: String,
    user_input: String,
) -> Result<AgentResponse, String> {
    println!("========== 使用 OpenCode 创建/修改文件 ==========");
    println!("项目 ID: {}", project_id);
    println!("用户输入: {}", user_input);

    // 1. 获取项目元数据
    let app_project_dir = state.projects_dir.join(&project_id);
    let meta_file = app_project_dir.join("project.json");

    let project: Project = if meta_file.exists() {
        let meta_content = fs::read_to_string(&meta_file)
            .map_err(|e| format!("无法读取 project.json: {}", e))?;
        serde_json::from_str(&meta_content)
            .map_err(|e| format!("无法解析 project.json: {}", e))?
    } else {
        return Err("项目不存在".to_string());
    };

    // 2. 确定项目根目录
    let project_root = if let Some(ref root_path) = project.root_path {
        PathBuf::from(root_path)
    } else {
        app_project_dir.clone()
    };

    let project_root_str = project_root.display().to_string();

    // 3. 读取需求文档（如果存在）
    let requirement_path = project_root.join("requirement.md");
    let requirement_content = if requirement_path.exists() {
        fs::read_to_string(&requirement_path).unwrap_or_default()
    } else {
        String::new()
    };

    // 4. 获取 OpenCode 配置
    let config = get_config();
    println!("OpenCode Server: {}", config.server_url);

    // 5. 创建 OpenCode 客户端
    let client = OpenCodeClient::new(
        config.server_url.clone(),
        config.username.clone(),
        config.password.clone(),
    );

    // 6. 检查服务器连接
    println!("检查 OpenCode Server 连接...");
    let health = client.health_check().await
        .map_err(|e| format!("无法连接到 OpenCode Server: {}\n请检查 Server 是否运行，地址是否正确", e))?;
    println!("Server 版本: {}", health.version);

    // 发送进度事件
    let _ = app.emit("agent-progress", serde_json::json!({
        "stage": "analyzing",
        "message": "正在分析项目结构和需求..."
    }));

    // 7. 构建提示词
    let prompt = if requirement_content.is_empty() {
        format!(
            "你是 Code Sensei 的代码生成助手。

## 项目路径
{}

## 用户需求
{}

## 任务
根据用户需求在项目中创建或修改文件。

## 工作原则
- 先用 Read 工具读取现有文件，了解项目结构
- 优先修改现有文件，避免创建不必要的文件
- 保持代码风格一致
- 确保代码可以运行

请简要说明你修改了哪些文件。",
            project_root_str, user_input
        )
    } else {
        format!(
            "你是 Code Sensei 的代码生成助手。

## 项目路径
{}

## 需求文档内容
```markdown{}
```

## 用户需求
{}

## 任务
根据需求文档和用户需求，在项目中创建或修改文件。

## 工作原则
- 先用 Read 工具读取现有文件，了解项目结构
- 优先修改现有文件，避免创建不必要的文件
- 保持代码风格一致
- 确保代码可以运行

请简要说明你修改了哪些文件。",
            project_root_str, requirement_content, user_input
        )
    };

    // 8. 创建会话
    println!("创建 OpenCode 会话...");
    let session = client.create_session(
        "代码生成",
        config.default_provider.clone(),
        config.default_model.clone(),
    ).await
        .map_err(|e| format!("创建会话失败: {}", e))?;

    println!("会话 ID: {}", session.id);

    // 发送进度事件
    let _ = app.emit("agent-progress", serde_json::json!({
        "stage": "working",
        "message": "正在创建/修改文件..."
    }));

    // 9. 发送消息并获取响应
    println!("发送消息到 OpenCode...");
    let response = client.send_message(&session.id, &prompt, None, None).await
        .map_err(|e| format!("发送消息失败: {}", e))?;

    // 10. 提取响应文本
    let response_text = response.parts
        .iter()
        .filter_map(|part| part.text.as_ref())
        .cloned()
        .collect::<Vec<_>>()
        .join("\n");

    println!("收到响应");
    println!("============================================");

    // 11. 删除临时会话
    let _ = client.delete_session(&session.id);

    // 12. 发送完成事件通知前端刷新文件树
    let _ = app.emit("files-operation-completed", serde_json::json!({
        "project_id": project_id,
        "message": response_text
    }));

    println!("📢 files-operation-completed 事件已发送");

    Ok(AgentResponse {
        success: true,
        message: response_text,
        file_modified: None,
        document_content: None,
        error: None,
    })
}

// ===== OpenCode 配置命令 =====

/// 获取 OpenCode 配置
#[tauri::command]
fn get_opencode_config() -> OpenCodeConfig {
    config::get_config()
}

/// 保存 OpenCode 配置
#[tauri::command]
fn save_opencode_config(config: OpenCodeConfig) -> Result<(), String> {
    config::save_config(&config)
}

/// 测试 OpenCode Server 连接
#[tauri::command]
async fn test_opencode_connection(
    app: tauri::AppHandle,
    server_url: String,
    username: String,
    password: Option<String>,
) -> Result<String, String> {
    use opencode::OpenCodeClient;

    // 发送测试事件
    let _ = app.emit(
        "opencode-test-start",
        serde_json::json!({"server_url": server_url}),
    );

    let client = OpenCodeClient::new(server_url, username, password);

    match client.health_check().await {
        Ok(health) => {
            let message = format!("✅ 连接成功！OpenCode Server 版本: {}", health.version);
            let _ = app.emit(
                "opencode-test-success",
                serde_json::json!({"version": health.version}),
            );
            Ok(message)
        }
        Err(e) => {
            let _ = app.emit(
                "opencode-test-error",
                serde_json::json!({"error": e}),
            );
            Err(format!("❌ 连接失败: {}", e))
        }
    }
}

/// 更新 Server URL
#[tauri::command]
fn update_server_url(server_url: String) -> Result<(), String> {
    let manager = config::CONFIG_MANAGER
        .lock()
        .unwrap();
    if let Some(mgr) = manager.as_ref() {
        mgr.update_server_url(server_url)
    } else {
        Err("配置管理器未初始化".to_string())
    }
}

/// 更新认证信息
#[tauri::command]
fn update_auth(username: String, password: Option<String>) -> Result<(), String> {
    let manager = config::CONFIG_MANAGER
        .lock()
        .unwrap();
    if let Some(mgr) = manager.as_ref() {
        mgr.update_auth(username, password)
    } else {
        Err("配置管理器未初始化".to_string())
    }
}

/// 更新 Provider 配置
#[tauri::command]
fn update_provider_config(
    provider: Option<String>,
    model: Option<String>,
) -> Result<(), String> {
    let manager = config::CONFIG_MANAGER
        .lock()
        .unwrap();
    if let Some(mgr) = manager.as_ref() {
        mgr.update_provider(provider, model)
    } else {
        Err("配置管理器未初始化".to_string())
    }
}

/// 获取可用的 AI Providers
#[tauri::command]
async fn get_available_providers(server_url: String, username: String, password: Option<String>) -> Result<Vec<opencode::Provider>, String> {
    use opencode::OpenCodeClient;

    let client = OpenCodeClient::new(server_url, username, password);

    // 尝试获取配置文件中的 providers（包含模型列表）
    match client.get_config_providers().await {
        Ok(providers) => Ok(providers),
        Err(_) => {
            // 如果失败，尝试获取基本的 providers 列表
            client.get_providers().await
        }
    }
}

fn main() {
    // 初始化配置管理器
    if let Err(e) = init_config_manager() {
        eprintln!("⚠️  初始化配置管理器失败: {}", e);
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            // 获取应用数据目录
            let app_data_dir = app.path().app_data_dir()
                .expect("Failed to get app data dir");

            // 创建项目目录
            let projects_dir = app_data_dir.join("projects");
            fs::create_dir_all(&projects_dir)
                .expect("Failed to create projects directory");

            let projects_dir_display = projects_dir.display().to_string();

            app.manage(AppState {
                projects_dir,
            });

            println!("🚀 Code Sensei 已启动");
            println!("📁 项目目录: {}", projects_dir_display);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            scan_projects,
            create_project,
            delete_project,
            read_file,
            write_file,
            get_project_files,
            get_source_file,
            save_source_file,
            create_file,
            create_folder,
            rename_file,
            delete_file,
            move_file,
            update_requirement_with_agent,
            create_files_with_agent,
            create_files_with_agent_async,
            get_session_messages,
            // OpenCode 配置命令
            get_opencode_config,
            save_opencode_config,
            test_opencode_connection,
            update_server_url,
            update_auth,
            update_provider_config,
            get_available_providers,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

