<script lang="ts" setup>
import {computed, reactive, ref} from 'vue'
import {processFile} from './utils/audioProcessor'
import {downloadFilesAsZip, generateZipFileName} from './utils/batchDownload'

interface FileProgress {
  id: string
  name: string
  status: 'pending' | 'processing' | 'completed' | 'error'
  progress: number
  message: string
  downloadUrl?: string
  originalSize?: number
  convertedSize?: number
  duration?: number
}

const files = reactive<Map<string, FileProgress>>(new Map())
const isDragging = ref(false)

const handleDragOver = (e: DragEvent) => {
  e.preventDefault()
  isDragging.value = true
}

const handleDragLeave = () => {
  isDragging.value = false
}

const handleDrop = async (e: DragEvent) => {
  e.preventDefault()
  isDragging.value = false

  const items = e.dataTransfer?.items
  if (!items) return

  const allFiles: File[] = []

  for (let i = 0; i < items.length; i++) {
    const item = items[i]
    if (item.kind === 'file') {
      const entry = item.webkitGetAsEntry()
      if (entry) {
        await traverseFileTree(entry, allFiles)
      }
    }
  }

  processFiles(allFiles)
}

const traverseFileTree = async (entry: any, fileList: File[]): Promise<void> => {
  if (entry.isFile) {
    const file = await new Promise<File>((resolve) => {
      entry.file((file: File) => resolve(file))
    })

    // 只处理音频文件
    if (file.type.startsWith('audio/')) {
      fileList.push(file)
    }
  } else if (entry.isDirectory) {
    const reader = entry.createReader()
    const entries = await new Promise<any[]>((resolve) => {
      reader.readEntries((entries: any[]) => resolve(entries))
    })

    for (const childEntry of entries) {
      await traverseFileTree(childEntry, fileList)
    }
  }
}

const handleFileInput = (e: Event) => {
  const target = e.target as HTMLInputElement
  if (target.files) {
    processFiles(Array.from(target.files))
  }
}

const processFiles = async (fileList: File[]) => {
  for (const file of fileList) {
    const fileId = `${file.name}-${Date.now()}-${Math.random()}`

    files.set(fileId, {
      id: fileId,
      name: file.name,
      status: 'pending',
      progress: 0,
      message: '等待处理...',
      originalSize: file.size
    })

    // 异步处理每个文件
    processFile(file, (progress) => {
      const fileProgress = files.get(fileId)
      if (fileProgress) {
        fileProgress.progress = progress
        fileProgress.status = 'processing'
        fileProgress.message = `处理中... ${Math.round(progress)}%`
      }
    })
        .then((result) => {
          const fileProgress = files.get(fileId)
          if (fileProgress) {
            fileProgress.status = 'completed'
            fileProgress.progress = 100
            fileProgress.message = '转换完成'
            fileProgress.downloadUrl = result.url
            fileProgress.convertedSize = result.size
            fileProgress.duration = result.duration
          }
        })
        .catch((error) => {
          const fileProgress = files.get(fileId)
          if (fileProgress) {
            fileProgress.status = 'error'
            fileProgress.message = `错误: ${error.message}`
          }
        })
  }
}

const downloadFile = (fileProgress: FileProgress) => {
  if (fileProgress.downloadUrl) {
    const a = document.createElement('a')
    a.href = fileProgress.downloadUrl
    a.download = fileProgress.name.replace(/\.[^/.]+$/, '') + '.wav'
    document.body.appendChild(a)
    a.click()
    document.body.removeChild(a)
  }
}

const removeFile = (fileId: string) => {
  const fileProgress = files.get(fileId)
  if (fileProgress?.downloadUrl) {
    URL.revokeObjectURL(fileProgress.downloadUrl)
  }
  files.delete(fileId)
}

const clearCompleted = () => {
  const toRemove: string[] = []
  files.forEach((file, id) => {
    if (file.status === 'completed' || file.status === 'error') {
      if (file.downloadUrl) {
        URL.revokeObjectURL(file.downloadUrl)
      }
      toRemove.push(id)
    }
  })
  toRemove.forEach(id => files.delete(id))
}

// 批量下载相关
const isBatchDownloading = ref(false)
const batchDownloadProgress = ref(0)

const completedFiles = computed(() => {
  const completed: FileProgress[] = []
  files.forEach(file => {
    if (file.status === 'completed' && file.downloadUrl) {
      completed.push(file)
    }
  })
  return completed
})

const hasCompletedFiles = computed(() => completedFiles.value.length > 0)

const batchDownload = async () => {
  if (completedFiles.value.length === 0) return

  isBatchDownloading.value = true
  batchDownloadProgress.value = 0

  try {
    const filesToDownload = completedFiles.value.map(file => ({
      name: file.name.replace(/\.[^/.]+$/, '') + '.wav',
      url: file.downloadUrl!
    }))

    await downloadFilesAsZip(
        filesToDownload,
        generateZipFileName('audio-to-wav'),
        (progress) => {
          batchDownloadProgress.value = progress
        }
    )
  } catch (error: any) {
    alert(`批量下载失败: ${error.message}`)
  } finally {
    isBatchDownloading.value = false
    batchDownloadProgress.value = 0
  }
}

const formatFileSize = (bytes?: number): string => {
  if (!bytes) return '-'
  if (bytes < 1024) return bytes + ' B'
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(2) + ' KB'
  return (bytes / (1024 * 1024)).toFixed(2) + ' MB'
}

const formatDuration = (seconds?: number): string => {
  if (!seconds) return '-'
  const mins = Math.floor(seconds / 60)
  const secs = Math.floor(seconds % 60)
  return `${mins}:${secs.toString().padStart(2, '0')}`
}
</script>

<template>
  <div class="app">
    <header class="header">
      <h1>🎵 音频转 WAV 格式工具</h1>
      <p>支持拖入文件/文件夹，自动将音频转为 WAV 格式，时长不足 30 秒自动重复拼接</p>
    </header>

    <div
        :class="{ 'drag-over': isDragging }"
        class="drop-zone"
        @dragleave="handleDragLeave"
        @dragover="handleDragOver"
        @drop="handleDrop"
    >
      <div class="drop-zone-content">
        <svg class="upload-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12" stroke-linecap="round" stroke-linejoin="round"
                stroke-width="2"/>
        </svg>
        <p class="drop-zone-text">拖入音频文件或文件夹</p>
        <p class="drop-zone-subtext">或者</p>
        <label class="file-input-label">
          <input
              accept="audio/*"
              class="file-input"
              multiple
              type="file"
              @change="handleFileInput"
          >
          <span class="file-input-button">选择文件</span>
        </label>
      </div>
    </div>

    <div v-if="files.size > 0" class="files-section">
      <div class="files-header">
        <h2>文件列表 ({{ files.size }})</h2>
        <div class="header-actions">
          <button
              v-if="hasCompletedFiles"
              :disabled="isBatchDownloading"
              class="btn-batch-download"
              @click="batchDownload"
          >
            <span v-if="!isBatchDownloading">📦 批量下载 ({{ completedFiles.length }})</span>
            <span v-else>打包中... {{ Math.round(batchDownloadProgress) }}%</span>
          </button>
          <button class="btn-clear" @click="clearCompleted">清除已完成</button>
        </div>
      </div>

      <div class="files-list">
        <div
            v-for="[id, file] in files"
            :key="id"
            :class="`status-${file.status}`"
            class="file-item"
        >
          <div class="file-header">
            <div class="file-info">
              <span class="file-icon">
                <span v-if="file.status === 'completed'">✓</span>
                <span v-else-if="file.status === 'error'">✕</span>
                <span v-else-if="file.status === 'processing'">⟳</span>
                <span v-else>◷</span>
              </span>
              <div class="file-details">
                <div class="file-name">{{ file.name }}</div>
                <div class="file-meta">
                  <span>原始大小: {{ formatFileSize(file.originalSize) }}</span>
                  <span v-if="file.convertedSize"> → {{ formatFileSize(file.convertedSize) }}</span>
                  <span v-if="file.duration" class="file-duration">
                    时长: {{ formatDuration(file.duration) }}
                  </span>
                </div>
              </div>
            </div>
            <div class="file-actions">
              <button
                  v-if="file.status === 'completed'"
                  class="btn-download"
                  @click="downloadFile(file)"
              >
                下载
              </button>
              <button class="btn-remove" @click="removeFile(id)">删除</button>
            </div>
          </div>

          <div class="progress-section">
            <div class="progress-bar-container">
              <div
                  :style="{ width: file.progress + '%' }"
                  class="progress-bar"
              ></div>
            </div>
            <span class="progress-text">{{ file.message }}</span>
          </div>
        </div>
      </div>
    </div>

    <!-- 底部信息 -->
    <footer class="footer">
      <div class="footer-content">
        <div class="footer-section">
          <span class="footer-label">💻 推荐桌面端应用：</span>
          <a 
            href="https://github.com/xihan123/audio-converter" 
            target="_blank" 
            rel="noopener noreferrer"
            class="footer-link desktop-link"
          >
            <svg viewBox="0 0 16 16" width="16" height="16" fill="currentColor">
              <path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.013 8.013 0 0016 8c0-4.42-3.58-8-8-8z"/>
            </svg>
            <span>桌面端（高性能）22.9MB</span>
          </a>
        </div>
        <div class="footer-divider">|</div>
        <div class="footer-section">
          <span class="footer-label">🌐 Web版源码：</span>
          <a 
            href="https://github.com/xihan123/audio-converter/tree/js" 
            target="_blank" 
            rel="noopener noreferrer"
            class="footer-link web-link"
          >
            <svg viewBox="0 0 16 16" width="16" height="16" fill="currentColor">
              <path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.013 8.013 0 0016 8c0-4.42-3.58-8-8-8z"/>
            </svg>
            <span>GitHub (JS分支)</span>
          </a>
        </div>
      </div>
    </footer>
  </div>
</template>

<style scoped>
.app {
  max-width: 1200px;
  margin: 0 auto;
  padding: 2rem;
}

.header {
  text-align: center;
  margin-bottom: 2rem;
}

.header h1 {
  font-size: 2rem;
  margin-bottom: 0.5rem;
  color: #2c3e50;
}

.header p {
  color: #ffffffff;
  font-size: 0.95rem;
}

.drop-zone {
  border: 3px dashed #cbd5e0;
  border-radius: 12px;
  padding: 3rem;
  text-align: center;
  transition: all 0.3s;
  background: #f7fafc;
  cursor: pointer;
}

.drop-zone.drag-over {
  border-color: #4299e1;
  background: #ebf8ff;
  transform: scale(1.02);
}

.drop-zone-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 1rem;
}

.upload-icon {
  width: 64px;
  height: 64px;
  color: #a0aec0;
}

.drop-zone-text {
  font-size: 1.25rem;
  color: #2d3748;
  font-weight: 500;
}

.drop-zone-subtext {
  color: #718096;
  font-size: 0.9rem;
}

.file-input {
  display: none;
}

.file-input-label {
  cursor: pointer;
}

.file-input-button {
  display: inline-block;
  padding: 0.75rem 2rem;
  background: #4299e1;
  color: white;
  border-radius: 8px;
  font-weight: 500;
  transition: background 0.2s;
}

.file-input-button:hover {
  background: #3182ce;
}

.files-section {
  margin-top: 2rem;
}

.files-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 1rem;
}

.files-header h2 {
  font-size: 1.5rem;
  color: #2c3e50;
}

.header-actions {
  display: flex;
  gap: 0.5rem;
}

.btn-batch-download {
  padding: 0.5rem 1rem;
  background: #4299e1;
  color: white;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-size: 0.9rem;
  font-weight: 500;
  transition: all 0.2s;
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.btn-batch-download:hover:not(:disabled) {
  background: #3182ce;
  transform: translateY(-1px);
  box-shadow: 0 4px 8px rgba(66, 153, 225, 0.3);
}

.btn-batch-download:disabled {
  background: #a0aec0;
  cursor: not-allowed;
  opacity: 0.6;
}

.btn-clear {
  padding: 0.5rem 1rem;
  background: #e2e8f0;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-size: 0.9rem;
  transition: background 0.2s;
}

.btn-clear:hover {
  background: #cbd5e0;
}

.files-list {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.file-item {
  background: white;
  border: 1px solid #e2e8f0;
  border-radius: 8px;
  padding: 1rem;
  transition: box-shadow 0.2s;
}

.file-item:hover {
  box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
}

.file-item.status-completed {
  border-left: 4px solid #48bb78;
}

.file-item.status-error {
  border-left: 4px solid #f56565;
}

.file-item.status-processing {
  border-left: 4px solid #4299e1;
}

.file-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 0.75rem;
}

.file-info {
  display: flex;
  gap: 0.75rem;
  flex: 1;
}

.file-icon {
  font-size: 1.5rem;
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
  background: #f7fafc;
}

.status-completed .file-icon {
  background: #c6f6d5;
  color: #22543d;
}

.status-error .file-icon {
  background: #fed7d7;
  color: #742a2a;
}

.status-processing .file-icon {
  background: #bee3f8;
  color: #2c5282;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}

.file-details {
  flex: 1;
}

.file-name {
  font-weight: 500;
  color: #2d3748;
  margin-bottom: 0.25rem;
  word-break: break-word;
}

.file-meta {
  font-size: 0.85rem;
  color: #718096;
  display: flex;
  gap: 1rem;
  flex-wrap: wrap;
}

.file-duration {
  color: #4299e1;
  font-weight: 500;
}

.file-actions {
  display: flex;
  gap: 0.5rem;
}

.btn-download,
.btn-remove {
  padding: 0.4rem 0.8rem;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-size: 0.85rem;
  transition: all 0.2s;
}

.btn-download {
  background: #48bb78;
  color: white;
}

.btn-download:hover {
  background: #38a169;
}

.btn-remove {
  background: #e2e8f0;
  color: #2d3748;
}

.btn-remove:hover {
  background: #cbd5e0;
}

.progress-section {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.progress-bar-container {
  width: 100%;
  height: 8px;
  background: #e2e8f0;
  border-radius: 4px;
  overflow: hidden;
}

.progress-bar {
  height: 100%;
  background: linear-gradient(90deg, #4299e1, #3182ce);
  transition: width 0.3s;
  border-radius: 4px;
}

.status-completed .progress-bar {
  background: linear-gradient(90deg, #48bb78, #38a169);
}

.status-error .progress-bar {
  background: linear-gradient(90deg, #f56565, #e53e3e);
}

.progress-text {
  font-size: 0.85rem;
  color: #718096;
}

/* 底部样式 */
.footer {
  margin-top: 3rem;
  padding: 2rem 0 1rem 0;
  border-top: 1px solid rgba(255, 255, 255, 0.2);
}

.footer-content {
  display: flex;
  justify-content: center;
  align-items: center;
  gap: 1.5rem;
  flex-wrap: wrap;
}

.footer-section {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.footer-label {
  color: rgba(255, 255, 255, 0.9);
  font-size: 0.9rem;
  font-weight: 500;
}

.footer-divider {
  color: rgba(255, 255, 255, 0.4);
  font-size: 1.2rem;
}

.footer-link {
  display: inline-flex;
  align-items: center;
  gap: 0.4rem;
  padding: 0.4rem 0.8rem;
  background: rgba(255, 255, 255, 0.1);
  color: rgba(255, 255, 255, 0.95);
  text-decoration: none;
  border-radius: 6px;
  font-size: 0.85rem;
  font-weight: 500;
  transition: all 0.3s;
  border: 1px solid rgba(255, 255, 255, 0.2);
}

.footer-link:hover {
  background: rgba(255, 255, 255, 0.2);
  border-color: rgba(255, 255, 255, 0.4);
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
}

.footer-link svg {
  transition: transform 0.3s;
  flex-shrink: 0;
}

.footer-link:hover svg {
  transform: rotate(360deg);
}

.desktop-link:hover {
  background: rgba(72, 187, 120, 0.2);
  border-color: rgba(72, 187, 120, 0.4);
  color: #48bb78;
}

.web-link:hover {
  background: rgba(66, 153, 225, 0.2);
  border-color: rgba(66, 153, 225, 0.4);
  color: #4299e1;
}

/* 响应式设计 */
@media (max-width: 768px) {
  .footer-content {
    flex-direction: column;
    gap: 1rem;
  }
  
  .footer-divider {
    display: none;
  }
  
  .footer-section {
    flex-direction: column;
    text-align: center;
    gap: 0.5rem;
  }
}
</style>
