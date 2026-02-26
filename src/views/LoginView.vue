<script setup lang="ts">
import { ref, computed } from 'vue'
import { PmButton, PmInput } from '@/components/ui'
import { useAuth } from '@/composables/useAuth'

const { vaultExists, loading, error, recoveryKey, createPassword, login } = useAuth()

const password = ref('')
const confirmPassword = ref('')
const showRecoveryKey = ref(false)
const copiedKey = ref(false)

const passwordMismatch = computed(() => {
  return confirmPassword.value.length > 0 && password.value !== confirmPassword.value
})

const canCreate = computed(() => {
  return password.value.length >= 8 && password.value === confirmPassword.value && !loading.value
})

const canLogin = computed(() => {
  return password.value.length > 0 && !loading.value
})

async function handleCreate() {
  if (!canCreate.value) return
  const key = await createPassword(password.value)
  if (key) {
    showRecoveryKey.value = true
  }
}

async function handleLogin() {
  if (!canLogin.value) return
  await login(password.value)
  password.value = ''
}

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Enter') {
    if (showRecoveryKey.value) return
    if (!vaultExists.value) handleCreate()
    else handleLogin()
  }
}

async function copyRecoveryKey() {
  if (recoveryKey.value) {
    await navigator.clipboard.writeText(recoveryKey.value)
    copiedKey.value = true
    setTimeout(() => { copiedKey.value = false }, 2000)
  }
}

function dismissRecoveryKey() {
  showRecoveryKey.value = false
}
</script>

<template>
  <div class="login" @keydown="handleKeydown">
    <div class="login__card">
      <div class="login__header">
        <h1 class="login__title">Port Manager</h1>
        <p class="login__subtitle">Vault</p>
      </div>

      <!-- Recovery Key Display -->
      <div v-if="showRecoveryKey" class="login__form">
        <div class="recovery">
          <p class="recovery__label">Recovery Key</p>
          <p class="recovery__desc">
            Save this key in a safe place. You will need it to recover your vault if you forget your password.
          </p>
          <div class="recovery__block">
            <code class="recovery__key">{{ recoveryKey }}</code>
            <button class="recovery__copy" @click="copyRecoveryKey" :title="copiedKey ? 'Copied!' : 'Copy'">
              <svg v-if="!copiedKey" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" width="14" height="14"><rect x="5" y="5" width="8" height="8" rx="1"/><path d="M3 11V3h8" stroke-linecap="round"/></svg>
              <svg v-else viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" width="14" height="14"><path d="M4 8l3 3 5-5" stroke-linecap="round" stroke-linejoin="round"/></svg>
            </button>
          </div>
        </div>
        <PmButton @click="dismissRecoveryKey" size="lg">I've saved my key</PmButton>
      </div>

      <!-- Setup Form (no vault exists) -->
      <div v-else-if="!vaultExists" class="login__form">
        <p class="login__desc">Create a master password to secure your vault.</p>
        <label class="form-label">
          Password
          <PmInput v-model="password" type="password" placeholder="Minimum 8 characters" />
        </label>
        <label class="form-label">
          Confirm Password
          <PmInput v-model="confirmPassword" type="password" placeholder="Re-enter password" />
        </label>
        <p v-if="passwordMismatch" class="form-error">Passwords do not match</p>
        <p v-if="error" class="form-error">{{ error }}</p>
        <PmButton :disabled="!canCreate" :loading="loading" size="lg" @click="handleCreate">Create Vault</PmButton>
      </div>

      <!-- Login Form (vault exists) -->
      <div v-else class="login__form">
        <p class="login__desc">Enter your master password to unlock the vault.</p>
        <label class="form-label">
          Password
          <PmInput v-model="password" type="password" placeholder="Master password" />
        </label>
        <p v-if="error" class="form-error">{{ error }}</p>
        <PmButton :disabled="!canLogin" :loading="loading" size="lg" @click="handleLogin">Unlock</PmButton>
      </div>
    </div>
  </div>
</template>

<style scoped>
.login {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 100vh;
  background: var(--pm-bg);
  background-image: var(--pm-gradient-ambiance);
}

.login__card {
  background: var(--pm-surface);
  border: 1px solid var(--pm-border);
  border-radius: var(--pm-radius-lg);
  box-shadow: var(--pm-shadow);
  padding: 40px;
  width: 100%;
  max-width: 400px;
}

.login__header {
  text-align: center;
  margin-bottom: 32px;
}

.login__title {
  font-family: var(--pm-font-display);
  font-size: 24px;
  font-weight: 700;
  color: var(--pm-text-primary);
  margin: 0 0 4px;
}

.login__subtitle {
  font-family: var(--pm-font-body);
  font-size: 14px;
  color: var(--pm-accent);
  margin: 0;
  letter-spacing: 0.05em;
  text-transform: uppercase;
}

.login__desc {
  font-family: var(--pm-font-body);
  font-size: 13px;
  color: var(--pm-text-muted);
  margin: 0 0 8px;
}

.login__form {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.form-label {
  font-family: var(--pm-font-body);
  font-size: 13px;
  color: var(--pm-text-secondary);
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.form-error {
  color: var(--pm-danger);
  font-size: 12px;
  margin: 0;
}

.recovery__label {
  font-family: var(--pm-font-display);
  font-size: 14px;
  font-weight: 600;
  color: var(--pm-text-primary);
  margin: 0 0 4px;
}

.recovery__desc {
  font-family: var(--pm-font-body);
  font-size: 12px;
  color: var(--pm-warning);
  margin: 0 0 12px;
}

.recovery__block {
  background: var(--pm-surface-elevated);
  border: 1px solid var(--pm-border);
  border-radius: var(--pm-radius-sm);
  padding: 12px 16px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.recovery__key {
  font-family: var(--pm-font-mono);
  font-size: 12px;
  color: var(--pm-text-primary);
  word-break: break-all;
  line-height: 1.5;
}

.recovery__copy {
  background: none;
  border: none;
  color: var(--pm-text-muted);
  cursor: pointer;
  padding: 4px;
  display: flex;
  align-items: center;
  flex-shrink: 0;
  transition: color 0.15s;
}

.recovery__copy:hover {
  color: var(--pm-text-primary);
}
</style>
