<template>
  <div v-if="show" class="modal-overlay" @click.self="onClose">
    <div class="update-card">
      <!-- Header -->
      <div class="update-header">
        <div class="header-icon-wrap">
          <span class="rocket-icon">🚀</span>
        </div>
        <div class="header-info">
          <div class="release-tag">BẢN CẬP NHẬT MỚI</div>
          <h2 class="update-title">QLCV v{{ update?.version }}</h2>
        </div>
        <button v-if="!downloading" class="btn-close" @click="onClose">✕</button>
      </div>

      <!-- Body / Changelog -->
      <div class="update-body">
        <div class="version-compare">
          <div class="ver-box current">
            <span class="ver-label">Phiên bản hiện tại</span>
            <span class="ver-num">v{{ currentAppVersion }}</span>
          </div>
          <div class="ver-arrow">➔</div>
          <div class="ver-box new">
            <span class="ver-label">Phiên bản mới nhất</span>
            <span class="ver-num">v{{ update?.version }}</span>
          </div>
        </div>

        <div v-if="update?.body" class="changelog-box">
          <div class="changelog-title">📋 Ghi chú bản phát hành:</div>
          <div class="changelog-content">{{ update.body }}</div>
        </div>
        <div v-else class="changelog-box changelog-empty">
          Phiên bản này bao gồm các cải tiến hiệu năng, tính năng mới và các bản sửa lỗi quan trọng.
        </div>

        <!-- Download Progress -->
        <div v-if="downloading" class="progress-section">
          <div class="progress-labels">
            <span class="progress-text">{{ statusText }}</span>
            <span class="progress-percent tabular-nums">{{ progressPercent }}%</span>
          </div>
          <div class="progress-bar-bg">
            <div class="progress-bar-fill" :style="{ width: progressPercent + '%' }"></div>
          </div>
        </div>

        <!-- Error Notice -->
        <div v-if="error" class="update-error">
          ⚠️ {{ error }}
        </div>
      </div>

      <!-- Actions -->
      <div class="update-footer">
        <button
          v-if="!downloading && !isFinished"
          type="button"
          class="btn-secondary"
          @click="onClose"
        >
          Để sau
        </button>

        <button
          v-if="!isFinished"
          type="button"
          class="btn-primary btn-update-action"
          :disabled="downloading"
          @click="startDownloadAndInstall"
        >
          <span v-if="downloading">⏳ Đang tải & cài đặt ({{ progressPercent }}%)...</span>
          <span v-else>⬇️ Tải xuống & Cập nhật ngay</span>
        </button>

        <button
          v-else
          type="button"
          class="btn-success btn-update-action"
          @click="relaunchApp"
        >
          🔄 Khởi động lại ngay
        </button>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref } from 'vue';
import { relaunch } from '@tauri-apps/plugin-process';

const props = defineProps({
  show: {
    type: Boolean,
    default: false,
  },
  update: {
    type: Object,
    default: null,
  },
  currentAppVersion: {
    type: String,
    default: '1.0.0',
  },
});

const emit = defineEmits(['close', 'updated']);

const downloading = ref(false);
const isFinished = ref(false);
const progressPercent = ref(0);
const statusText = ref('Đang kết nối tới máy chủ cập nhật...');
const error = ref('');

function onClose() {
  if (downloading.value) return;
  emit('close');
}

async function startDownloadAndInstall() {
  if (!props.update) return;

  downloading.value = true;
  error.value = '';
  progressPercent.value = 0;
  statusText.value = 'Đang tải gói cập nhật từ GitHub Releases...';

  try {
    let downloadedBytes = 0;
    let totalBytes = 0;

    await props.update.downloadAndInstall((event) => {
      switch (event.event) {
        case 'Started':
          totalBytes = event.data.contentLength || 0;
          statusText.value = 'Bắt đầu tải xuống...';
          break;
        case 'Progress':
          downloadedBytes += event.data.chunkLength;
          if (totalBytes > 0) {
            progressPercent.value = Math.min(100, Math.round((downloadedBytes / totalBytes) * 100));
            const mbDown = (downloadedBytes / (1024 * 1024)).toFixed(1);
            const mbTotal = (totalBytes / (1024 * 1024)).toFixed(1);
            statusText.value = `Đang tải: ${mbDown} MB / ${mbTotal} MB`;
          } else {
            const mbDown = (downloadedBytes / (1024 * 1024)).toFixed(1);
            statusText.value = `Đang tải: ${mbDown} MB`;
          }
          break;
        case 'Finished':
          progressPercent.value = 100;
          statusText.value = 'Đã tải xong! Đang cài đặt gói cập nhật...';
          break;
      }
    });

    isFinished.value = true;
    statusText.value = 'Cập nhật thành công! Vui lòng khởi động lại.';
    emit('updated');

    // Auto relaunch after 1.5 seconds or click button
    setTimeout(async () => {
      await relaunchApp();
    }, 1500);

  } catch (err) {
    console.error('Update error:', err);
    error.value = typeof err === 'string' ? err : (err.message || 'Lỗi trong quá trình cập nhật');
    downloading.value = false;
  }
}

async function relaunchApp() {
  try {
    await relaunch();
  } catch (err) {
    console.error('Relaunch failed:', err);
    error.value = 'Không thể tự khởi động lại. Vui lòng tắt app và mở lại bằng tay.';
  }
}
</script>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  background-color: rgba(0, 0, 0, 0.65);
  backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 9999;
  padding: 16px;
  animation: fadeIn 0.2s ease-out;
}

@keyframes fadeIn {
  from { opacity: 0; transform: scale(0.98); }
  to { opacity: 1; transform: scale(1); }
}

.update-card {
  width: 100%;
  max-width: 520px;
  background-color: var(--bg-card, #ffffff);
  border: 1px solid var(--border, #e2e8f0);
  border-radius: var(--radius-lg, 12px);
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.25);
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.update-header {
  padding: 18px 20px;
  background: linear-gradient(135deg, rgba(37, 99, 235, 0.08), rgba(99, 102, 241, 0.12));
  border-bottom: 1px solid var(--border, #e2e8f0);
  display: flex;
  align-items: center;
  gap: 14px;
  position: relative;
}

.header-icon-wrap {
  width: 44px;
  height: 44px;
  border-radius: 10px;
  background: linear-gradient(135deg, #2563eb, #6366f1);
  display: flex;
  align-items: center;
  justify-content: center;
  box-shadow: 0 4px 12px rgba(37, 99, 235, 0.3);
  flex-shrink: 0;
}

.rocket-icon {
  font-size: 22px;
}

.header-info {
  flex: 1;
}

.release-tag {
  font-size: 10.5px;
  font-weight: 700;
  letter-spacing: 0.08em;
  color: #2563eb;
  text-transform: uppercase;
}

.update-title {
  margin: 0;
  font-size: 18px;
  font-weight: 700;
  color: var(--text-ink, #0f172a);
}

.btn-close {
  background: transparent;
  border: none;
  font-size: 16px;
  color: var(--text-muted, #64748b);
  cursor: pointer;
  padding: 4px 8px;
  border-radius: 6px;
}
.btn-close:hover {
  background-color: rgba(0, 0, 0, 0.06);
  color: var(--text-ink, #0f172a);
}

.update-body {
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.version-compare {
  display: flex;
  align-items: center;
  justify-content: space-between;
  background-color: var(--bg-sub, #f8fafc);
  border: 1px solid var(--border-sub, #e2e8f0);
  border-radius: var(--radius-md, 8px);
  padding: 12px 18px;
}

.ver-box {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.ver-label {
  font-size: 11px;
  color: var(--text-muted, #64748b);
}

.ver-num {
  font-size: 15px;
  font-weight: 700;
  color: var(--text-ink, #0f172a);
  font-family: monospace;
}

.ver-box.new .ver-num {
  color: #2563eb;
}

.ver-arrow {
  color: var(--text-muted, #94a3b8);
  font-size: 16px;
}

.changelog-box {
  background-color: var(--bg-surface, #ffffff);
  border: 1px solid var(--border, #e2e8f0);
  border-radius: var(--radius-md, 8px);
  padding: 12px 14px;
  max-height: 180px;
  overflow-y: auto;
}

.changelog-title {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-sub, #475569);
  margin-bottom: 6px;
}

.changelog-content {
  font-size: 12.5px;
  color: var(--text-ink, #1e293b);
  line-height: 1.5;
  white-space: pre-wrap;
  word-break: break-word;
}

.changelog-empty {
  font-size: 12.5px;
  color: var(--text-muted, #64748b);
  font-style: italic;
}

.progress-section {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.progress-labels {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 12px;
}

.progress-text {
  color: var(--text-sub, #475569);
  font-weight: 500;
}

.progress-percent {
  font-weight: 700;
  color: #2563eb;
}

.progress-bar-bg {
  height: 8px;
  background-color: var(--bg-hover, #e2e8f0);
  border-radius: 999px;
  overflow: hidden;
}

.progress-bar-fill {
  height: 100%;
  background: linear-gradient(90deg, #2563eb, #6366f1);
  border-radius: 999px;
  transition: width 0.2s ease;
}

.update-error {
  background-color: #fef2f2;
  color: #dc2626;
  border: 1px solid #fecaca;
  padding: 8px 12px;
  border-radius: 6px;
  font-size: 12px;
}

.update-footer {
  padding: 14px 20px;
  border-top: 1px solid var(--border, #e2e8f0);
  display: flex;
  justify-content: flex-end;
  gap: 10px;
  background-color: var(--bg-sub, #f8fafc);
}

.btn-update-action {
  font-weight: 600;
  padding: 8px 18px;
}
</style>
