import { createRouter, createWebHashHistory } from 'vue-router'
import Home from '../views/Home.vue'
import Project from '../views/Project.vue'
import Graph from '../views/Graph.vue'

const routes = [
  {
    path: '/',
    name: 'home',
    component: Home
  },
  {
    path: '/project/:id',
    name: 'project',
    component: Project
  },
  {
    path: '/graph',
    name: 'graph',
    component: Graph
  }
]

const router = createRouter({
  history: createWebHashHistory(),
  routes
})

export default router
