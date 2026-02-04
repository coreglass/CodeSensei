<template>
  <div class="home-container">
    <div class="toolbar">
      <el-button type="primary" :icon="Plus" @click="showCreateDialog = true">
        新建项目
      </el-button>
    </div>

    <el-table :data="projects" style="width: 100%" v-loading="loading">
      <el-table-column prop="name" label="项目名称" width="200" />
      <el-table-column prop="description" label="描述" />
      <el-table-column prop="language" label="语言" width="120" />
      <el-table-column prop="created_at" label="创建时间" width="180">
        <template #default="{ row }">
          {{ formatDate(row.created_at) }}
        </template>
      </el-table-column>
      <el-table-column label="操作" width="200">
        <template #default="{ row }">
          <el-button size="small" @click="openProject(row.id)">
            打开项目
          </el-button>
          <el-button size="small" type="danger" @click="deleteProject(row.id)">
            删除
          </el-button>
        </template>
      </el-table-column>
    </el-table>

    <!-- 新建项目弹窗 -->
    <el-dialog v-model="showCreateDialog" title="新建项目" width="500px">
      <el-form :model="newProject" label-width="100px">
        <el-form-item label="项目名称">
          <el-input v-model="newProject.name" placeholder="输入项目名称" />
        </el-form-item>
        <el-form-item label="描述">
          <el-input
            v-model="newProject.description"
            type="textarea"
            placeholder="简单描述这个项目"
            :rows="3"
          />
        </el-form-item>
        <el-form-item label="项目根目录">
          <el-input v-model="newProject.rootPath" placeholder="留空则使用默认目录" readonly>
            <template #append>
              <el-button @click="selectDirectory">选择目录</el-button>
            </template>
          </el-input>
        </el-form-item>
        <el-alert
          v-if="newProject.rootPath"
          title="已选择根目录，将扫描该目录下的所有文件"
          type="info"
          :closable="false"
          style="margin-top: 10px"
        />
      </el-form>
      <template #footer>
        <el-button @click="showCreateDialog = false">取消</el-button>
        <el-button type="primary" @click="createProject" :loading="loading">创建</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { Plus } from '@element-plus/icons-vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import * as tauriApi from '../api/tauri'

const router = useRouter()
const projects = ref([])
const showCreateDialog = ref(false)
const loading = ref(false)

const newProject = ref({
  name: '',
  description: '',
  rootPath: ''
})

// 加载项目列表
onMounted(async () => {
  await loadProjects()
})

async function loadProjects() {
  try {
    loading.value = true
    projects.value = await tauriApi.scanProjects()
  } catch (error) {
    console.error('加载项目失败:', error)
    ElMessage.error('加载项目列表失败: ' + error)
  } finally {
    loading.value = false
  }
}

function formatDate(timestamp) {
  return new Date(timestamp).toLocaleString('zh-CN')
}

async function createProject() {
  if (!newProject.value.name) {
    ElMessage.warning('请输入项目名称')
    return
  }

  try {
    loading.value = true
    const project = await tauriApi.createProject({
      name: newProject.value.name,
      description: newProject.value.description,
      rootPath: newProject.value.rootPath || null
    })

    showCreateDialog.value = false
    newProject.value = { name: '', description: '', rootPath: '' }

    ElMessage.success('项目创建成功')

    // 直接跳转到项目界面
    router.push(`/project/${project.id}`)
  } catch (error) {
    ElMessage.error('创建项目失败: ' + error)
  } finally {
    loading.value = false
  }
}

async function selectDirectory() {
  try {
    const selectedPath = await tauriApi.selectDirectory()
    if (selectedPath) {
      newProject.value.rootPath = selectedPath
      ElMessage.success('已选择目录: ' + selectedPath)
    }
  } catch (error) {
    ElMessage.error('选择目录失败: ' + error)
  }
}

function openProject(id) {
  router.push(`/project/${id}`)
}

async function deleteProject(id) {
  try {
    await ElMessageBox.confirm('确定要删除这个项目吗？', '确认删除', {
      type: 'warning'
    })

    await tauriApi.deleteProject(id)
    projects.value = projects.value.filter((p) => p.id !== id)
    ElMessage.success('项目已删除')
  } catch (error) {
    if (error !== 'cancel') {
      ElMessage.error('删除项目失败: ' + error)
    }
  }
}
</script>

<style scoped>
.home-container {
  height: 100%;
  display: flex;
  flex-direction: column;
  padding: 20px;
}

.toolbar {
  margin-bottom: 20px;
}
</style>
