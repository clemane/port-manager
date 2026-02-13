import { createRouter, createWebHistory } from 'vue-router'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    { path: '/', name: 'dashboard', component: () => import('../views/DashboardView.vue'), meta: { title: 'Port Scanner' } },
    { path: '/k8s', name: 'k8s-browser', component: () => import('../views/K8sBrowserView.vue'), meta: { title: 'Kubernetes' } },
    { path: '/forwards', name: 'forwards', component: () => import('../views/ForwardsView.vue'), meta: { title: 'Port Forwards' } },
    { path: '/ngrok', name: 'ngrok', component: () => import('../views/NgrokView.vue'), meta: { title: 'Ngrok Tunnels' } },
    { path: '/database', name: 'database', component: () => import('../views/DatabaseView.vue'), meta: { title: 'Database' } },
    { path: '/settings', name: 'settings', component: () => import('../views/SettingsView.vue'), meta: { title: 'Settings' } },
  ],
})

export default router
