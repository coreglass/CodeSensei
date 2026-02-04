<template>
  <div class="graph-container">
    <div class="toolbar">
      <el-select v-model="selectedProject" placeholder="选择项目" style="width: 200px">
        <el-option
          v-for="project in projects"
          :key="project.id"
          :label="project.name"
          :value="project.id"
        />
      </el-select>
      <el-button type="primary" @click="loadGraph">加载图谱</el-button>
    </div>

    <div class="graph-content" ref="graphContainer">
      <div v-if="!graphData" class="placeholder">
        <el-empty description="选择项目后点击加载图谱" />
      </div>
      <!-- TODO: 集成 D3.js 或 ECharts -->
      <div v-else class="graph-placeholder">
        <p>依赖关系图谱将在此显示</p>
        <pre>{{ graphData }}</pre>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref } from 'vue'

const selectedProject = ref('')
const projects = ref([
  { id: '1', name: '示例项目' }
])
const graphData = ref(null)
const graphContainer = ref(null)

function loadGraph() {
  // TODO: 调用 Tauri command 获取依赖关系
  graphData.value = {
    nodes: [
      { id: 'main.py', label: 'main.py' },
      { id: 'utils.py', label: 'utils.py' }
    ],
    edges: [
      { from: 'main.py', to: 'utils.py', label: 'imports' }
    ]
  }
}
</script>

<style scoped>
.graph-container {
  height: 100%;
  display: flex;
  flex-direction: column;
  padding: 20px;
}

.toolbar {
  display: flex;
  gap: 10px;
  margin-bottom: 20px;
}

.graph-content {
  flex: 1;
  border: 1px solid #e4e7ed;
  border-radius: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.placeholder {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
}

.graph-placeholder {
  text-align: center;
}

.graph-placeholder pre {
  margin-top: 20px;
  text-align: left;
  background-color: #f5f7fa;
  padding: 20px;
  border-radius: 4px;
}
</style>
