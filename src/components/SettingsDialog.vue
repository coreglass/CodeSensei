<template>
  <el-dialog
    v-model="visible"
    title="设置"
    width="500px"
    :close-on-click-modal="false"
  >
    <el-form :model="form" label-width="120px">
      <el-form-item label="Claude API Key">
        <el-input
          v-model="form.apiKey"
          placeholder="sk-ant-api03-..."
          type="password"
          show-password
        />
        <div class="hint">
          获取 API Key: <a href="https://console.anthropic.com/" target="_blank">https://console.anthropic.com/</a>
        </div>
      </el-form-item>

      <el-form-item label="API Key 状态">
        <el-tag :type="hasApiKey ? 'success' : 'warning'">
          {{ hasApiKey ? '已配置' : '未配置' }}
        </el-tag>
      </el-form-item>

      <el-form-item label="模型">
        <el-input value="Claude Sonnet 4.5" disabled />
        <div class="hint">当前使用最新的 Claude Sonnet 4.5 模型</div>
      </el-form-item>
    </el-form>

    <template #footer>
      <el-button @click="visible = false">取消</el-button>
      <el-button type="primary" @click="saveSettings" :loading="saving">
        保存
      </el-button>
    </template>
  </el-dialog>
</template>

<script setup>
import { ref, computed, watch } from 'vue'
import { ElMessage } from 'element-plus'
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
  apiKey: ''
})

const saving = ref(false)
const hasApiKey = ref(false)

// 监听对话框打开，检查 API Key 状态
watch(() => props.modelValue, async (val) => {
  if (val) {
    await checkApiKeyStatus()
  }
})

async function checkApiKeyStatus() {
  try {
    hasApiKey.value = await tauriApi.checkApiKey()
  } catch (error) {
    console.error('检查 API Key 状态失败:', error)
    hasApiKey.value = false
  }
}

async function saveSettings() {
  if (!form.value.apiKey.trim()) {
    ElMessage.warning('请输入 API Key')
    return
  }

  // 简单验证 API Key 格式
  if (!form.value.apiKey.startsWith('sk-ant-')) {
    ElMessage.warning('API Key 格式不正确，应该以 sk-ant- 开头')
    return
  }

  try {
    saving.value = true
    await tauriApi.saveClaudeApiKey(form.value.apiKey)
    hasApiKey.value = true
    ElMessage.success('API Key 已保存')
    visible.value = false
    form.value.apiKey = ''
  } catch (error) {
    ElMessage.error('保存 API Key 失败: ' + error)
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
}

.hint a {
  color: #409eff;
  text-decoration: none;
}

.hint a:hover {
  text-decoration: underline;
}
</style>
