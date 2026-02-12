import { createRouter, createWebHistory } from 'vue-router'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    { path: '/', name: 'dashboard', component: () => import('../views/DashboardView.vue') },
    { path: '/k8s', name: 'k8s-browser', component: () => import('../views/K8sBrowserView.vue') },
    { path: '/forwards', name: 'forwards', component: () => import('../views/ForwardsView.vue') },
    { path: '/settings', name: 'settings', component: () => import('../views/SettingsView.vue') },
  ],
})

export default router
