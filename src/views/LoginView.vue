<template>
  <div class="login-wrapper">
    <!-- Top Bar with Theme Toggle -->
    <div class="login-top-bar">
      <button type="button" class="btn-theme-toggle" @click="toggleTheme">
        {{ isDark ? '☀️ Giao diện Sáng' : '🌙 Giao diện Tối' }}
      </button>
    </div>

    <div class="login-card">
      <!-- Brand Header -->
      <div class="login-header">
        <div class="login-icon">💼</div>
        <h2 class="login-title">QUẢN LÝ CÔNG VIỆC</h2>
        <p class="login-subtitle">Hệ thống phân công & theo dõi tiến độ</p>
      </div>

      <!-- Error Alert -->
      <div v-if="errorMessage" class="login-error-alert">
        <span>⚠️</span> {{ errorMessage }}
      </div>

      <!-- Login Form -->
      <form class="login-form" @submit.prevent="handleLogin">
        <div class="form-group">
          <label class="form-label">Email tài khoản:</label>
          <input 
            v-model="email" 
            type="email" 
            class="form-control" 
            placeholder="Nhập địa chỉ email của bạn..."
            required 
            autofocus
          />
        </div>

        <div class="form-group">
          <label class="form-label">Mật khẩu:</label>
          <input 
            v-model="password" 
            type="password" 
            class="form-control" 
            placeholder="Nhập mật khẩu..."
            required 
          />
        </div>

        <div class="login-options">
          <label class="remember-label">
            <input v-model="rememberMe" type="checkbox" />
            <span>Ghi nhớ phiên đăng nhập trên máy này</span>
          </label>
        </div>

        <button type="submit" class="btn-primary btn-login" :disabled="loading">
          <span v-if="loading">Đang xác thực...</span>
          <span v-else>Đăng nhập vào hệ thống</span>
        </button>
      </form>
    </div>
  </div>
</template>

<script setup>
import { ref } from 'vue';
import { api } from '../api.js';

const emit = defineEmits(['login-success', 'toggle-theme']);

const props = defineProps({
  isDark: {
    type: Boolean,
    default: false,
  },
});

const email = ref('');
const password = ref('');
const rememberMe = ref(true);
const loading = ref(false);
const errorMessage = ref('');

function toggleTheme() {
  emit('toggle-theme');
}

async function handleLogin() {
  if (!email.value || !password.value) return;

  loading.value = true;
  errorMessage.value = '';

  try {
    const user = await api.login(email.value.trim(), password.value);
    
    if (rememberMe.value) {
      localStorage.setItem('qlcv_session', JSON.stringify(user));
    } else {
      sessionStorage.setItem('qlcv_session', JSON.stringify(user));
    }

    emit('login-success', user);
  } catch (err) {
    errorMessage.value = typeof err === 'string' ? err : (err.message || 'Đăng nhập thất bại. Vui lòng kiểm tra lại tài khoản hoặc mật khẩu!');
  } finally {
    loading.value = false;
  }
}
</script>

<style scoped>
.login-wrapper {
  width: 100vw;
  height: 100vh;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  background-color: var(--bg-canvas);
  padding: 20px;
  position: relative;
}

.login-top-bar {
  position: absolute;
  top: 16px;
  right: 20px;
}

.btn-theme-toggle {
  background-color: var(--bg-surface);
  color: var(--text-ink);
  border: 1px solid var(--border);
  padding: 6px 12px;
  border-radius: var(--radius-md);
  font-size: 12px;
  font-weight: 500;
}
.btn-theme-toggle:hover {
  background-color: var(--bg-hover);
}

.login-card {
  width: 100%;
  max-width: 420px;
  background-color: var(--bg-surface);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  padding: 36px 32px 32px;
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.login-header {
  text-align: center;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
}

.login-icon {
  font-size: 38px;
  margin-bottom: 6px;
  line-height: 1;
}

.login-title {
  font-size: 18px;
  font-weight: 800;
  color: var(--text-ink);
  letter-spacing: -0.01em;
}

.login-subtitle {
  font-size: 12.5px;
  color: var(--text-muted);
}

.login-error-alert {
  background-color: var(--danger-sub);
  color: var(--danger);
  border: 1px solid var(--danger);
  border-radius: var(--radius-md);
  padding: 9px 12px;
  font-size: 12.5px;
  font-weight: 500;
  display: flex;
  align-items: center;
  gap: 8px;
}

.login-form {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.login-options {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.remember-label {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12.5px;
  color: var(--text-muted);
  cursor: pointer;
}

.btn-login {
  width: 100%;
  padding: 10px;
  font-size: 14px;
  margin-top: 6px;
  font-weight: 700;
}
</style>
