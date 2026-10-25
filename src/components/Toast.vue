<template>
  <div class="toast-container" v-if="toasts.length > 0">
    <div 
      v-for="toast in toasts" 
      :key="toast.id" 
      class="toast-item"
      :class="`toast-${toast.type || 'info'}`"
    >
      <div class="toast-icon">
        <span v-if="toast.type === 'success'">✓</span>
        <span v-else-if="toast.type === 'error'">✕</span>
        <span v-else-if="toast.type === 'warning'">⚠️</span>
        <span v-else>ℹ</span>
      </div>
      <div class="toast-message">{{ toast.message }}</div>
      <button class="toast-close" @click="removeToast(toast.id)">✕</button>
    </div>
  </div>
</template>

<script setup>
import { ref } from 'vue';

const toasts = ref([]);
let nextId = 1;

function showToast(message, type = 'info', duration = 3500) {
  const id = nextId++;
  toasts.value.push({ id, message, type });
  if (duration > 0) {
    setTimeout(() => {
      removeToast(id);
    }, duration);
  }
}

function removeToast(id) {
  toasts.value = toasts.value.filter(t => t.id !== id);
}

defineExpose({
  show: showToast,
  success: (msg, dur) => showToast(msg, 'success', dur),
  error: (msg, dur) => showToast(msg, 'error', dur),
  warning: (msg, dur) => showToast(msg, 'warning', dur),
  info: (msg, dur) => showToast(msg, 'info', dur),
});
</script>

<style scoped>
.toast-container {
  position: fixed;
  top: 18px;
  right: 20px;
  z-index: 9999;
  display: flex;
  flex-direction: column;
  gap: 8px;
  pointer-events: none;
}

.toast-item {
  pointer-events: auto;
  display: flex;
  align-items: center;
  gap: 10px;
  min-width: 280px;
  max-width: 420px;
  padding: 10px 14px;
  border-radius: var(--radius-md);
  box-shadow: var(--shadow-lg);
  font-size: 13px;
  font-weight: 500;
  animation: toastIn 0.25s cubic-bezier(0.16, 1, 0.3, 1);
  border: 1px solid transparent;
}

@keyframes toastIn {
  from {
    opacity: 0;
    transform: translateY(-10px) scale(0.95);
  }
  to {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
}

.toast-success {
  background-color: var(--success-sub);
  color: #047857;
  border-color: #6ee7b7;
}
[data-theme="dark"] .toast-success {
  background-color: #064e3b;
  color: #a7f3d0;
  border-color: #047857;
}

.toast-error {
  background-color: var(--danger-sub);
  color: #b91c1c;
  border-color: #fca5a5;
}
[data-theme="dark"] .toast-error {
  background-color: #450a0a;
  color: #fca5a5;
  border-color: #991b1b;
}

.toast-warning {
  background-color: var(--warning-sub);
  color: #b45309;
  border-color: #fcd34d;
}
[data-theme="dark"] .toast-warning {
  background-color: #451a03;
  color: #fde68a;
  border-color: #b45309;
}

.toast-info {
  background-color: var(--accent-sub);
  color: var(--accent);
  border-color: #bfdbfe;
}
[data-theme="dark"] .toast-info {
  background-color: #172554;
  color: #93c5fd;
  border-color: #1e3a8a;
}

.toast-icon {
  font-weight: 700;
  font-size: 14px;
}

.toast-message {
  flex: 1;
  word-break: break-word;
}

.toast-close {
  background: none;
  border: none;
  color: inherit;
  opacity: 0.6;
  cursor: pointer;
  padding: 2px 4px;
  border-radius: var(--radius-sm);
  font-size: 11px;
}
.toast-close:hover {
  opacity: 1;
}
</style>
