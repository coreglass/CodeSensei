<template>
  <div class="project-container">
    <div class="toolbar">
      <el-button :icon="ArrowLeft" @click="goBack">返回</el-button>
      <el-divider direction="vertical" />
      <span class="project-name">{{ projectName }}</span>
      <el-divider direction="vertical" />
      <el-button :icon="ChatDotSquare" @click="toggleSidebar">
        {{ sidebarVisible ? '收起AI助手' : '展开AI助手' }}
      </el-button>
    </div>

    <div class="main-content">
      <!-- 最左侧：文件树 -->
      <div class="file-tree-panel" :style="{ width: fileTreeWidth + 'px' }">
        <div class="file-tree-header">
          <span>项目文件</span>
          <el-button size="small" :icon="Refresh" @click="loadProjectFiles" circle />
        </div>
        <div class="file-tree-content" @contextmenu.prevent="showTreeContextMenu($event, null)">
          <el-empty v-if="fileTree.length === 0" description="暂无文件" :image-size="80" />
          <el-tree
            v-else
            :data="fileTree"
            :props="treeProps"
            node-key="path"
            :default-expand-all="false"
            draggable
            @node-click="handleNodeClick"
            @node-contextmenu="showNodeContextMenu"
            @node-drag-end="handleDragEnd"
            :allow-drag="checkAllowDrag"
            :allow-drop="checkAllowDrop"
          >
            <template #default="{ node, data }">
              <div class="tree-node">
                <el-icon v-if="!data.is_file">
                  <Folder />
                </el-icon>
                <el-icon v-else>
                  <Document />
                </el-icon>
                <span>{{ node.label }}</span>
              </div>
            </template>
          </el-tree>
        </div>
      </div>

      <!-- 分隔条：文件树和编辑器之间 -->
      <div
        class="resizer resizer-left"
        @mousedown="startResize('left', $event)"
      ></div>

      <!-- 左侧：文件编辑区域 -->
      <div class="editor-area">
        <div class="file-selector">
          <div class="file-tabs">
            <div
              v-for="file in openFiles"
              :key="file.path"
              :class="['file-tab', { active: selectedFile === file.path }]"
              @click="selectFile(file.path)"
            >
              <el-icon><Document /></el-icon>
              <span>{{ file.name }}</span>
              <el-icon class="close-tab" @click.stop="closeFile(file.path)"><Close /></el-icon>
            </div>
          </div>
        </div>

        <div class="code-editor">
          <MonacoEditor
            v-if="selectedFile"
            :code="fileContent"
            :language="getLanguage(selectedFile)"
            @change="onCodeChange"
            @save="saveCurrentFile"
          />
          <el-empty v-else description="选择文件开始编辑，或在AI助手中创建文件" />
        </div>
      </div>

      <!-- 分隔条：编辑器和AI侧边栏之间 -->
      <div
        v-show="sidebarVisible"
        class="resizer resizer-right"
        @mousedown="startResize('right', $event)"
      ></div>

      <!-- 右侧：AI对话侧边栏 -->
      <div
        v-show="sidebarVisible"
        class="ai-sidebar"
        :style="{ width: aiSidebarWidth + 'px' }"
      >
        <!-- 侧边栏收起时的触发按钮 -->
        <div v-show="!sidebarVisible" class="sidebar-toggle-btn" @click="toggleSidebar">
          <el-icon :size="20"><ChatDotSquare /></el-icon>
        </div>

        <!-- 侧边栏内容 -->
        <div v-show="sidebarVisible" class="sidebar-content">
          <div class="sidebar-header">
            <span>AI 助手</span>
            <el-button size="small" :icon="Close" @click="toggleSidebar" circle />
          </div>

        <!-- 功能选择 -->
        <div class="mode-selector">
          <el-radio-group v-model="aiMode" size="small">
            <el-radio-button value="chat">对话</el-radio-button>
            <el-radio-button value="requirement">需求文档</el-radio-button>
            <el-radio-button value="create">创建文件</el-radio-button>
          </el-radio-group>
        </div>

        <!-- 聊天历史 -->
        <div class="chat-messages" ref="chatContainer">
          <div
            v-for="(msg, index) in chatHistory[aiMode]"
            :key="index"
            :class="['message', msg.role, { 'progress-message': msg.isProgress, 'loading-message': msg.isLoading }]"
          >
            <div class="message-content">
              <span v-if="msg.isLoading" class="loading-dots">正在处理中...</span>
              <span v-else>{{ msg.content }}</span>
            </div>
          </div>
          <div v-if="isLoading" class="message assistant">
            <div class="message-content">正在思考...</div>
          </div>
        </div>

        <!-- 输入区域 -->
        <div class="chat-input">
          <el-input
            v-model="userInput"
            type="textarea"
            :rows="3"
            :placeholder="getPlaceholder()"
            @keydown.ctrl.enter="sendMessage"
          />
          <div class="input-actions">
            <el-button
              type="primary"
              @click="sendMessage"
              :loading="isLoading"
              :disabled="!userInput.trim()"
            >
              发送 (Ctrl+Enter)
            </el-button>
          </div>
        </div>
        </div>
      </div>
    </div>

    <!-- 新建文件弹窗 -->
    <el-dialog v-model="showNewFileDialog" title="新建文件" width="400px">
      <el-form :model="newFileInfo" label-width="80px">
        <el-form-item label="文件名">
          <el-input v-model="newFileInfo.name" placeholder="例如：main.py" />
        </el-form-item>
        <el-form-item label="路径">
          <el-input v-model="newFileInfo.path" placeholder="例如：src/ (可选)" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="showNewFileDialog = false">取消</el-button>
        <el-button type="primary" @click="createNewFile">创建</el-button>
      </template>
    </el-dialog>

    <!-- 右键菜单 -->
    <el-dropdown
      ref="contextMenu"
      :virtual-ref="contextMenuTarget"
      virtual-triggering
      trigger="contextmenu"
      @visible-change="handleContextMenuVisible"
    >
      <template #dropdown>
        <el-dropdown-menu>
          <el-dropdown-item v-if="contextMenuData && contextMenuData.isDirectory" @click="createNewFileFromMenu">
            <el-icon><DocumentAdd /></el-icon>
            新建文件
          </el-dropdown-item>
          <el-dropdown-item v-if="contextMenuData && contextMenuData.isDirectory" @click="createNewFolderFromMenu">
            <el-icon><FolderAdd /></el-icon>
            新建文件夹
          </el-dropdown-item>
          <el-dropdown-item v-if="contextMenuData" @click="renameFromMenu" divided>
            <el-icon><Edit /></el-icon>
            重命名
          </el-dropdown-item>
          <el-dropdown-item v-if="contextMenuData" @click="deleteFromMenu">
            <el-icon><Delete /></el-icon>
            删除
          </el-dropdown-item>
        </el-dropdown-menu>
      </template>
    </el-dropdown>
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted, computed, nextTick } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import {
  ArrowLeft,
  ChatDotSquare,
  Close,
  Plus,
  Document,
  Refresh,
  Folder,
  DocumentAdd,
  FolderAdd,
  Edit,
  Delete,
} from '@element-plus/icons-vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import MonacoEditor from '../components/MonacoEditor.vue'
import * as tauriApi from '../api/tauri'
import { listen } from '@tauri-apps/api/event'

const route = useRoute()
const router = useRouter()

const projectId = computed(() => route.params.id)
const projectName = ref('项目')
const sidebarVisible = ref(true)
const showNewFileDialog = ref(false)
const aiMode = ref('chat')

// 文件管理
const openFiles = ref([])
const selectedFile = ref('')
const fileContent = ref('')
const originalContent = ref('')
const unsavedChanges = ref(false)
const fileTree = ref([])
const treeProps = {
  children: 'children',
  label: 'name',
}

// 右键菜单
const contextMenu = ref(null)
const contextMenuTarget = ref(null)
const contextMenuData = ref(null)

// 文件创建表单
const newFileInfo = ref({
  name: '',
  path: ''
})

// 面板宽度（可拖拽调整）
const fileTreeWidth = ref(250)
const editorWidth = ref('auto')  // 自动填充剩余空间
const aiSidebarWidth = ref(400)
const isResizing = ref(false)
const resizerType = ref('')  // 'left' 或 'right'

// AI对话
const userInput = ref('')
// 不同模式的独立会话历史
const chatHistory = ref({
  chat: [
    {
      role: 'assistant',
      content: '你好！我是你的AI助手。\n\n我可以帮你解答技术问题、分析代码、提供编程建议等。\n\n有什么问题随时问我！'
    }
  ],
  requirement: [
    {
      role: 'assistant',
      content: '你好！我是需求文档编辑助手。\n\n我可以帮你：\n• 📝 创建新的需求文档\n• ✏️ 更新现有需求\n• 📋 整理功能列表\n• 🔍 完善项目描述\n\n告诉我你的需求，我会帮你更新需求文档。'
    }
  ],
  create: [
    {
      role: 'assistant',
      content: '你好！我是文件创建助手。\n\n我可以帮你创建各种代码文件。\n\n💡 告诉我你想创建什么文件，例如：\n• "创建一个 main.py 入口文件"\n• "添加一个 utils.js 工具函数库"\n• "创建一个 User 用户类"'
    }
  ]
})
const isLoading = ref(false)
const chatContainer = ref(null)

// 需求文档
const requirementContent = ref('')

// 事件监听器存储
let unlistenRequirementUpdated = null

onMounted(async () => {
  await loadProjectInfo()
  await loadRequirement()
  await loadProjectFiles()

  // 监听需求文档更新事件
  unlistenRequirementUpdated = await listen('requirement-updated', async (event) => {
    console.log('=== 收到 requirement-updated 事件 ===', event.payload)
    const { project_id } = event.payload
    // 只刷新当前项目的需求文档
    if (project_id === projectId.value) {
      console.log('=== 项目ID匹配，开始刷新需求文档 ===')
      console.log('=== 当前选中的文件:', selectedFile.value, '===')

      // 直接重新加载文件内容（支持 'requirement' 和 'requirement.md' 两种情况）
      if (selectedFile.value === 'requirement' || selectedFile.value === 'requirement.md') {
        console.log('=== 正在重新加载 requirement 文件 ===')
        // 使用 selectedFile 的实际值来加载文件
        await loadFileContent(selectedFile.value)
        console.log('=== 文件内容已更新，长度:', fileContent.value.length, '===')
      } else {
        console.log('=== 当前未选中 requirement 文件，仅更新 requirementContent ===')
        await loadRequirement()
      }

      ElMessage.success('需求文档已更新')
    }
  })

  // 监听 Claude 消息事件（用于调试）
  const unlistenClaudeMessage = await listen('claude-message', async (event) => {
    console.log('========== 发送给 Claude Agent 的消息 ==========')
    console.log('模式:', event.payload.mode)
    console.log('')
    console.log('--- 系统提示词 ---')
    console.log(event.payload.system_prompt)
    console.log('')
    console.log('--- 用户消息 ---')
    console.log(event.payload.user_message)
    console.log('======================================================')
  })

  // 监听文件创建事件
  const unlistenFilesCreated = await listen('files-created', async (event) => {
    console.log('=== 收到 files-created 事件 ===', event.payload)
    const { project_id, count, first_file } = event.payload
    // 只刷新当前项目的文件树
    if (project_id === projectId.value) {
      console.log('=== 项目ID匹配，开始刷新文件树，创建了', count, '个文件 ===')

      // 刷新文件树
      await loadProjectFiles()

      // 自动打开第一个创建的文件
      if (first_file) {
        console.log('=== 自动打开文件:', first_file, '===')
        // 添加到打开的文件列表（如果尚未打开）
        if (!openFiles.value.some(f => f.path === first_file)) {
          const name = first_file.split('/').pop()
          openFiles.value.push({ name, path: first_file })
        }
        // 选中和加载文件
        selectFile(first_file)
      }

      ElMessage.success(`已创建 ${count} 个文件`)
    }
  })

  // 监听Agent进度事件
  const unlistenAgentProgress = await listen('agent-progress', async (event) => {
    console.log('=== 收到 agent-progress 事件 ===', event.payload)
    const { project_id, stage, message } = event.payload

    // 只处理当前项目的进度
    if (project_id === projectId.value) {
      // 在创建文件模式下显示进度消息
      if (aiMode.value === 'create') {
        // 检查是否已经有加载消息，如果有则先移除
        const loadingIndex = chatHistory.value.create.findIndex(msg => msg.isLoading)
        if (loadingIndex !== -1) {
          // 移除加载消息
          chatHistory.value.create.splice(loadingIndex, 1)
        }

        // 添加进度消息到聊天历史
        const progressMessage = {
          role: 'assistant',
          content: `⏳ ${message}`,
          isProgress: true  // 标记为进度消息
        }
        chatHistory.value.create.push(progressMessage)
        scrollToBottom()
      }
    }
  })
})

onUnmounted(() => {
  // 取消事件监听
  if (unlistenRequirementUpdated) {
    unlistenRequirementUpdated()
  }
  if (unlistenClaudeMessage) {
    unlistenClaudeMessage()
  }
  if (unlistenFilesCreated) {
    unlistenFilesCreated()
  }
  if (unlistenAgentProgress) {
    unlistenAgentProgress()
  }
})

async function loadProjectInfo() {
  try {
    const projects = await tauriApi.scanProjects()
    const project = projects.find(p => p.id === projectId.value)
    if (project) {
      projectName.value = project.name
    }
  } catch (error) {
    console.error('加载项目信息失败:', error)
  }
}

async function loadRequirement() {
  try {
    // 使用 getSourceFile API 获取最新内容
    const content = await tauriApi.getSourceFile(projectId.value, 'requirement')
    requirementContent.value = content

    // 如果用户当前正在查看需求文档，更新编辑器内容
    if (selectedFile.value === 'requirement' || selectedFile.value === 'requirement.md') {
      fileContent.value = content
      originalContent.value = content
      unsavedChanges.value = false
    }
  } catch (error) {
    const defaultContent = '# 需求文档\n\n暂无需求文档，请在右侧AI助手中创建。'
    requirementContent.value = defaultContent

    // 如果用户当前正在查看需求文档，显示默认内容
    if (selectedFile.value === 'requirement' || selectedFile.value === 'requirement.md') {
      fileContent.value = defaultContent
      originalContent.value = defaultContent
    }
  }
}

async function loadProjectFiles() {
  try {
    const files = await tauriApi.getProjectFiles(projectId.value)
    // 后端现在直接返回树形结构，不需要构建
    fileTree.value = files
  } catch (error) {
    console.error('加载项目文件失败:', error)
    fileTree.value = []
  }
}

function handleNodeClick(data) {
  if (data.is_file) {
    // 如果是文件，打开它
    if (!openFiles.value.some(f => f.path === data.path)) {
      const name = data.path.split('/').pop()
      openFiles.value.push({ name, path: data.path })
    }
    selectFile(data.path)
  }
}

// 右键菜单处理
function showTreeContextMenu(event, data) {
  event.preventDefault()
  contextMenuTarget.value = {
    getBoundingClientRect: () => ({
      left: event.clientX,
      top: event.clientY,
      width: 0,
      height: 0,
    }),
  }
  contextMenuData.value = {
    path: '',
    isDirectory: true,
  }
  nextTick(() => {
    if (contextMenu.value) {
      contextMenu.value.handleOpen()
    }
  })
}

function showNodeContextMenu(event, data) {
  event.preventDefault()
  event.stopPropagation()
  contextMenuTarget.value = {
    getBoundingClientRect: () => ({
      left: event.clientX,
      top: event.clientY,
      width: 0,
      height: 0,
    }),
  }
  contextMenuData.value = {
    path: data.path,
    isDirectory: !data.is_file,
  }
  nextTick(() => {
    if (contextMenu.value) {
      contextMenu.value.handleOpen()
    }
  })
}

function handleContextMenuVisible(visible) {
  if (!visible) {
    contextMenuTarget.value = null
  }
}

async function createNewFileFromMenu() {
  const parentPath = contextMenuData.value?.path || ''
  ElMessageBox.prompt('请输入文件名', '新建文件', {
    confirmButtonText: '创建',
    cancelButtonText: '取消',
    inputPattern: /\S+/,
    inputErrorMessage: '文件名不能为空'
  }).then(async ({ value }) => {
    try {
      const filePath = parentPath ? `${parentPath}/${value}` : value
      await tauriApi.createFile(projectId.value, filePath, '')
      await loadProjectFiles()
      ElMessage.success('文件已创建')
    } catch (error) {
      ElMessage.error('创建文件失败: ' + error)
    }
  }).catch(() => {})
}

async function createNewFolderFromMenu() {
  const parentPath = contextMenuData.value?.path || ''
  ElMessageBox.prompt('请输入文件夹名称', '新建文件夹', {
    confirmButtonText: '创建',
    cancelButtonText: '取消',
    inputPattern: /\S+/,
    inputErrorMessage: '文件夹名称不能为空'
  }).then(async ({ value }) => {
    try {
      const folderPath = parentPath ? `${parentPath}/${value}` : value
      await tauriApi.createFolder(projectId.value, folderPath)
      await loadProjectFiles()
      ElMessage.success('文件夹已创建')
    } catch (error) {
      ElMessage.error('创建文件夹失败: ' + error)
    }
  }).catch(() => {})
}

async function renameFromMenu() {
  const oldPath = contextMenuData.value?.path
  const oldName = oldPath?.split('/').pop()

  ElMessageBox.prompt('请输入新名称', '重命名', {
    confirmButtonText: '确定',
    cancelButtonText: '取消',
    inputValue: oldName,
    inputPattern: /\S+/,
    inputErrorMessage: '名称不能为空'
  }).then(async ({ value }) => {
    try {
      const pathParts = oldPath.split('/')
      pathParts[pathParts.length - 1] = value
      const newPath = pathParts.join('/')

      await tauriApi.renameFile(projectId.value, oldPath, newPath)
      await loadProjectFiles()
      ElMessage.success('重命名成功')
    } catch (error) {
      ElMessage.error('重命名失败: ' + error)
    }
  }).catch(() => {})
}

async function deleteFromMenu() {
  const path = contextMenuData.value?.path
  const name = path?.split('/').pop()

  ElMessageBox.confirm(`确定要删除 "${name}" 吗？此操作不可恢复。`, '确认删除', {
    type: 'warning',
    confirmButtonText: '删除',
    cancelButtonText: '取消'
  }).then(async () => {
    try {
      await tauriApi.deleteFile(projectId.value, path)
      await loadProjectFiles()

      // 如果删除的是当前打开的文件，关闭它
      if (openFiles.value.some(f => f.path === path)) {
        closeFile(path)
      }

      ElMessage.success('已删除')
    } catch (error) {
      ElMessage.error('删除失败: ' + error)
    }
  }).catch(() => {})
}

// 拖拽功能
function checkAllowDrag(draggingNode) {
  return true
}

function checkAllowDrop(draggingNode, dropNode, type) {
  // 不允许拖到自己里面
  if (draggingNode.key === dropNode.key) {
    return false
  }
  // 只允许插入到文件夹中，或者作为兄弟节点
  if (type === 'inner') {
    return !dropNode.data.is_file
  }
  return true
}

async function handleDragEnd(draggingNode, dropNode, dropType, event) {
  if (!dropNode || draggingNode.key === dropNode.key) {
    return
  }

  const sourcePath = draggingNode.key
  let targetPath = ''

  if (dropType === 'inner') {
    // 拖入文件夹
    targetPath = `${dropNode.key}/${draggingNode.label}`
  } else {
    // 作为兄弟节点
    const dropPathParts = dropNode.key.split('/')
    dropPathParts[dropPathParts.length - 1] = draggingNode.label
    targetPath = dropPathParts.join('/')
  }

  try {
    await tauriApi.moveFile(projectId.value, sourcePath, targetPath)
    await loadProjectFiles()
    ElMessage.success('移动成功')
  } catch (error) {
    ElMessage.error('移动失败: ' + error)
    await loadProjectFiles()
  }
}

function toggleSidebar() {
  sidebarVisible.value = !sidebarVisible.value
}

// 面板拖拽调整
function startResize(type, event) {
  event.preventDefault()
  isResizing.value = true
  resizerType.value = type

  // 添加全局事件监听
  document.addEventListener('mousemove', handleResize)
  document.addEventListener('mouseup', stopResize)
}

function handleResize(event) {
  if (!isResizing.value) return

  if (resizerType.value === 'left') {
    // 拖拽左侧分隔条：调整文件树宽度
    const newWidth = event.clientX
    if (newWidth >= 150 && newWidth <= 500) {
      fileTreeWidth.value = newWidth
    }
  } else if (resizerType.value === 'right') {
    // 拖拽右侧分隔条：调整AI侧边栏宽度
    const containerWidth = document.querySelector('.main-content').offsetWidth
    const newWidth = containerWidth - event.clientX
    if (newWidth >= 300 && newWidth <= 800) {
      aiSidebarWidth.value = newWidth
    }
  }
}

function stopResize() {
  isResizing.value = false
  resizerType.value = ''

  // 移除全局事件监听
  document.removeEventListener('mousemove', handleResize)
  document.removeEventListener('mouseup', stopResize)
}

function getPlaceholder() {
  switch (aiMode.value) {
    case 'chat':
      return '问AI任何技术问题...'
    case 'requirement':
      return '描述你的需求，AI会更新需求文档...'
    case 'create':
      return '告诉AI你想创建什么文件，例如："创建一个 main.py 入口文件"...'
    default:
      return '输入消息...'
  }
}

function selectFile(filePath) {
  selectedFile.value = filePath
  loadFileContent(filePath)
}

async function loadFileContent(filePath) {
  try {
    const content = await tauriApi.getSourceFile(projectId.value, filePath)
    fileContent.value = content
    originalContent.value = content
    unsavedChanges.value = false
  } catch (error) {
    ElMessage.error('加载文件失败: ' + error)
  }
}

function closeFile(filePath) {
  const index = openFiles.value.findIndex(f => f.path === filePath)
  if (index > -1) {
    openFiles.value.splice(index, 1)
  }

  if (selectedFile.value === filePath) {
    selectedFile.value = ''
    fileContent.value = ''
  }
}

function onCodeChange(newCode) {
  fileContent.value = newCode
  unsavedChanges.value = newCode !== originalContent.value
}

async function saveCurrentFile() {
  if (!selectedFile.value || !unsavedChanges.value) return

  try {
    await tauriApi.saveSourceFile(
      projectId.value,
      selectedFile.value,
      fileContent.value
    )
    originalContent.value = fileContent.value
    unsavedChanges.value = false
    ElMessage.success('文件已保存')
  } catch (error) {
    ElMessage.error('保存失败: ' + error)
  }
}

async function createNewFile() {
  const filename = newFileInfo.value.name.trim()
  if (!filename) {
    ElMessage.warning('请输入文件名')
    return
  }

  const path = newFileInfo.value.path.trim()
  const fullPath = path ? `${path}/${filename}`.replace(/\/+/g, '/') : filename

  if (openFiles.value.some(f => f.path === fullPath)) {
    ElMessage.warning('文件已打开')
    return
  }

  try {
    // 创建空文件
    await tauriApi.createFile(projectId.value, fullPath, '')

    // 刷新文件树
    await loadProjectFiles()

    // 添加到打开的文件列表
    const name = filename.split('/').pop()
    openFiles.value.push({ name, path: fullPath })

    // 选中新文件
    selectFile(fullPath)

    showNewFileDialog.value = false
    newFileInfo.value = { name: '', path: '' }

    ElMessage.success('文件已创建')
  } catch (error) {
    ElMessage.error('创建文件失败: ' + error)
  }
}

async function sendMessage() {
  const message = userInput.value.trim()
  if (!message || isLoading.value) return

  // 添加用户消息到当前模式的会话历史
  chatHistory.value[aiMode.value].push({
    role: 'user',
    content: message
  })
  userInput.value = ''
  scrollToBottom()

  isLoading.value = true

  try {
    await simulateAIResponse(message)
  } catch (error) {
    ElMessage.error('AI 响应失败: ' + error)
  } finally {
    isLoading.value = false
  }
}

async function simulateAIResponse(userMessage) {
  let aiResponse = ''

  switch (aiMode.value) {
    case 'requirement':
      // 使用 Claude Agent 更新需求文档
      try {
        const response = await tauriApi.updateRequirementWithAgent(
          projectId.value,
          userMessage
        )

        if (response.success) {
          // 需求文档会通过事件自动刷新
          aiResponse = response.message || '需求文档已更新，请查看左侧内容。'
        } else {
          aiResponse = '更新需求文档失败：' + (response.message || '未知错误')
        }
      } catch (error) {
        console.error('调用 Claude Agent 失败:', error)
        // 检查是否是 API Key 未配置的错误
        if (error.includes('API key')) {
          aiResponse = '错误：未配置 Claude API Key。\n\n请先在设置中配置你的 API Key。'
        } else {
          aiResponse = '调用 Claude Agent 失败：' + error
        }
      }
      break

    case 'create':
      // 使用 OpenCode Agent 创建文件（异步版本 - 实时显示执行过程）
      try {
        // 添加开始消息
        const startMessage = {
          role: 'assistant',
          content: '🚀 开始处理你的请求...',
          isProgress: true
        }
        chatHistory.value.create.push(startMessage)
        scrollToBottom()

        // 使用异步 API，立即返回 session_id
        const sessionId = await tauriApi.createFilesWithAgentAsync(
          projectId.value,
          userMessage
        )

        console.log('会话已创建，ID:', sessionId)

        // 更新消息，开始轮询
        const pollingMessage = {
          role: 'assistant',
          content: '⏳ AI Agent 正在工作，正在执行任务...\n\n你可以看到详细的执行过程。',
          isProgress: true
        }
        // 替换最后一条消息
        chatHistory.value.create[chatHistory.value.create.length - 1] = pollingMessage
        scrollToBottom()

        // 轮询获取消息，显示实时对话过程
        let lastMessageCount = 0
        let completed = false
        let maxAttempts = 120 // 最多轮询 2 分钟（每秒一次）
        let attempts = 0

        while (!completed && attempts < maxAttempts) {
          attempts++
          await new Promise(resolve => setTimeout(resolve, 1000)) // 等待 1 秒

          try {
            const messages = await tauriApi.getSessionMessages(sessionId, 50)

            // 如果有新消息
            if (messages.length > lastMessageCount) {
              // 清除之前的进度消息
              chatHistory.value.create = chatHistory.value.create.filter(msg => !msg.isProgress)

              // 添加新的消息
              for (let i = lastMessageCount; i < messages.length; i++) {
                const msg = messages[i]
                const content = msg.parts
                  ?.map(part => part.text || part.reasoning || '')
                  .join('\n') || ''

                if (content) {
                  chatHistory.value.create.push({
                    role: msg.role === 'user' ? 'user' : 'assistant',
                    content: content
                  })
                }
              }

              lastMessageCount = messages.length
              scrollToBottom()

              // 检查是否完成（最后一条消息状态为 completed）
              const lastMsg = messages[messages.length - 1]
              if (lastMsg && lastMsg.status === 'completed') {
                completed = true
              }
            }
          } catch (pollError) {
            console.error('轮询消息失败:', pollError)
            // 继续轮询，不要中断
          }
        }

        // 清除进度消息
        chatHistory.value.create = chatHistory.value.create.filter(msg => !msg.isProgress)

        // 刷新文件树
        await loadProjectFiles()

        // 添加完成消息
        if (completed) {
          aiResponse = '✅ 任务已完成！\n\n请查看上方对话了解详细执行过程，文件已更新到左侧文件树。'
        } else {
          aiResponse = '⚠️ 任务仍在后台执行中，你可以稍后查看结果。\n\n请刷新文件树查看最新变化。'
        }

        // 超时删除临时会话
        try {
          // 可选：调用删除会话 API
        } catch (e) {
          console.error('删除会话失败:', e)
        }
      } catch (error) {
        console.error('调用 OpenCode Agent 失败:', error)
        // 移除进度消息
        chatHistory.value.create = chatHistory.value.create.filter(msg => !msg.isProgress)

        if (error.includes('API key') || error.includes('配置')) {
          aiResponse = '❌ 错误：请先在设置中配置 OpenCode Server。'
        } else {
          aiResponse = '❌ 调用 OpenCode Agent 失败：\n\n' + error
        }
      }
      break

    case 'chat':
    default:
      // 普通对话
      if (userMessage.includes('保存') && unsavedChanges.value) {
        await saveCurrentFile()
        aiResponse = '文件已保存！'
      } else if (userMessage.includes('运行') || userMessage.includes('执行')) {
        aiResponse = '执行功能开发中...目前请手动运行代码。'
      } else if (userMessage.includes('bug') || userMessage.includes('错误')) {
        aiResponse = '请将错误信息和相关代码发给我，我会帮你分析。'
      } else {
        aiResponse = `我收到你的消息："${userMessage}"。\n\n我正在学习如何更好地帮助你！目前你可以：\n• 切换到"创建文件"模式让AI帮你生成代码\n• 切换到"需求文档"模式让AI更新需求`
      }
      break
  }

  // 添加助手回复到当前模式的会话历史
  chatHistory.value[aiMode.value].push({
    role: 'assistant',
    content: aiResponse
  })

  scrollToBottom()
}

function scrollToBottom() {
  nextTick(() => {
    if (chatContainer.value) {
      chatContainer.value.scrollTop = chatContainer.value.scrollHeight
    }
  })
}

function getLanguage(filename) {
  const ext = filename.split('.').pop()
  const langMap = {
    'py': 'python',
    'js': 'javascript',
    'ts': 'typescript',
    'go': 'go',
    'rs': 'rust',
    'java': 'java',
    'cpp': 'cpp',
    'c': 'c',
    'h': 'c'
  }
  return langMap[ext] || 'python'
}

async function switchMode(mode) {
  aiMode.value = mode

  if (mode === 'requirement') {
    // 重新加载需求文档
    await loadRequirement()
  }
}

function goBack() {
  if (unsavedChanges.value) {
    ElMessageBox.confirm(
      '有未保存的更改，确定要离开吗？',
      '确认',
      {
        confirmButtonText: '离开',
        cancelButtonText: '取消',
        type: 'warning'
      }
    ).then(() => {
      router.push('/')
    }).catch(() => {
      // 用户取消，不做任何事
    })
  } else {
    router.push('/')
  }
}
</script>

<style scoped>
.project-container {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.toolbar {
  height: 50px;
  display: flex;
  align-items: center;
  padding: 0 20px;
  border-bottom: 1px solid #e4e7ed;
  gap: 10px;
}

.project-name {
  font-size: 16px;
  font-weight: bold;
}

.main-content {
  flex: 1;
  display: flex;
  overflow: hidden;
  position: relative;
}

.file-tree-panel {
  display: flex;
  flex-direction: column;
  border-right: 1px solid #e4e7ed;
  background-color: #fafafa;
  overflow: hidden;
  flex-shrink: 0;
}

/* 分隔条 */
.resizer {
  width: 4px;
  background-color: #e4e7ed;
  cursor: col-resize;
  transition: background-color 0.2s;
  flex-shrink: 0;
  position: relative;
  z-index: 10;
}

.resizer:hover {
  background-color: #409eff;
}

.resizer-left {
  /* 左侧分隔条样式 */
}

.resizer-right {
  /* 右侧分隔条样式 */
}

.file-tree-header {
  height: 45px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 15px;
  background-color: #f5f7fa;
  border-bottom: 1px solid #e4e7ed;
  font-weight: bold;
  font-size: 14px;
  color: #303133;
}

.file-tree-content {
  flex: 1;
  overflow-y: auto;
  padding: 10px;
}

.tree-node {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
}

:deep(.el-tree-node__content) {
  height: 32px;
  padding-left: 10px;
}

:deep(.el-tree-node__content:hover) {
  background-color: #ecf5ff;
}

:deep(.el-tree-node:focus > .el-tree-node__content) {
  background-color: #ecf5ff;
}

.editor-area {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.file-selector {
  display: flex;
  align-items: center;
  padding: 0 15px;
  background-color: #f5f7fa;
  border-bottom: 1px solid #e4e7ed;
  min-height: 45px;
}

.file-tabs {
  flex: 1;
  display: flex;
  gap: 5px;
  overflow-x: auto;
}

.file-tab {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  background-color: white;
  border: 1px solid #dcdfe6;
  border-radius: 4px 4px 0 0;
  cursor: pointer;
  font-size: 13px;
  white-space: nowrap;
  transition: all 0.2s;
}

.file-tab:hover {
  background-color: #ecf5ff;
}

.file-tab.active {
  background-color: #409eff;
  color: white;
  border-color: #409eff;
}

.close-tab {
  font-size: 12px;
  opacity: 0.7;
}

.close-tab:hover {
  opacity: 1;
}

.code-editor {
  flex: 1;
  overflow: hidden;
}

.ai-sidebar {
  display: flex;
  flex-direction: column;
  border-left: 1px solid #e4e7ed;
  background-color: #fafafa;
  flex-shrink: 0;
}

/* 侧边栏收起时的触发按钮 */
.sidebar-toggle-btn {
  position: fixed;
  right: 0;
  top: 50%;
  transform: translateY(-50%);
  width: 40px;
  height: 100px;
  background-color: #409eff;
  border-radius: 8px 0 0 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  color: white;
  box-shadow: -2px 0 8px rgba(0, 0, 0, 0.15);
  transition: all 0.3s ease;
  z-index: 1000;
}

.sidebar-toggle-btn:hover {
  background-color: #66b1ff;
  width: 45px;
}

.sidebar-content {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.sidebar-header {
  height: 50px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 15px;
  background-color: #409eff;
  color: white;
  font-weight: bold;
}

.mode-selector {
  padding: 10px 15px;
  background-color: white;
  border-bottom: 1px solid #e4e7ed;
}

.mode-header {
  padding: 10px 15px;
  font-size: 13px;
  font-weight: bold;
  color: #606266;
  background-color: white;
}

.requirement-mode {
  flex: 1;
  overflow-y: auto;
  padding: 0 15px 15px;
}

.requirement-preview {
  padding: 15px;
  background-color: white;
  border-radius: 8px;
  font-size: 13px;
  line-height: 1.6;
}

.requirement-preview h1 {
  font-size: 18px;
  margin-bottom: 10px;
  color: #303133;
}

.requirement-preview h2 {
  font-size: 16px;
  margin: 10px 0 8px;
  color: #303133;
}

.requirement-preview p,
.requirement-preview li {
  margin: 5px 0;
  color: #606266;
}

.create-mode {
  flex: 1;
  overflow-y: auto;
  padding: 0 15px 15px;
}

.create-hint {
  padding: 15px;
  background-color: white;
  border-radius: 8px;
  font-size: 13px;
  line-height: 1.8;
}

.create-hint p {
  margin: 8px 0;
  color: #606266;
}

.chat-messages {
  flex: 1;
  overflow-y: auto;
  padding: 15px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.message {
  max-width: 85%;
  padding: 10px 14px;
  border-radius: 8px;
  font-size: 13px;
  line-height: 1.5;
}

.message.user {
  align-self: flex-end;
  background-color: #409eff;
  color: white;
  margin-left: auto;
}

.message.assistant {
  align-self: flex-start;
  background-color: white;
  border: 1px solid #e4e7ed;
  color: #303133;
}

.message-content {
  white-space: pre-wrap;
  word-break: break-word;
}

/* 进度消息样式 */
.message.progress-message {
  background-color: #f0f9ff;
  border: 1px solid #91caff;
  opacity: 0.9;
}

.message.loading-message {
  background-color: #f0f9ff;
  border: 1px solid #91caff;
}

.loading-dots {
  display: inline-block;
  animation: pulse 1.5s ease-in-out infinite;
}

@keyframes pulse {
  0%, 100% {
    opacity: 1;
  }
  50% {
    opacity: 0.5;
  }
}

.chat-input {
  padding: 15px;
  background-color: white;
  border-top: 1px solid #e4e7ed;
}

.chat-input .el-textarea {
  margin-bottom: 10px;
}

.chat-input :deep(.el-textarea__inner) {
  resize: none;
  font-size: 13px;
}

.input-actions {
  display: flex;
  justify-content: flex-end;
}
</style>
