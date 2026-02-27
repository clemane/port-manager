import { ref, readonly } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const isUnlocked = ref(false)
const vaultExists = ref(false)
const loading = ref(false)
const error = ref<string | null>(null)
const recoveryKey = ref<string | null>(null)

let inactivityTimer: ReturnType<typeof setTimeout> | null = null
const INACTIVITY_TIMEOUT = 15 * 60 * 1000

function startInactivityTimer() {
  stopInactivityTimer()
  inactivityTimer = setTimeout(() => {
    lock()
  }, INACTIVITY_TIMEOUT)
}

function stopInactivityTimer() {
  if (inactivityTimer) {
    clearTimeout(inactivityTimer)
    inactivityTimer = null
  }
}

function resetActivity() {
  if (isUnlocked.value) {
    startInactivityTimer()
  }
}

async function checkStatus() {
  try {
    const status = await invoke<{ exists: boolean; unlocked: boolean }>('vault_status')
    vaultExists.value = status.exists
    isUnlocked.value = status.unlocked
  } catch (e) {
    error.value = `Failed to check vault status: ${e}`
  }
}

async function createPassword(password: string): Promise<string | null> {
  loading.value = true
  error.value = null
  try {
    const key = await invoke<string>('create_master_password', { password })
    recoveryKey.value = key
    vaultExists.value = true
    return key
  } catch (e) {
    error.value = `Failed to create vault: ${e}`
    return null
  } finally {
    loading.value = false
  }
}

function confirmVaultCreation() {
  isUnlocked.value = true
  startInactivityTimer()
}

async function login(password: string): Promise<boolean> {
  loading.value = true
  error.value = null
  try {
    const success = await invoke<boolean>('login', { password })
    if (success) {
      isUnlocked.value = true
      startInactivityTimer()
    } else {
      error.value = 'Invalid password'
    }
    return success
  } catch (e) {
    error.value = `Login failed: ${e}`
    return false
  } finally {
    loading.value = false
  }
}

async function recoverWithKey(recoveryKeyInput: string): Promise<boolean> {
  loading.value = true
  error.value = null
  try {
    const success = await invoke<boolean>('recover_vault', { recoveryKey: recoveryKeyInput })
    if (success) {
      isUnlocked.value = true
      startInactivityTimer()
    } else {
      error.value = 'Invalid recovery key'
    }
    return success
  } catch (e) {
    error.value = `Recovery failed: ${e}`
    return false
  } finally {
    loading.value = false
  }
}

async function lock() {
  try {
    await invoke('lock_vault')
  } catch {
    // Silently fail on lock
  }
  isUnlocked.value = false
  stopInactivityTimer()
}

export function useAuth() {
  return {
    isUnlocked: readonly(isUnlocked),
    vaultExists: readonly(vaultExists),
    loading: readonly(loading),
    error: readonly(error),
    recoveryKey: readonly(recoveryKey),
    checkStatus,
    createPassword,
    confirmVaultCreation,
    login,
    recoverWithKey,
    lock,
    resetActivity,
  }
}
