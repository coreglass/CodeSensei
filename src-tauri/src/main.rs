// Prevents additional console window on Windows in release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod claude;
mod claude_node;

use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use serde_json;
use tauri::{Manager, Emitter};
use claude::{save_api_key, get_api_key};
use claude_node::{ClaudeAgentNode, RequirementUpdateRequest, CreateFilesRequest, AgentResponse};

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

/// 使用 Claude Code (Node.js SDK) 更新需求文档
#[tauri::command]
async fn update_requirement_with_agent(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    project_id: String,
    user_input: String,
) -> Result<AgentResponse, String> {
    // 1. 获取项目元数据目录
    let app_project_dir = state.projects_dir.join(&project_id);
    let meta_file = app_project_dir.join("project.json");

    // 2. 读取项目元数据
    let project: Project = if meta_file.exists() {
        let meta_content = fs::read_to_string(&meta_file)
            .map_err(|e| format!("Failed to read project.json: {}", e))?;
        serde_json::from_str(&meta_content)
            .map_err(|e| format!("Failed to parse project.json: {}", e))?
    } else {
        return Err("Project not found".to_string());
    };

    // 3. 确定需求文档的保存位置
    let requirement_path = if let Some(ref root_path) = project.root_path {
        PathBuf::from(root_path).join("requirement.md")
    } else {
        app_project_dir.join("requirement.md")
    };

    let requirement_path_display = requirement_path.display().to_string();

    // 4. 确定项目根目录（用于 Claude Code 工作目录）
    let project_root = if let Some(ref root_path) = project.root_path {
        PathBuf::from(root_path)
    } else {
        app_project_dir.clone()
    };

    // 5. 创建 Claude Agent Node 并调用
    let agent = ClaudeAgentNode::new();

    // 读取现有的需求文档内容（如果存在）
    let current_requirement = if requirement_path.exists() {
        fs::read_to_string(&requirement_path).unwrap_or_default()
    } else {
        String::new()
    };

    let req = RequirementUpdateRequest {
        user_input,
        project_path: project_root.display().to_string(),
        current_content: current_requirement,
    };

    let response = agent.update_requirement(&app, req).await?;

    // 6. 保存返回的内容到 requirement.md
    if let Some(doc_content) = &response.document_content {
        // 确保父目录存在
        if let Some(parent) = requirement_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directory: {}", e))?;
        }

        // 写入文件
        fs::write(&requirement_path, doc_content)
            .map_err(|e| format!("Failed to save requirement.md at {}: {}", requirement_path_display, e))?;

        // 发送事件通知前端刷新需求文档
        app.emit("requirement-updated", serde_json::json!({
            "project_id": project_id,
            "file_path": requirement_path_display
        })).map_err(|e| format!("Failed to emit event: {}", e))?;
    }

    Ok(response)
}

/// 使用 Claude Code (Node.js SDK) 创建/修改文件
#[tauri::command]
async fn create_files_with_agent(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    project_id: String,
    user_input: String,
) -> Result<AgentResponse, String> {
    // 1. 获取项目元数据
    let app_project_dir = state.projects_dir.join(&project_id);
    let meta_file = app_project_dir.join("project.json");

    let project: Project = if meta_file.exists() {
        let meta_content = fs::read_to_string(&meta_file)
            .map_err(|e| format!("Failed to read project.json: {}", e))?;
        serde_json::from_str(&meta_content)
            .map_err(|e| format!("Failed to parse project.json: {}", e))?
    } else {
        return Err("Project not found".to_string());
    };

    // 2. 确定项目根目录
    let project_root = if let Some(ref root_path) = project.root_path {
        PathBuf::from(root_path)
    } else {
        app_project_dir.clone()
    };

    // 3. 读取需求文档路径
    let requirement_path = project_root.join("requirement.md");
    let requirement_path_display = requirement_path.display().to_string();

    // 4. 创建 Claude Agent Node 并调用
    let agent = ClaudeAgentNode::new();

    // 发送进度事件
    app.emit("agent-progress", serde_json::json!({
        "project_id": project_id,
        "stage": "analyzing",
        "message": "正在分析项目结构和需求..."
    })).map_err(|e| format!("Failed to emit event: {}", e))?;

    let req = CreateFilesRequest {
        user_input,
        project_path: project_root.display().to_string(),
        requirement_path: requirement_path_display,
    };

    let response = agent.create_files(&app, req).await?;

    // 5. 发送完成事件通知前端刷新文件树
    app.emit("files-operation-completed", serde_json::json!({
        "project_id": project_id,
        "message": response.message
    })).map_err(|e| format!("Failed to emit event: {}", e))?;

    println!("📢 files-operation-completed 事件已发送");

    Ok(response)
}

/// 保存 Claude API Key
#[tauri::command]
fn save_claude_api_key(api_key: String) -> Result<(), String> {
    save_api_key(api_key)
}

/// 检查是否已配置 API Key
#[tauri::command]
fn check_api_key() -> Result<bool, String> {
    match get_api_key() {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

fn main() {
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

            app.manage(AppState {
                projects_dir,
            });

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
            save_claude_api_key,
            check_api_key,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

