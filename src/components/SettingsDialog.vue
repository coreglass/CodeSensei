<template>
  <el-dialog
    v-model="visible"
    title="OpenCode 设置"
    width="700px"
    :close-on-click-modal="false"
  >
    <el-form :model="form" label-width="140px">
      <!-- Server 配置 -->
      <el-divider content-position="left">
        <el-icon><Connection /></el-icon>
        Server 配置
      </el-divider>

      <el-form-item label="Server 地址" required>
        <el-input
          v-model="form.serverUrl"
          placeholder="http://localhost:4096"
        />
        <div class="hint">
          OpenCode Server 的 HTTP 地址。默认: http://localhost:4096
        </div>
      </el-form-item>

      <el-form-item label="用户名">
        <el-input
          v-model="form.username"
          placeholder="opencode"
        />
        <div class="hint">
          Server 认证用户名，默认为 "opencode"
        </div>
      </el-form-item>

      <el-form-item label="密码（可选）">
        <el-input
          v-model="form.password"
          type="password"
          show-password
          placeholder="如果不设置密码则留空"
        />
        <div class="hint">
          如果 Server 设置了密码，请在此填写
        </div>
      </el-form-item>

      <el-form-item>
        <el-button
          type="primary"
          @click="testConnection"
          :loading="testing"
          :icon="testSuccess ? SuccessFilled : Connection"
          :type="testSuccess ? 'success' : 'primary'"
        >
          {{ testButtonText }}
        </el-button>
      </el-form-item>

      <!-- AI Provider 配置 -->
      <el-divider content-position="left">
        <el-icon><MagicStick /></el-icon>
        AI Provider 配置
      </el-divider>

      <el-form-item label="刷新提供商">
        <el-button
          @click="refreshProviders"
          :loading="loadingProviders"
          :icon="Refresh"
        >
          刷新可用的提供商
        </el-button>
        <div class="hint">
          点击刷新按钮从 OpenCode Server 获取所有可用的 AI 提供商
        </div>
      </el-form-item>

      <el-form-item label="Provider" v-if="providers.length > 0">
        <el-select
          v-model="form.provider"
          placeholder="选择 Provider"
          clearable
          @change="onProviderChange"
          style="width: 100%"
        >
          <el-option
            v-for="provider in providers"
            :key="provider.id"
            :label="provider.display_name || provider.name"
            :value="provider.id"
          >
            <div style="display: flex; justify-content: space-between; align-items: center;">
              <span>{{ provider.display_name || provider.name }}</span>
              <el-tag v-if="provider.models && provider.models.length > 0" size="small" type="info">
                {{ provider.models.length }} 个模型
              </el-tag>
            </div>
          </el-option>
        </el-select>
        <div class="hint">
          {{ getProviderHint() }}
        </div>
      </el-form-item>

      <el-form-item label="模型" v-if="currentModels.length > 0">
        <el-select
          v-model="form.model"
          placeholder="选择模型"
          clearable
          style="width: 100%"
        >
          <el-option
            v-for="model in currentModels"
            :key="model"
            :label="model"
            :value="model"
          >
            {{ model }}
          </el-option>
        </el-select>
        <div class="hint">
          选择 {{ providers.find(p => p.id === form.provider)?.display_name || provider }} 提供的模型
        </div>
      </el-form-item>

      <!-- 快速选择 -->
      <el-form-item label="快速选择" v-if="providers.length > 0">
        <el-space wrap>
          <el-button
            size="small"
            @click="setQuickModel('openai', 'gpt-4')"
            :disabled="!hasProvider('openai')"
          >
            GPT-4
          </el-button>
          <el-button
            size="small"
            @click="setQuickModel('openai', 'gpt-4o')"
            :disabled="!hasProvider('openai')"
          >
            GPT-4o
          </el-button>
          <el-button
            size="small"
            @click="setQuickModel('anthropic', 'claude-3-5-sonnet-20241022')"
            :disabled="!hasProvider('anthropic')"
          >
            Claude 3.5 Sonnet
          </el-button>
          <el-button
            size="small"
            @click="setQuickModel('zhipuai', 'glm-4.7')"
            :disabled="!hasProvider('zhipuai')"
          >
            GLM 4.7
          </el-button>
        </el-space>
      </el-form-item>

      <!-- 状态信息 -->
      <el-form-item label="配置状态">
        <el-tag :type="isConfigured ? 'success' : 'warning'">
          {{ isConfigured ? '✅ 已配置' : '⚠️ 未完成配置' }}
        </el-tag>
        <div v-if="serverVersion" class="hint">
          Server 版本: {{ serverVersion }}
        </div>
      </el-form-item>

      <!-- 帮助信息 -->
      <el-alert
        type="info"
        :closable="false"
        show-icon
      >
        <template #title>
          <div style="font-size: 13px;">
            <strong>如何使用：</strong><br/>
            1. 安装 OpenCode: <code>npm install -g @opencode/opencode</code><br/>
            2. 启动 Server: <code>opencode serve --port 4096</code><br/>
            3. 在 OpenCode 中配置 AI Provider API Key（例如 Zhipu AI）<br/>
            4. 在此处配置 Server 地址并点击"刷新提供商"<br/>
            5. 从列表中选择 Provider 和 Model
          </div>
        </template>
      </el-alert>
    </el-form>

    <template #footer>
      <el-button @click="visible = false">取消</el-button>
      <el-button type="primary" @click="saveSettings" :loading="saving">
        保存配置
      </el-button>
    </template>
  </el-dialog>
</template>

<script setup>
import { ref, computed, watch, reactive } from 'vue'
import { ElMessage } from 'element-plus'
import { Connection, SuccessFilled, MagicStick, Refresh } from '@element-plus/icons-vue'
import * as tauriApi from '../api/tauri'

const props = defineProps({
  modelValue: {
    type: Boolean,
    required: true
  }
})

const emit = defineEmits(['update:modelValue'])

const visible = computed({
  get: () => props.modelValue,
  set: (val) => emit('update:modelValue', val)
})

const form = ref({
  serverUrl: 'http://localhost:4096',
  username: 'opencode',
  password: '',
  provider: '',
  model: ''
})

const saving = ref(false)
const testing = ref(false)
const testSuccess = ref(false)
const serverVersion = ref('')
const isConfigured = ref(false)
const loadingProviders = ref(false)

// Providers 数据
const providers = ref([])

// 当前选择的 Provider 的可用模型
const currentModels = computed(() => {
  if (!form.value.provider) return []
  const provider = providers.value.find(p => p.id === form.value.provider)
  return provider?.models || []
})

const testButtonText = computed(() => {
  if (testing.value) return '测试中...'
  if (testSuccess.value) return '✓ 连接成功'
  return '测试连接'
})

// 监听对话框打开，加载配置
watch(() => props.modelValue, async (val) => {
  if (val) {
    await loadConfig()
    // 自动刷新 providers（如果配置了 Server）
    if (form.value.serverUrl) {
      await refreshProviders()
    }
  }
})

async function loadConfig() {
  try {
    const config = await tauriApi.getOpenCodeConfig()
    form.value = {
      serverUrl: config.server_url || 'http://localhost:4096',
      username: config.username || 'opencode',
      password: config.password || '',
      provider: config.default_provider || '',
      model: config.default_model || ''
    }
    checkConfigured()
  } catch (error) {
    console.error('加载配置失败:', error)
  }
}

function checkConfigured() {
  // 检查基本配置是否完整
  isConfigured.value = !!(
    form.value.serverUrl &&
    form.value.username
  )
}

function hasProvider(providerId) {
  return providers.value.some(p => p.id === providerId)
}

function getProviderHint() {
  if (providers.value.length === 0) {
    return '请先点击"刷新可用的提供商"按钮获取 Provider 列表'
  }
  if (!form.value.provider) {
    return '请选择一个 AI Provider'
  }
  const provider = providers.value.find(p => p.id === form.value.provider)
  if (provider && provider.models && provider.models.length > 0) {
    return `可用模型：${provider.models.slice(0, 5).join(', ')}${provider.models.length > 5 ? '...' : ''}`
  }
  return '选择一个 Provider 后查看可用模型'
}

async function testConnection() {
  if (!form.value.serverUrl.trim()) {
    ElMessage.warning('请先输入 Server 地址')
    return
  }

  try {
    testing.value = true
    testSuccess.value = false
    serverVersion.value = ''

    const result = await tauriApi.testOpenCodeConnection(
      form.value.serverUrl.trim(),
      form.value.username.trim(),
      form.value.password || null
    )

    // 提取版本号
    const versionMatch = result.match(/版本:\s*(.+)/)
    if (versionMatch) {
      serverVersion.value = versionMatch[1]
    }

    testSuccess.value = true
    ElMessage.success(result)

    // 3秒后重置按钮状态
    setTimeout(() => {
      testSuccess.value = false
    }, 3000)
  } catch (error) {
    testSuccess.value = false
    ElMessage.error('连接失败: ' + error)
  } finally {
    testing.value = false
  }
}

async function refreshProviders() {
  if (!form.value.serverUrl.trim()) {
    ElMessage.warning('请先输入 Server 地址')
    return
  }

  try {
    loadingProviders.value = true

    const result = await tauriApi.getAvailableProviders(
      form.value.serverUrl.trim(),
      form.value.username.trim(),
      form.value.password || null
    )

    providers.value = result

    // 如果当前选择的 Provider 不在列表中，清空选择
    if (form.value.provider && !hasProvider(form.value.provider)) {
      form.value.provider = ''
      form.value.model = ''
    }

    // 如果当前模型不在可用模型列表中，清空模型选择
    if (form.value.model && !currentModels.value.includes(form.value.model)) {
      form.value.model = ''
    }

    ElMessage.success(`成功获取 ${result.length} 个 AI 提供商`)
  } catch (error) {
    providers.value = []
    ElMessage.error('获取提供商失败: ' + error)
  } finally {
    loadingProviders.value = false
  }
}

function onProviderChange() {
  // Provider 改变时，清空模型选择
  form.value.model = ''
}

function setQuickModel(provider, model) {
  if (!hasProvider(provider)) {
    ElMessage.warning(`Provider "${provider}" 不可用，请先刷新提供商列表`)
    return
  }

  form.value.provider = provider
  form.value.model = model

  const providerName = providers.value.find(p => p.id === provider)?.display_name || provider
  ElMessage.success(`已设置: ${providerName} / ${model}`)
}

async function saveSettings() {
  if (!form.value.serverUrl.trim()) {
    ElMessage.warning('请输入 Server 地址')
    return
  }

  // 验证 URL 格式
  try {
    new URL(form.value.serverUrl.trim())
  } catch {
    ElMessage.warning('Server 地址格式不正确，请输入有效的 URL')
    return
  }

  try {
    saving.value = true

    const config = {
      server_url: form.value.serverUrl.trim(),
      username: form.value.username.trim() || 'opencode',
      password: form.value.password || null,
      default_provider: form.value.provider || null,
      default_model: form.value.model || null
    }

    await tauriApi.saveOpenCodeConfig(config)
    isConfigured.value = true
    ElMessage.success('配置已保存')
    visible.value = false
  } catch (error) {
    ElMessage.error('保存配置失败: ' + error)
  } finally {
    saving.value = false
  }
}
</script>

<style scoped>
.hint {
  font-size: 12px;
  color: #909399;
  margin-top: 5px;
  line-height: 1.5;
}

.hint code {
  background: #f5f7fa;
  padding: 2px 6px;
  border-radius: 3px;
  font-family: 'Courier New', monospace;
  color: #e74c3c;
}

.el-divider {
  margin: 24px 0 16px;
}

:deep(.el-divider__text) {
  font-size: 14px;
  font-weight: 500;
  display: flex;
  align-items: center;
  gap: 6px;
}
</style>
