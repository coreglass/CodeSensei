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
      <div class="file-tree-panel">
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
          <el-button size="small" :icon="Plus" @click="showNewFileDialog = true">
            新建文件
          </el-button>
        </div>

        <div class="code-editor">
          <MonacoEditor
            v-if="selectedFile"
            :code="fileContent"
            :language="getLanguage(selectedFile)"
            @change="onCodeChange"
            @save="saveCurrentFile"
          />
          <el-empty v-else description="选择或创建文件开始编辑">
            <template #default>
              <el-button type="primary" @click="showNewFileDialog = true">新建第一个文件</el-button>
            </template>
          </el-empty>
        </div>
      </div>

      <!-- 右侧：AI对话侧边栏 -->
      <div :class="['ai-sidebar', { collapsed: !sidebarVisible }]">
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

        <!-- 需求文档模式 -->
        <div v-if="aiMode === 'requirement'" class="requirement-mode">
          <div class="mode-header">
            <span>当前需求文档预览</span>
          </div>
          <div class="requirement-preview" v-html="renderedRequirement"></div>
        </div>

        <!-- 创建文件模式 -->
        <div v-if="aiMode === 'create'" class="create-mode">
          <div class="mode-header">
            <span>文件创建向导</span>
          </div>
          <div class="create-hint">
            <p>💡 告诉AI你想创建什么文件，例如：</p>
            <p>"创建一个 main.py 作为入口文件"</p>
            <p>"添加一个用户管理的 user.go 文件"</p>
            <p>"创建一个处理数据的 utils.js"</p>
          </div>
        </div>

        <!-- 聊天历史 -->
        <div class="chat-messages" ref="chatContainer">
          <div
            v-for="(msg, index) in chatHistory"
            :key="index"
            :class="['message', msg.role]"
          >
            <div class="message-content">{{ msg.content }}</div>
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
import { ref, onMounted, computed, nextTick } from 'vue'
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

// AI对话
const userInput = ref('')
const chatHistory = ref([
  {
    role: 'assistant',
    content: '你好！我是你的AI助手。\n\n我可以帮你：\n• 💬 回答技术问题\n• 📄 创建新的代码文件\n• 📝 更新需求文档\n• 🔧 修复代码bug\n\n选择上方的模式，然后告诉我你的需求！'
  }
])
const isLoading = ref(false)
const chatContainer = ref(null)

// 需求文档
const requirementContent = ref('')

onMounted(async () => {
  await loadProjectInfo()
  await loadRequirement()
  await loadProjectFiles()
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
    requirementContent.value = await tauriApi.readProjectFile(projectId.value, 'requirement')
  } catch (error) {
    requirementContent.value = '# 需求文档\n\n暂无需求文档，请在右侧AI助手中创建。'
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

  // 添加用户消息
  chatHistory.value.push({
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
  await new Promise(resolve => setTimeout(resolve, 1500))

  let aiResponse = ''

  switch (aiMode.value) {
    case 'requirement':
      // 更新需求文档
      if (userMessage.includes('添加') || userMessage.includes('新增')) {
        requirementContent.value += `\n- ${userMessage}`
        aiResponse = `已将你的需求添加到需求文档中，请查看左侧内容。`
      } else if (userMessage.includes('修改') || userMessage.includes('更改')) {
        requirementContent.value += `\n\n- ${userMessage}`
        aiResponse = `需求文档已更新。`
      } else {
        requirementContent.value += `\n\n- ${userMessage}`
        aiResponse = `好的，我已更新需求文档。`
      }

      // 保存需求文档
      await tauriApi.writeProjectFile(
        projectId.value,
        'requirement',
        requirementContent.value
      )
      break

    case 'create':
      // 创建文件
      const filenameMatch = userMessage.match(/(\w+\.(?:py|js|ts|go|rs|java|cpp|c|h))/i)
      if (filenameMatch) {
        const filename = filenameMatch[0]
        const codeMap = {
          'py': `# ${filename}\n\ndef main():\n    print("Hello, World!")\n\nif __name__ == "__main__":\n    main()`,
          'js': `// ${filename}\n\nfunction main() {\n    console.log("Hello, World!");\n}\n\nmain();`,
          'ts': `// ${filename}\n\nfunction main(): void {\n    console.log("Hello, World!");\n}\n\nmain();`,
          'go': `package main\n\nimport "fmt"\n\nfunc main() {\n\tfmt.Println("Hello, World!")\n}`,
          'rs': `fn main() {\n    println!("Hello, World!");\n}`,
        }

        const ext = filename.split('.').pop()
        const code = codeMap[ext] || codeMap['py']

        await tauriApi.createFile(projectId.value, filename, code)

        // 刷新文件树
        await loadProjectFiles()

        // 添加到打开的文件
        if (!openFiles.value.some(f => f.path === filename)) {
          openFiles.value.push({ name: filename, path: filename })
        }

        // 选中新文件
        await selectFile(filename)

        aiResponse = `已创建文件：${filename}\n\n文件已自动打开，你可以开始编辑了。`
      } else {
        aiResponse = `请告诉我你想创建什么文件，例如：\n\n• "创建 main.py 入口文件"\n• "添加 utils.js 工具函数"\n• "创建 User.ts 用户类"`
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

  chatHistory.value.push({
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

const renderedRequirement = computed(() => {
  if (!requirementContent.value) return ''

  return requirementContent.value
    .replace(/^# (.*$)/gim, '<h1>$1</h1>')
    .replace(/^## (.*$)/gim, '<h2>$1</h2>')
    .replace(/^### (.*$)/gim, '<h3>$1</h3>')
    .replace(/\*\*(.*?)\*\*/g, '<strong>$1</strong>')
    .replace(/^- (.*$)/gim, '<li>$1</li>')
    .replace(/\n\n/g, '</p><p>')
    .replace(/\n/g, '<br>')
})

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
}

.file-tree-panel {
  width: 250px;
  display: flex;
  flex-direction: column;
  border-right: 1px solid #e4e7ed;
  background-color: #fafafa;
  overflow: hidden;
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
  width: 400px;
  display: flex;
  flex-direction: column;
  border-left: 1px solid #e4e7ed;
  background-color: #fafafa;
  transition: all 0.3s ease;
}

.ai-sidebar.collapsed {
  width: 0;
  border-left: none;
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
