<template>
  <div class="task-execution-container">
    <div class="toolbar">
      <el-button :icon="ArrowLeft" @click="goBack">返回</el-button>
      <el-divider direction="vertical" />
      <span class="project-name">{{ projectName }}</span>
      <el-divider direction="vertical" />
      <el-button type="primary" :icon="Check" @click="executeSelectedTasks" :disabled="!hasSelectedTasks">
        执行选中任务 ({{ selectedCount }})
      </el-button>
      <el-button :icon="Refresh" @click="regenerateTasks">重新生成任务</el-button>
      <el-button :icon="Edit" @click="backToRequirement">修改需求</el-button>
    </div>

    <div class="main-content">
      <!-- 左侧任务列表 -->
      <div class="task-list-panel">
        <div class="panel-header">
          <span>任务列表</span>
          <div class="header-actions">
            <el-checkbox
              v-model="selectAll"
              :indeterminate="isIndeterminate"
              @change="handleSelectAll"
            >
              全选
            </el-checkbox>
          </div>
        </div>
        <div class="task-list">
          <div
            v-for="task in tasks"
            :key="task.id"
            :class="['task-item', task.status, { selected: selectedTasks.includes(task.id) }]"
            @click="toggleTaskSelection(task.id)"
          >
            <div class="task-checkbox">
              <el-checkbox v-model="selectedTasks" :value="task.id" @change="handleTaskCheck" />
            </div>
            <div class="task-content">
              <div class="task-header">
                <span class="task-title">{{ task.title }}</span>
                <el-tag :type="getStatusType(task.status)" size="small">
                  {{ getStatusText(task.status) }}
                </el-tag>
              </div>
              <div class="task-description">{{ task.description }}</div>
            </div>
            <div class="task-actions">
              <el-button
                v-if="task.status === 'completed'"
                size="small"
                :icon="View"
                @click.stop="viewTaskResult(task)"
              >
                查看结果
              </el-button>
            </div>
          </div>
          <el-empty v-if="tasks.length === 0" description="暂无任务，请先生成任务列表" />
        </div>
      </div>

      <!-- 右侧执行日志 -->
      <div class="execution-log-panel">
        <div class="panel-header">
          <span>执行日志</span>
          <el-button size="small" @click="clearLog">清空</el-button>
        </div>
        <div class="log-content" ref="logContainer">
          <div
            v-for="(log, index) in logs"
            :key="index"
            :class="['log-item', log.type]"
          >
            <span class="log-time">{{ formatTime(log.timestamp) }}</span>
            <span class="log-message">{{ log.message }}</span>
          </div>
          <div v-if="isExecuting" class="log-item info">
            <span class="log-time">{{ formatTime(Date.now()) }}</span>
            <span class="log-message">正在执行...</span>
          </div>
        </div>
      </div>
    </div>

    <!-- 重新生成任务弹窗 -->
    <el-dialog v-model="showRegenerateDialog" title="重新生成任务" width="600px">
      <el-form label-width="100px">
        <el-form-item label="API Key">
          <el-input v-model="apiConfig.apiKey" type="password" show-password />
        </el-form-item>
        <el-form-item label="API Base">
          <el-input v-model="apiConfig.apiBase" />
        </el-form-item>
        <el-form-item label="模型">
          <el-select v-model="apiConfig.model">
            <el-option label="智谱 GLM-4" value="glm-4" />
            <el-option label="智谱 GLM-4-Flash" value="glm-4-flash" />
            <el-option label="通义千问" value="qwen-plus" />
          </el-select>
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="showRegenerateDialog = false">取消</el-button>
        <el-button type="primary" @click="confirmRegenerate" :loading="isRegenerating">
          重新生成
        </el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { ArrowLeft, Check, Refresh, Edit, View } from '@element-plus/icons-vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import * as tauriApi from '../api/tauri'

const route = useRoute()
const router = useRouter()

const projectId = computed(() => route.params.id)
const projectName = ref('项目')
const tasks = ref([])
const selectedTasks = ref([])
const logs = ref([])
const isExecuting = ref(false)
const isRegenerating = ref(false)
const showRegenerateDialog = ref(false)
const selectAll = ref(false)
const logContainer = ref(null)

const apiConfig = ref({
  apiKey: '',
  apiBase: 'https://open.bigmodel.cn/api/paas/v4',
  model: 'glm-4-flash'
})

const hasSelectedTasks = computed(() => selectedTasks.value.length > 0)
const selectedCount = computed(() => selectedTasks.value.length)
const isIndeterminate = computed(() => {
  const count = selectedTasks.value.length
  return count > 0 && count < tasks.value.length
})

onMounted(async () => {
  await loadProjectInfo()
  await loadTasks()
})

async function loadProjectInfo() {
  try {
    const project = await tauriApi.getProject(projectId.value)
    projectName.value = project.name
  } catch (error) {
    ElMessage.error('加载项目信息失败')
  }
}

async function loadTasks() {
  try {
    const result = await tauriApi.getTasks(projectId.value)
    tasks.value = result.map(([id, title, description, status, order,]) => ({
      id,
      title,
      description,
      status,
      order
    }))
  } catch (error) {
    console.error('加载任务失败:', error)
  }
}

function toggleTaskSelection(taskId) {
  const index = selectedTasks.value.indexOf(taskId)
  if (index > -1) {
    selectedTasks.value.splice(index, 1)
  } else {
    selectedTasks.value.push(taskId)
  }
}

function handleSelectAll(checked) {
  if (checked) {
    selectedTasks.value = tasks.value.map(t => t.id)
  } else {
    selectedTasks.value = []
  }
}

function handleTaskCheck() {
  selectAll.value = selectedTasks.value.length === tasks.value.length && tasks.value.length > 0
}

async function executeSelectedTasks() {
  if (!hasSelectedTasks.value) return

  try {
    await ElMessageBox.confirm(
      `确定要执行选中的 ${selectedCount.value} 个任务吗？`,
      '确认执行',
      { type: 'warning' }
    )

    isExecuting.value = true
    addLog('info', `开始执行 ${selectedCount.value} 个任务...`)

    for (const taskId of selectedTasks.value) {
      const task = tasks.value.find(t => t.id === taskId)
      if (task && task.status !== 'completed') {
        await executeTask(task)
      }
    }

    addLog('success', '所有任务执行完成！')
    await loadTasks()
  } catch (error) {
    if (error !== 'cancel') {
      addLog('error', '任务执行失败: ' + error)
    }
  } finally {
    isExecuting.value = false
  }
}

async function executeTask(task) {
  addLog('info', `执行任务: ${task.title}`)

  try {
    // 更新任务状态为进行中
    await tauriApi.updateTaskStatus({ taskId: task.id, status: 'in_progress' })
    task.status = 'in_progress'

    // TODO: 实际执行任务逻辑
    // 这里可以根据任务类型调用不同的 API
    await new Promise(resolve => setTimeout(resolve, 2000))

    // 更新任务状态为完成
    await tauriApi.updateTaskStatus({ taskId: task.id, status: 'completed' })
    task.status = 'completed'

    addLog('success', `任务 "${task.title}" 执行完成`)
  } catch (error) {
    await tauriApi.updateTaskStatus({ taskId: task.id, status: 'failed' })
    task.status = 'failed'
    addLog('error', `任务 "${task.title}" 执行失败: ${error}`)
    throw error
  }
}

function regenerateTasks() {
  showRegenerateDialog.value = true
}

async function confirmRegenerate() {
  if (!apiConfig.value.apiKey) {
    ElMessage.warning('请输入 API Key')
    return
  }

  try {
    isRegenerating.value = true

    // 获取需求文档
    const doc = await tauriApi.getRequirementDoc(projectId.value)
    if (!doc) {
      ElMessage.warning('请先完善需求文档')
      return
    }

    const requirement = doc[1]

    // 生成新任务
    await tauriApi.generateTasks({
      projectId: projectId.value,
      requirement,
      ...apiConfig.value
    })

    await loadTasks()
    selectedTasks.value = []

    showRegenerateDialog.value = false
    ElMessage.success('任务列表已重新生成')
  } catch (error) {
    ElMessage.error('生成任务失败: ' + error)
  } finally {
    isRegenerating.value = false
  }
}

function backToRequirement() {
  router.push(`/requirement/${projectId.value}`)
}

function viewTaskResult(task) {
  ElMessage.info(`查看任务 "${task.title}" 的结果`)
  // TODO: 实现查看任务结果的功能
}

function addLog(type, message) {
  logs.value.push({
    type,
    message,
    timestamp: Date.now()
  })

  // 自动滚动到底部
  setTimeout(() => {
    if (logContainer.value) {
      logContainer.value.scrollTop = logContainer.value.scrollHeight
    }
  }, 100)
}

function clearLog() {
  logs.value = []
}

function formatTime(timestamp) {
  return new Date(timestamp).toLocaleTimeString('zh-CN')
}

function getStatusType(status) {
  const types = {
    pending: 'info',
    in_progress: 'warning',
    completed: 'success',
    failed: 'danger'
  }
  return types[status] || 'info'
}

function getStatusText(status) {
  const texts = {
    pending: '待执行',
    in_progress: '进行中',
    completed: '已完成',
    failed: '失败'
  }
  return texts[status] || status
}

function goBack() {
  router.push('/')
}
</script>

<style scoped>
.task-execution-container {
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

.task-list-panel {
  flex: 1;
  display: flex;
  flex-direction: column;
  border-right: 1px solid #e4e7ed;
  min-width: 400px;
}

.execution-log-panel {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 300px;
}

.panel-header {
  height: 50px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 20px;
  background-color: #f5f7fa;
  border-bottom: 1px solid #e4e7ed;
  font-weight: bold;
}

.task-list {
  flex: 1;
  overflow-y: auto;
}

.task-item {
  padding: 15px;
  border-bottom: 1px solid #e4e7ed;
  cursor: pointer;
  transition: background-color 0.2s;
  display: flex;
  gap: 10px;
}

.task-item:hover {
  background-color: #f5f7fa;
}

.task-item.selected {
  background-color: #ecf5ff;
}

.task-item.completed {
  opacity: 0.8;
}

.task-checkbox {
  display: flex;
  align-items: flex-start;
  padding-top: 2px;
}

.task-content {
  flex: 1;
}

.task-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 8px;
}

.task-title {
  font-weight: bold;
  font-size: 15px;
}

.task-description {
  color: #606266;
  font-size: 13px;
  line-height: 1.5;
}

.task-actions {
  display: flex;
  align-items: center;
}

.log-content {
  flex: 1;
  overflow-y: auto;
  padding: 15px;
  background-color: #1e1e1e;
  font-family: 'Courier New', monospace;
  font-size: 13px;
}

.log-item {
  margin-bottom: 8px;
  display: flex;
  gap: 10px;
}

.log-time {
  color: #858585;
  min-width: 80px;
}

.log-message {
  flex: 1;
}

.log-item.info .log-message {
  color: #409eff;
}

.log-item.success .log-message {
  color: #67c23a;
}

.log-item.error .log-message {
  color: #f56c6c;
}
</style>
