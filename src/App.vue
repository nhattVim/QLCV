<template>
  <div class="app-root" :data-theme="isDark ? 'dark' : 'light'">
    <!-- Toast Notifications -->
    <Toast ref="toastRef" />

    <!-- Global Top Update Banner (Shown on Login Screen if update available) -->
    <div v-if="updateAvailable && !currentUser" class="global-update-banner">
      <div class="banner-content">
        <span class="banner-icon">🚀</span>
        <span>Đã có phiên bản mới: <strong>QLCV v{{ updateInfo?.version }}</strong></span>
      </div>
      <button class="btn-banner-download" @click="showUpdateModal = true">
        ⬇️ Tải xuống bản cập nhật
      </button>
    </div>

    <!-- 1. Login View (If not logged in) -->
    <LoginView 
      v-if="!currentUser" 
      :is-dark="isDark"
      @login-success="onLoginSuccess"
      @toggle-theme="toggleTheme"
    />

    <!-- 2. Main Desktop Workstation (Matches Python Layout) -->
    <div v-else class="app-workstation">
      <!-- Top Header Bar (Matching Python _build_header) -->
      <header class="app-header">
        <div class="header-left">
          <span class="header-icon">💼</span>
          <h1 class="header-title">
            {{ currentUser.role === 'admin' ? 'QLCV - BẢNG ĐIỀU KHIỂN QUẢN TRỊ VIÊN' : 'QLCV - KHÔNG GIAN LÀM VIỆC CÁ NHÂN' }}
          </h1>
        </div>

        <div class="header-right">
          <!-- Top Title Bar Update Download Button -->
          <button 
            v-if="updateAvailable" 
            type="button"
            class="btn-title-update pulse-anim" 
            title="Đã có phiên bản mới! Bấm để tải xuống và cập nhật ngay"
            @click="showUpdateModal = true"
          >
            <span class="update-icon">⚡</span>
            <span class="update-text">Có bản mới: <strong>v{{ updateInfo?.version }}</strong></span>
            <span class="update-badge">Tải ngay ⬇️</span>
          </button>

          <div class="user-greeting">
            Xin chào, <strong>{{ currentUser.full_name }}</strong> ({{ currentUser.email }})
            <span class="role-badge" :class="currentUser.role === 'admin' ? 'badge-admin' : 'badge-user'">
              {{ currentUser.role === 'admin' ? 'Quản trị' : 'Cán bộ' }}
            </span>
          </div>

          <button class="btn-secondary btn-header" title="Đổi chế độ Sáng / Tối" @click="toggleTheme">
            🌓 Đổi Theme
          </button>
          <button class="btn-secondary btn-header" title="Làm mới dữ liệu từ MongoDB Atlas" @click="refreshData">
            ↻ Làm mới
          </button>
          <button class="btn-danger btn-header" title="Đăng xuất khỏi hệ thống" @click="onLogout">
            🚪 Đăng xuất
          </button>
        </div>
      </header>

      <!-- Stat Metric Banner (Matching Python _create_stat_card 6 Cards) -->
      <section class="stat-banner">
        <div class="stat-card" style="border-top-color: #6366f1;">
          <span class="stat-label" style="color: #6366f1;">TỔNG CÔNG VIỆC</span>
          <span class="stat-val tabular-nums">{{ stats.total }}</span>
        </div>

        <div class="stat-card" style="border-top-color: #64748b;">
          <span class="stat-label" style="color: #64748b;">CHƯA LÀM</span>
          <span class="stat-val tabular-nums">{{ stats.todo }}</span>
        </div>

        <div class="stat-card" style="border-top-color: #2563eb;">
          <span class="stat-label" style="color: #2563eb;">ĐANG THỰC HIỆN</span>
          <span class="stat-val tabular-nums">{{ stats.in_progress }}</span>
        </div>

        <div class="stat-card" style="border-top-color: #d97706;">
          <span class="stat-label" style="color: #d97706;">CHỜ PHÊ DUYỆT</span>
          <span class="stat-val tabular-nums">{{ stats.review }}</span>
        </div>

        <div class="stat-card" style="border-top-color: #16a34a;">
          <span class="stat-label" style="color: #16a34a;">HOÀN THÀNH</span>
          <span class="stat-val tabular-nums">{{ stats.done }}</span>
        </div>

        <div class="stat-card" style="border-top-color: #dc2626;">
          <span class="stat-label" style="color: #dc2626;">⚠️ QUÁ HẠN</span>
          <span class="stat-val tabular-nums" style="color: #dc2626;">{{ stats.overdue }}</span>
        </div>
      </section>

      <section class="workspace-main">
      <nav class="tabs-nav">
        <button 
          class="tab-btn" 
          :class="{ active: currentTab === 'tasks' }"
          @click="currentTab = 'tasks'"
        >
          Quản lý Công việc ({{ tasks.length }})
        </button>

        <button 
          v-if="currentUser.role === 'admin'"
          class="tab-btn" 
          :class="{ active: currentTab === 'users' }"
          @click="currentTab = 'users'"
        >
          Danh bạ Cán bộ ({{ users.length }})
        </button>

        <button 
          v-if="currentUser.role === 'admin'"
          class="tab-btn" 
          :class="{ active: currentTab === 'reports' }"
          @click="currentTab = 'reports'"
        >
          Giám sát & Báo cáo Tiến độ
        </button>
      </nav>

      <main class="tab-content">
        <!-- Tab 1: Tasks (Kanban Board Jira / Trello) -->
        <TasksView 
          v-if="currentTab === 'tasks'"
          :tasks="tasks"
          :users="users"
          :user-map="userMap"
          :current-user="currentUser"
          @add-task="openNewTaskModal"
          @view-task="openViewTaskModal"
          @edit-task="openEditTaskModal"
          @delete-task="confirmDeleteTask"
          @approve-task="quickApproveTask"
          @status-change="onStatusChange"
        />

        <!-- Tab 2: Users (Admin Only) -->
        <UsersView 
          v-else-if="currentTab === 'users' && currentUser.role === 'admin'"
          :users="users"
          :current-user="currentUser"
          @add-user="openNewUserModal"
          @edit-user="openEditUserModal"
          @toggle-status="toggleUserStatus"
          @delete-user="confirmDeleteUser"
        />

        <!-- Tab 3: Reports (Admin Only) -->
        <ReportsView 
          v-else-if="currentTab === 'reports' && currentUser.role === 'admin'"
          :tasks="tasks"
          :users="users"
          :user-map="userMap"
        />
      </main>
      </section>
    </div>

    <!-- Modals -->
    <!-- Task Modal -->
    <TaskModal 
      v-if="showTaskModal"
      :task-data="selectedTask"
      :users="users"
      :current-user="currentUser"
      @close="showTaskModal = false"
      @save="onSaveTask"
      @attachment-changed="onAttachmentChanged"
    />

    <!-- User Modal -->
    <UserModal 
      v-if="showUserModal"
      :user-data="selectedUser"
      @close="showUserModal = false"
      @save="onSaveUser"
    />

    <!-- Confirmation Dialog -->
    <div v-if="confirmDialog.show" class="modal-overlay" @click.self="confirmDialog.show = false">
      <div class="modal-content" style="max-width: 440px;">
        <div class="modal-header">
          <h3 class="modal-title">{{ confirmDialog.title }}</h3>
          <button class="btn-ghost" @click="confirmDialog.show = false">✕</button>
        </div>
        <div class="modal-body">
          <p style="font-size: 13.5px; line-height: 1.5; white-space: pre-line;">{{ confirmDialog.message }}</p>
        </div>
        <div class="modal-footer">
          <button class="btn-secondary" @click="confirmDialog.show = false">Hủy bỏ</button>
          <button class="btn-danger" @click="executeConfirmAction">Đồng ý</button>
        </div>
      </div>
    </div>
    <!-- Update Modal -->
    <UpdateModal 
      :show="showUpdateModal"
      :update="updateInfo"
      :current-app-version="currentAppVersion"
      @close="showUpdateModal = false"
      @updated="onAppUpdated"
    />
  </div>
</template>

<script setup>
import { ref, reactive, computed, onMounted, onUnmounted, shallowRef, markRaw } from 'vue';
import { getVersion } from '@tauri-apps/api/app';
import { check } from '@tauri-apps/plugin-updater';
import { api } from './api.js';

import Toast from './components/Toast.vue';
import TaskModal from './components/TaskModal.vue';
import UserModal from './components/UserModal.vue';
import UpdateModal from './components/UpdateModal.vue';

import LoginView from './views/LoginView.vue';
import TasksView from './views/TasksView.vue';
import UsersView from './views/UsersView.vue';
import ReportsView from './views/ReportsView.vue';

// State
const toastRef = ref(null);
const currentUser = ref(null);
const currentTab = ref('tasks');
const isDark = ref(false);

const tasks = ref([]);
const users = ref([]);
const stats = ref({
  total: 0,
  todo: 0,
  in_progress: 0,
  review: 0,
  done: 0,
  overdue: 0,
});

// Modals
const showTaskModal = ref(false);
const selectedTask = ref(null);

const showUserModal = ref(false);
const selectedUser = ref(null);

const confirmDialog = reactive({
  show: false,
  title: '',
  message: '',
  action: null,
});

// Auto-Updater State
const updateAvailable = ref(false);
const updateInfo = shallowRef(null);
const showUpdateModal = ref(false);
const currentAppVersion = ref('1.0.0');

async function checkForAppUpdates(silent = true) {
  try {
    const update = await check();
    if (update && update.available) {
      updateAvailable.value = true;
      updateInfo.value = markRaw(update);
      console.log('New update available:', update.version);
      if (!silent) {
        showUpdateModal.value = true;
      }
    } else {
      updateAvailable.value = false;
      if (!silent) {
        toastRef.value?.show('Bạn đang sử dụng phiên bản mới nhất!', 'success');
      }
    }
  } catch (err) {
    console.warn('Check update notice:', err);
    if (!silent) {
      toastRef.value?.show(`Lỗi kiểm tra cập nhật: ${err}`, 'error');
    }
  }
}

function onAppUpdated() {
  updateAvailable.value = false;
  showUpdateModal.value = false;
}

// Map of email -> Full Name
const userMap = computed(() => {
  const map = {};
  users.value.forEach(u => {
    map[u.email] = u.full_name;
  });
  return map;
});

onMounted(async () => {
  // Theme init
  const savedTheme = localStorage.getItem('qlcv_theme');
  if (savedTheme === 'dark') {
    isDark.value = true;
    applyTheme('dark');
  } else {
    applyTheme('light');
  }

  // Restore saved session
  const savedSession = localStorage.getItem('qlcv_session') || sessionStorage.getItem('qlcv_session');
  if (savedSession) {
    try {
      const user = JSON.parse(savedSession);
      if (user && user.email) {
        currentUser.value = user;
        await refreshData();
      }
    } catch {
      localStorage.removeItem('qlcv_session');
    }
  }

  // Keyboard shortcut listeners
  window.addEventListener('keydown', onKeyDown);

  // Read app version
  try {
    currentAppVersion.value = await getVersion();
  } catch (e) {
    console.warn('Could not get app version:', e);
  }

  // Auto check for updates once on startup
  checkForAppUpdates(true);
});

onUnmounted(() => {
  window.removeEventListener('keydown', onKeyDown);
});

function onKeyDown(e) {
  if (e.key === 'Escape') {
    showTaskModal.value = false;
    showUserModal.value = false;
    confirmDialog.show = false;
  } else if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'n') {
    if (currentUser.value?.role === 'admin') {
      e.preventDefault();
      openNewTaskModal();
    }
  }
}

function applyTheme(theme) {
  document.documentElement.setAttribute('data-theme', theme);
  document.documentElement.style.colorScheme = theme;
  localStorage.setItem('qlcv_theme', theme);
}

function toggleTheme() {
  isDark.value = !isDark.value;
  applyTheme(isDark.value ? 'dark' : 'light');
}

function onLoginSuccess(user) {
  currentUser.value = user;
  currentTab.value = 'tasks';
  toastRef.value?.success(`Chào mừng ${user.full_name} đã đăng nhập!`);
  refreshData();
}

function onLogout() {
  localStorage.removeItem('qlcv_session');
  sessionStorage.removeItem('qlcv_session');
  currentUser.value = null;
  tasks.value = [];
  users.value = [];
  toastRef.value?.info('Đã đăng xuất khỏi hệ thống');
}

// Data fetching
async function refreshData() {
  if (!currentUser.value) return;

  try {
    const isUserRole = currentUser.value.role !== 'admin';
    const forUserEmail = isUserRole ? currentUser.value.email : null;

    // Load users
    users.value = await api.getUsers();

    // Load tasks
    tasks.value = await api.getTasks(null, null, forUserEmail);

    // Load stats
    stats.value = await api.getDashboardStats(forUserEmail);

    toastRef.value?.info('Dữ liệu đã được làm mới');
  } catch (err) {
    console.error('Error refreshing data:', err);
    toastRef.value?.warning('Không thể đồng bộ dữ liệu. Kiểm tra kết nối mạng!');
  }
}

// Task Handlers
function openNewTaskModal() {
  selectedTask.value = null;
  showTaskModal.value = true;
}

function openNewTaskWithStatus(statusKey) {
  selectedTask.value = { status: statusKey };
  showTaskModal.value = true;
}

function openViewTaskModal(task) {
  selectedTask.value = { ...task };
  showTaskModal.value = true;
}

function openEditTaskModal(task) {
  selectedTask.value = { ...task };
  showTaskModal.value = true;
}

function onAttachmentChanged({ taskId, attachments }) {
  if (taskId) {
    const task = tasks.value.find((t) => t.id === taskId);
    if (task) {
      task.attachments = [...attachments];
    }
    if (selectedTask.value && selectedTask.value.id === taskId) {
      selectedTask.value.attachments = [...attachments];
    }
  }
}

async function onSaveTask(taskFormData) {
  try {
    if (currentUser.value?.role !== 'admin' && selectedTask.value?.id) {
      // Regular user only changes status
      await api.updateTaskStatus(selectedTask.value.id, taskFormData.status);
      toastRef.value?.success('Đã cập nhật tiến độ công việc!');
    } else if (selectedTask.value?.id) {
      await api.updateTask(selectedTask.value.id, taskFormData);
      toastRef.value?.success('Đã cập nhật công việc thành công!');
    } else {
      await api.createTask({
        ...taskFormData,
        created_by: currentUser.value.email,
      });
      toastRef.value?.success('Đã thêm công việc mới thành công!');
    }
    showTaskModal.value = false;
    await refreshData();
  } catch (err) {
    toastRef.value?.error(`Lỗi: ${typeof err === 'string' ? err : (err.message || 'Thất bại')}`);
  }
}

function confirmDeleteTask(task) {
  confirmDialog.title = 'Xác nhận xóa công việc';
  confirmDialog.message = `Bạn có chắc chắn muốn xóa công việc:\n"${task.title}" không?\nThao tác này không thể hoàn tác!`;
  confirmDialog.action = async () => {
    try {
      await api.deleteTask(task.id);
      toastRef.value?.success('Đã xóa công việc thành công!');
      await refreshData();
    } catch (err) {
      toastRef.value?.error('Không thể xóa công việc!');
    }
  };
  confirmDialog.show = true;
}

async function quickApproveTask(taskId) {
  try {
    await api.updateTaskStatus(taskId, 'DONE');
    toastRef.value?.success('Đã phê duyệt hoàn thành công việc!');
    await refreshData();
  } catch (err) {
    toastRef.value?.error('Lỗi khi phê duyệt công việc!');
  }
}

async function onStatusChange({ id, status }) {
  try {
    await api.updateTaskStatus(id, status);
    toastRef.value?.success('Đã cập nhật trạng thái tiến độ!');
    await refreshData();
  } catch (err) {
    toastRef.value?.error('Không thể cập nhật trạng thái!');
  }
}

// User Handlers
function openNewUserModal() {
  selectedUser.value = null;
  showUserModal.value = true;
}

function openEditUserModal(user) {
  selectedUser.value = { ...user };
  showUserModal.value = true;
}

async function onSaveUser(userFormData) {
  try {
    if (selectedUser.value?.email) {
      await api.updateUser(
        userFormData.email,
        userFormData.full_name,
        userFormData.role,
        userFormData.is_active,
        userFormData.password
      );
      toastRef.value?.success('Đã cập nhật thông tin cán bộ!');
    } else {
      await api.createUser(
        userFormData.full_name,
        userFormData.email,
        userFormData.password,
        userFormData.role
      );
      toastRef.value?.success('Đã thêm cán bộ mới thành công!');
    }
    showUserModal.value = false;
    await refreshData();
  } catch (err) {
    toastRef.value?.error(`Lỗi: ${typeof err === 'string' ? err : (err.message || 'Thất bại')}`);
  }
}

async function toggleUserStatus(user) {
  try {
    const newStatus = !user.is_active;
    await api.updateUser(user.email, null, null, newStatus, null);
    toastRef.value?.success(newStatus ? 'Đã mở khóa tài khoản cán bộ!' : 'Đã khóa tài khoản cán bộ!');
    await refreshData();
  } catch (err) {
    toastRef.value?.error('Lỗi khi cập nhật trạng thái cán bộ!');
  }
}

function confirmDeleteUser(user) {
  confirmDialog.title = 'Xác nhận xóa tài khoản Cán bộ';
  confirmDialog.message = `Bạn có chắc chắn muốn xóa tài khoản cán bộ: "${user.full_name} (${user.email})"?\n\nLưu ý: Cán bộ này cũng sẽ tự động được gỡ bỏ khỏi tất cả các công việc đang được phân công.`;
  confirmDialog.action = async () => {
    try {
      await api.deleteUser(user.email);
      toastRef.value?.success('Đã xóa cán bộ và gỡ khỏi tất cả công việc liên quan!');
      await refreshData();
    } catch (err) {
      toastRef.value?.error('Lỗi khi xóa cán bộ!');
    }
  };
  confirmDialog.show = true;
}

function executeConfirmAction() {
  if (confirmDialog.action) {
    confirmDialog.action();
  }
  confirmDialog.show = false;
}
</script>

<style scoped>
.app-root {
  width: 100vw;
  height: 100vh;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background-color: var(--bg-canvas);
}

.app-workstation {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

/* ================= HEADER (Python Header Style) ================= */
.app-header {
  height: 58px;
  background-color: var(--bg-surface);
  border-bottom: 1px solid var(--border);
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 18px;
  flex-shrink: 0;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 10px;
}

.header-icon {
  font-size: 20px;
}

.header-title {
  font-size: 15px;
  font-weight: 800;
  color: var(--text-ink);
  letter-spacing: -0.01em;
}

.header-right {
  display: flex;
  align-items: center;
  gap: 8px;
}

.user-greeting {
  font-size: 12.5px;
  color: var(--text-muted);
  margin-right: 6px;
  display: flex;
  align-items: center;
  gap: 6px;
}

.role-badge {
  font-size: 10.5px;
  font-weight: 700;
  padding: 1px 6px;
  border-radius: var(--radius-sm);
  border: 1px solid transparent;
}
.badge-admin {
  background-color: #fee2e2;
  color: #dc2626;
  border-color: #fca5a5;
}
.badge-user {
  background-color: #dbeafe;
  color: #2563eb;
  border-color: #93c5fd;
}

.btn-header {
  padding: 5px 10px;
  font-size: 12px;
  font-weight: 500;
}

/* ================= STAT METRIC BANNER (Python 6-Card Banner) ================= */
.stat-banner {
  display: grid;
  grid-template-columns: repeat(6, 1fr);
  gap: 10px;
  padding: 12px 18px 6px;
  flex-shrink: 0;
}

.stat-card {
  background-color: var(--bg-surface);
  border: 1px solid var(--border);
  border-top-width: 3px;
  border-radius: var(--radius-md);
  padding: 10px 12px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
}

.stat-label {
  font-size: 10.5px;
  font-weight: 700;
  letter-spacing: 0.03em;
  text-align: center;
}

.stat-val {
  font-size: 20px;
  font-weight: 800;
  color: var(--text-ink);
  line-height: 1.2;
}

.workspace-main {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  padding: 8px 18px 16px;
}

.tabs-nav {
  display: flex;
  align-items: flex-end;
  gap: 4px;
  padding: 0 8px 0 0;
  flex-shrink: 0;
  position: relative;
  z-index: 1;
}

.tab-btn {
  padding: 9px 16px;
  font-size: 13px;
  font-weight: 600;
  color: var(--text-muted);
  border-radius: var(--radius-md) var(--radius-md) 0 0;
  border: 1px solid transparent;
  border-bottom: none;
  background: transparent;
  margin-bottom: -1px;
}
.tab-btn:hover {
  background-color: var(--bg-hover);
  color: var(--text-ink);
}
.tab-btn.active {
  background-color: var(--bg-surface);
  color: var(--accent);
  border-color: var(--border);
  border-bottom-color: var(--bg-surface);
  font-weight: 700;
}

.tab-content {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding: 14px 16px 28px;
  display: flex;
  flex-direction: column;
  background-color: var(--bg-surface);
  border: 1px solid var(--border);
  border-radius: 0 var(--radius-lg) var(--radius-lg) var(--radius-lg);
}

/* ==========================================
   AUTO-UPDATER TITLE BUTTON & BANNER STYLES
========================================== */
.btn-title-update {
  display: inline-flex;
  align-items: center;
  gap: 7px;
  background: linear-gradient(135deg, #2563eb, #6366f1);
  color: #ffffff;
  border: 1px solid rgba(255, 255, 255, 0.35);
  border-radius: 9999px;
  padding: 4px 12px;
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  box-shadow: 0 2px 10px rgba(37, 99, 235, 0.35);
  margin-right: 6px;
}

.btn-title-update:hover {
  transform: translateY(-1px);
  box-shadow: 0 4px 14px rgba(37, 99, 235, 0.55);
  filter: brightness(1.1);
}

.update-icon {
  font-size: 13px;
}

.update-badge {
  background-color: rgba(255, 255, 255, 0.25);
  padding: 2px 7px;
  border-radius: 9999px;
  font-size: 11px;
  font-weight: 700;
}

@keyframes updatePulseGlow {
  0% { box-shadow: 0 0 0 0 rgba(37, 99, 235, 0.7); }
  70% { box-shadow: 0 0 0 10px rgba(37, 99, 235, 0); }
  100% { box-shadow: 0 0 0 0 rgba(37, 99, 235, 0); }
}

.pulse-anim {
  animation: updatePulseGlow 2.2s infinite;
}

.global-update-banner {
  background: linear-gradient(90deg, #0f172a, #1e293b);
  color: #ffffff;
  border-bottom: 2px solid #2563eb;
  padding: 9px 20px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: 13px;
  position: relative;
  z-index: 1000;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15);
}

.banner-content {
  display: flex;
  align-items: center;
  gap: 10px;
}

.banner-icon {
  font-size: 16px;
}

.btn-banner-download {
  background: linear-gradient(135deg, #2563eb, #6366f1);
  color: #ffffff;
  border: 1px solid rgba(255, 255, 255, 0.25);
  padding: 5px 14px;
  border-radius: 6px;
  font-size: 12.5px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.15s ease;
}

.btn-banner-download:hover {
  filter: brightness(1.15);
  transform: translateY(-1px);
}
</style>
