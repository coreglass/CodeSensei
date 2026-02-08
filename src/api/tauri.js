// Tauri API 封装
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'

/**
 * 扫描项目列表
 */
export async function scanProjects() {
  return invoke('scan_projects')
}

/**
 * 创建项目
 */
export async function createProject(params) {
  return invoke('create_project', {
    name: params.name,
    description: params.description,
    rootPath: params.rootPath
  })
}

/**
 * 删除项目
 */
export async function deleteProject(projectId) {
  return invoke('delete_project', { projectId })
}

/**
 * 读取项目文件
 */
export async function readProjectFile(projectId, fileType) {
  return invoke('read_file', {
    projectId,
    fileType
  })
}

/**
 * 写入项目文件
 */
export async function writeProjectFile(projectId, fileType, content) {
  return invoke('write_file', {
    projectId,
    fileType,
    content
  })
}

/**
 * 获取项目文件列表
 */
export async function getProjectFiles(projectId) {
  return invoke('get_project_files', { projectId })
}

/**
 * 获取源文件内容
 */
export async function getSourceFile(projectId, relativePath) {
  return invoke('get_source_file', {
    projectId,
    relativePath
  })
}

/**
 * 保存源文件
 */
export async function saveSourceFile(projectId, relativePath, content) {
  return invoke('save_source_file', {
    projectId,
    relativePath,
    content
  })
}

/**
 * 打开目录选择对话框
 */
export async function selectDirectory() {
  return open({
    directory: true,
    multiple: false,
    title: '选择项目根目录'
  })
}

/**
 * 创建新文件
 */
export async function createFile(projectId, relativePath, content = '') {
  return invoke('create_file', {
    projectId,
    relativePath,
    content
  })
}

/**
 * 创建新文件夹
 */
export async function createFolder(projectId, relativePath) {
  return invoke('create_folder', {
    projectId,
    relativePath
  })
}

/**
 * 重命名文件或文件夹
 */
export async function renameFile(projectId, oldPath, newPath) {
  return invoke('rename_file', {
    projectId,
    oldPath,
    newPath
  })
}

/**
 * 删除文件或文件夹
 */
export async function deleteFile(projectId, relativePath) {
  return invoke('delete_file', {
    projectId,
    relativePath
  })
}

/**
 * 移动文件或文件夹
 */
export async function moveFile(projectId, source, target) {
  return invoke('move_file', {
    projectId,
    source,
    target
  })
}

// ===== OpenCode API =====

/**
 * 使用 OpenCode 更新需求文档
 */
export async function updateRequirementWithAgent(projectId, userInput) {
  return invoke('update_requirement_with_agent', {
    projectId,
    userInput
  })
}

/**
 * 使用 OpenCode 创建文件
 */
export async function createFilesWithAgent(projectId, userInput) {
  return invoke('create_files_with_agent', {
    projectId,
    userInput
  })
}

/**
 * 获取 OpenCode 配置
 */
export async function getOpenCodeConfig() {
  return invoke('get_opencode_config')
}

/**
 * 保存 OpenCode 配置
 */
export async function saveOpenCodeConfig(config) {
  return invoke('save_opencode_config', { config })
}

/**
 * 测试 OpenCode Server 连接
 */
export async function testOpenCodeConnection(serverUrl, username, password) {
  return invoke('test_opencode_connection', {
    serverUrl,
    username,
    password
  })
}

/**
 * 更新 Server URL
 */
export async function updateServerUrl(serverUrl) {
  return invoke('update_server_url', { serverUrl })
}

/**
 * 更新认证信息
 */
export async function updateAuth(username, password) {
  return invoke('update_auth', { username, password })
}

/**
 * 更新 Provider 配置
 */
export async function updateProviderConfig(provider, model) {
  return invoke('update_provider_config', { provider, model })
}

/**
 * 获取可用的 AI Providers
 */
export async function getAvailableProviders(serverUrl, username, password) {
  return invoke('get_available_providers', {
    serverUrl,
    username,
    password
  })
}
