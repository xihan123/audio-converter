<script lang="ts" setup>
// AudioConverter 组件 - 用于显示单个文件的转换进度
interface Props {
  fileName: string
  progress: number
  status: 'pending' | 'processing' | 'completed' | 'error'
}

defineProps<Props>()
</script>

<template>
  <div class="converter-item">
    <div class="file-info">
      <span class="file-name">{{ fileName }}</span>
      <span :class="status" class="status-badge">
        {{ status === 'pending' ? '等待' : status === 'processing' ? '处理中' : status === 'completed' ? '完成' : '错误' }}
      </span>
    </div>
    <div class="progress-container">
      <div :style="{ width: progress + '%' }" class="progress-bar"></div>
    </div>
    <span class="progress-text">{{ Math.round(progress) }}%</span>
  </div>
</template>

<style scoped>
.converter-item {
  padding: 1rem;
  background: white;
  border-radius: 8px;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

.file-info {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 0.5rem;
}

.file-name {
  font-weight: 500;
  color: #2c3e50;
  flex: 1;
  margin-right: 1rem;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.status-badge {
  padding: 0.25rem 0.5rem;
  border-radius: 4px;
  font-size: 0.75rem;
  font-weight: 600;
}

.status-badge.pending {
  background: #e2e8f0;
  color: #4a5568;
}

.status-badge.processing {
  background: #bee3f8;
  color: #2c5282;
}

.status-badge.completed {
  background: #c6f6d5;
  color: #22543d;
}

.status-badge.error {
  background: #fed7d7;
  color: #742a2a;
}

.progress-container {
  height: 8px;
  background: #e2e8f0;
  border-radius: 4px;
  overflow: hidden;
  margin-bottom: 0.5rem;
}

.progress-bar {
  height: 100%;
  background: linear-gradient(90deg, #667eea, #764ba2);
  transition: width 0.3s ease;
}

.progress-text {
  font-size: 0.875rem;
  color: #718096;
}
</style>
