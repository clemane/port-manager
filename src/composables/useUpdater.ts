import { ref, readonly } from 'vue'
import { check } from '@tauri-apps/plugin-updater'
import { relaunch } from '@tauri-apps/plugin-process'

const updateAvailable = ref(false)
const newVersion = ref('')
const releaseNotes = ref('')
const downloading = ref(false)
const progress = ref(0)
const error = ref<string | null>(null)

let pendingUpdate: Awaited<ReturnType<typeof check>> | null = null

async function checkForUpdate() {
  error.value = null
  try {
    const update = await check()
    if (update) {
      pendingUpdate = update
      updateAvailable.value = true
      newVersion.value = update.version
      releaseNotes.value = update.body ?? ''
    }
  } catch (e) {
    error.value = `Update check failed: ${e}`
  }
}

async function downloadAndInstall() {
  if (!pendingUpdate) return
  downloading.value = true
  progress.value = 0
  error.value = null
  try {
    let totalBytes = 0
    let downloadedBytes = 0
    await pendingUpdate.downloadAndInstall((event) => {
      if (event.event === 'Started') {
        totalBytes = event.data.contentLength ?? 0
      } else if (event.event === 'Progress') {
        downloadedBytes += event.data.chunkLength
        if (totalBytes > 0) {
          progress.value = Math.round((downloadedBytes / totalBytes) * 100)
        }
      } else if (event.event === 'Finished') {
        progress.value = 100
      }
    })
    await relaunch()
  } catch (e) {
    error.value = `Update failed: ${e}`
    downloading.value = false
  }
}

function dismiss() {
  updateAvailable.value = false
  pendingUpdate = null
}

export function useUpdater() {
  return {
    updateAvailable: readonly(updateAvailable),
    newVersion: readonly(newVersion),
    releaseNotes: readonly(releaseNotes),
    downloading: readonly(downloading),
    progress: readonly(progress),
    error: readonly(error),
    checkForUpdate,
    downloadAndInstall,
    dismiss,
  }
}
