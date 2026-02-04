<template>
  <el-dialog
    v-model="visible"
    title="输入需求"
    width="600px"
    :close-on-click-modal="false"
  >
    <el-form :model="form" label-width="100px">
      <el-form-item label="需求描述">
        <el-input
          v-model="form.requirement"
          type="textarea"
          placeholder="描述你想要实现的功能..."
          :rows="6"
        />
      </el-form-item>

      <el-form-item label="编程语言">
        <el-select v-model="form.language" placeholder="选择语言">
          <el-option label="Python" value="python" />
          <el-option label="JavaScript" value="javascript" />
          <el-option label="TypeScript" value="typescript" />
          <el-option label="Java" value="java" />
          <el-option label="Go" value="go" />
          <el-option label="Rust" value="rust" />
        </el-select>
      </el-form-item>

      <el-form-item label="API 设置">
        <el-input
          v-model="form.apiKey"
          placeholder="输入 API Key"
          type="password"
          show-password
        />
        <el-input
          v-model="form.apiBase"
          placeholder="API Base URL (如 https://open.bigmodel.cn/api/paas/v4)"
          style="margin-top: 10px"
        />
      </el-form-item>

      <el-form-item label="模型">
        <el-select v-model="form.model" placeholder="选择模型">
          <el-option label="智谱 GLM-4" value="glm-4" />
          <el-option label="智谱 GLM-4-Flash" value="glm-4-flash" />
          <el-option label="通义千问" value="qwen-plus" />
        </el-select>
      </el-form-item>
    </el-form>

    <div v-if="generating" class="generating-status">
      <el-progress :percentage="progress" :status="progressStatus" />
      <p>{{ statusText }}</p>
    </div>

    <template #footer>
      <el-button @click="visible = false" :disabled="generating">取消</el-button>
      <el-button type="primary" @click="generate" :loading="generating">
        {{ generating ? '生成中...' : '开始生成' }}
      </el-button>
    </template>
  </el-dialog>
</template>

<script setup>
import { ref, watch, computed } from 'vue'
import { ElMessage } from 'element-plus'

const props = defineProps({
  modelValue: {
    type: Boolean,
    required: true
  },
  projectId: {
    type: String,
    required: true
  }
})

const emit = defineEmits(['update:modelValue', 'generated'])

const visible = computed({
  get: () => props.modelValue,
  set: (val) => emit('update:modelValue', val)
})

const form = ref({
  requirement: '',
  language: 'python',
  apiKey: '',
  apiBase: 'https://open.bigmodel.cn/api/paas/v4',
  model: 'glm-4-flash'
})

const generating = ref(false)
const progress = ref(0)
const statusText = ref('')

async function generate() {
  if (!form.value.requirement) {
    ElMessage.warning('请输入需求描述')
    return
  }

  if (!form.value.apiKey) {
    ElMessage.warning('请输入 API Key')
    return
  }

  generating.value = true
  progress.value = 10
  statusText.value = '正在分析需求...'

  try {
    // TODO: 调用 Tauri command 生成代码
    // const result = await invoke('generate_code', {
    //   projectId: props.projectId,
    //   ...form.value
    // })

    // 模拟生成过程
    await simulateGeneration()

    ElMessage.success('代码生成成功')
    emit('generated')
    visible.value = false
  } catch (error) {
    ElMessage.error('生成失败: ' + error.message)
  } finally {
    generating.value = false
    progress.value = 0
  }
}

async function simulateGeneration() {
  const steps = [
    { progress: 20, text: '正在分析需求...' },
    { progress: 40, text: '正在规划文件结构...' },
    { progress: 60, text: '正在生成代码...' },
    { progress: 80, text: '正在生成文档...' },
    { progress: 100, text: '完成！' }
  ]

  for (const step of steps) {
    await new Promise((resolve) => setTimeout(resolve, 800))
    progress.value = step.progress
    statusText.value = step.text
  }
}

watch(
  () => props.modelValue,
  (val) => {
    if (val) {
      // 对话框打开时重置表单
      progress.value = 0
      statusText.value = ''
    }
  }
)
</script>

<style scoped>
.generating-status {
  margin-top: 20px;
  padding: 15px;
  background-color: #f5f7fa;
  border-radius: 4px;
}

.generating-status p {
  margin-top: 10px;
  text-align: center;
  color: #606266;
}
</style>
