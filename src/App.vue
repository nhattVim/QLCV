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

    <!-- 2. Main Desktop Workstation -->
    <div v-else class="app-workstation">
      <!-- Top Header Bar -->
      <header class="app-header">
        <div class="header-left">
          <span class="header-icon">💼</span>
          <h1 class="header-title">QLCV</h1>

          <!-- Super Admin Team Quick Selector -->
          <div v-if="isSuperAdmin && teams.length > 0" class="header-select-group" title="Chọn Team để xem dữ liệu">
            <span class="header-select-icon">🏢</span>
            <select v-model="currentTeamId" class="header-select header-team-select" @change="onTeamSelectChange">
              <option v-for="t in teams" :key="t.id" :value="t.id">
                Team: {{ t.name }}
              </option>
            </select>
          </div>

          <!-- Quick Workspace Selector (On Title Bar) -->
          <div v-if="workspaces.length > 0" class="header-select-group" title="Chọn nhanh không gian làm việc">
            <span class="header-select-icon">📁</span>
            <select v-model="currentWorkspaceId" class="header-select header-ws-select" @change="onWorkspaceSelectChange">
              <option v-for="ws in workspaces" :key="ws.id" :value="ws.id">
                {{ ws.name }}
              </option>
            </select>
          </div>
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
            <span class="update-text"><strong>v{{ updateInfo?.version }}</strong></span>
            <span class="update-badge">Tải ngay ⬇️</span>
          </button>

          <!-- User Greeting -->
          <div class="user-greeting">
            Xin chào, <strong>{{ currentUser.full_name }}</strong> ({{ currentUser.email }})
            <span class="role-badge" :class="`badge-${currentUser.role}`">
              {{ roleDisplayName }}
            </span>
          </div>

          <button class="btn-secondary btn-header" title="Đổi chế độ Sáng / Tối" @click="toggleTheme">
            🌓 Đổi Theme
          </button>
          <button 
            class="btn-secondary btn-header" 
            :disabled="isRefreshing"
            title="Làm mới toàn bộ dữ liệu (Đội nhóm, Không gian, Công việc, Thống kê)" 
            @click="refreshData"
          >
            <span :class="{ 'spin-icon': isRefreshing }">↻</span>
            {{ isRefreshing ? 'Đang tải...' : 'Làm mới' }}
          </button>
          <button class="btn-danger btn-header" title="Đăng xuất khỏi hệ thống" @click="onLogout">
            🚪 Đăng xuất
          </button>
        </div>
      </header>

      <!-- Workspace Navigation Tabs -->
      <section class="workspace-main">
        <nav class="tabs-nav">
          <!-- 1. Super Admin: Tab Teams -->
          <button 
            v-if="isSuperAdmin"
            class="tab-btn" 
            :class="{ active: currentTab === 'teams' }"
            @click="currentTab = 'teams'"
          >
            🏢 Quản lý Đội nhóm ({{ teams.length }})
          </button>

          <!-- 2. Workspaces Tab (Available for all) -->
          <button 
            class="tab-btn" 
            :class="{ active: currentTab === 'workspaces' }"
            @click="currentTab = 'workspaces'"
          >
            📁 Không gian làm việc ({{ workspaces.length }})
          </button>

          <!-- 3. Tasks Tab (Tickets of current workspace) -->
          <button 
            class="tab-btn" 
            :class="{ active: currentTab === 'tasks' }"
            @click="currentTab = 'tasks'"
          >
            📋 Quản lý Công việc ({{ tasks.length }})
          </button>

          <!-- 4. Users Tab (Admin & Super Admin) -->
          <button 
            v-if="isAdmin"
            class="tab-btn" 
            :class="{ active: currentTab === 'users' }"
            @click="currentTab = 'users'"
          >
            👥 Danh bạ Cán bộ ({{ staffUsers.length }})
          </button>

          <!-- 5. Reports Tab (Admin & Super Admin) -->
          <button 
            v-if="isAdmin"
            class="tab-btn" 
            :class="{ active: currentTab === 'reports' }"
            @click="currentTab = 'reports'"
          >
            📊 Báo cáo tiến độ
          </button>

          <!-- 6. Team Settings Tab (Admin & Super Admin) -->
          <button 
            v-if="isAdmin"
            class="tab-btn" 
            :class="{ active: currentTab === 'settings' }"
            @click="currentTab = 'settings'"
          >
            ⚙️ Cài đặt Đội nhóm
          </button>
        </nav>

        <main class="tab-content">
          <!-- Tab 1: Teams (Super Admin Only) -->
          <TeamsView 
            v-if="currentTab === 'teams' && isSuperAdmin"
            :teams="teams"
            :current-team-id="currentTeamId"
            @select-team="onSelectTeamFromView"
            @create-team="onCreateTeam"
            @update-team="onUpdateTeam"
            @delete-team="onDeleteTeam"
          />

          <!-- Tab 2: Workspaces -->
          <WorkspacesView 
            v-else-if="currentTab === 'workspaces'"
            :workspaces="workspaces"
            :current-workspace-id="currentWorkspaceId"
            :current-user="currentUser"
            :team-name="currentTeam?.name"
            @select-workspace="onSelectWorkspaceFromView"
            @create-workspace="onCreateWorkspace"
            @update-workspace="onUpdateWorkspace"
            @delete-workspace="onDeleteWorkspace"
          />

          <!-- Tab 3: Tasks (Tickets of current workspace) -->
          <TasksView 
            v-else-if="currentTab === 'tasks'"
            :tasks="tasks"
            :users="users"
            :user-map="userMap"
            :current-user="currentUser"
            :current-team="currentTeam"
            :workspace-name="currentWorkspace?.name"
            :team-name="currentTeam?.name"
            @add-task="openNewTaskModal"
            @view-task="openViewTaskModal"
            @edit-task="openEditTaskModal"
            @delete-task="confirmDeleteTask"
            @approve-task="quickApproveTask"
            @status-change="onStatusChange"
          />

          <!-- Tab 4: Users (Admin & Super Admin) -->
          <UsersView 
            v-else-if="currentTab === 'users' && isAdmin"
            :users="users"
            :teams="teams"
            :current-user="currentUser"
            @add-user="openNewUserModal"
            @edit-user="openEditUserModal"
            @toggle-status="toggleUserStatus"
            @delete-user="confirmDeleteUser"
          />

          <!-- Tab 5: Reports (Admin & Super Admin) -->
          <ReportsView 
            v-else-if="currentTab === 'reports' && isAdmin"
            :tasks="tasks"
            :users="users"
            :user-map="userMap"
            :current-user="currentUser"
            :team-name="currentTeam?.name"
            :current-team="currentTeam"
            :workspace-name="currentWorkspace?.name"
            :workspaces="workspaces"
            @view-task="openViewTaskModal"
            @task-updated="refreshScopedData"
          />

          <!-- Tab 6: Team Settings (Admin & Super Admin) -->
          <TeamSettingsView 
            v-else-if="currentTab === 'settings' && isAdmin"
            :current-team="currentTeam"
            :teams="teams"
            :current-user="currentUser"
            :is-super-admin="isSuperAdmin"
            @update-team-settings="onUpdateTeamSettings"
            @select-team="onSelectTeamFromSettings"
          />
        </main>
      </section>
    </div>

    <!-- Modals -->
    <!-- Task Modal -->
    <TaskModal 
      v-if="showTaskModal"
      :task-data="selectedTask"
      :modal-mode="modalMode"
      :users="users"
      :current-user="currentUser"
      :current-team="currentTeam"
      :current-team-id="currentTeamId"
      :current-workspace-id="currentWorkspaceId"
      :workspaces="workspaces"
      @close="showTaskModal = false"
      @save="onSaveTask"
      @attachment-changed="onAttachmentChanged"
    />

    <!-- User Modal -->
    <UserModal 
      v-if="showUserModal"
      :user-data="selectedUser"
      :teams="teams"
      :current-team-id="currentTeamId"
      :current-user="currentUser"
      @close="showUserModal = false"
      @save="onSaveUser"
    />

    <!-- Global Confirmation Dialog -->
    <div v-if="confirmDialog.show" class="modal-backdrop" @click.self="confirmDialog.show = false">
      <div class="modal-dialog">
        <div class="modal-header">
          <h3 class="modal-title">{{ confirmDialog.title }}</h3>
          <button class="btn-ghost" @click="confirmDialog.show = false">✕</button>
        </div>
        <div class="modal-body">
          <p style="white-space: pre-wrap; font-size: 13.5px; line-height: 1.5;">
            {{ confirmDialog.message }}
          </p>
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
import TeamsView from './views/TeamsView.vue';
import WorkspacesView from './views/WorkspacesView.vue';
import TasksView from './views/TasksView.vue';
import UsersView from './views/UsersView.vue';
import ReportsView from './views/ReportsView.vue';
import TeamSettingsView from './views/TeamSettingsView.vue';

// State
const toastRef = ref(null);
const currentUser = ref(null);
const currentTab = ref('tasks');
const isDark = ref(false);

const teams = ref([]);
const currentTeamId = ref('');

const workspaces = ref([]);
const currentWorkspaceId = ref('');

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

const isRefreshing = ref(false);

// Modals
const showTaskModal = ref(false);
const selectedTask = ref(null);
const modalMode = ref('edit'); // 'create' | 'edit' | 'view'

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

// Computeds
const isSuperAdmin = computed(() => currentUser.value?.role === 'super_admin');
const isAdmin = computed(() => currentUser.value?.role === 'admin' || isSuperAdmin.value);
const staffUsers = computed(() =>
  (users.value || []).filter((u) => u.role !== 'super_admin')
);

const currentTeam = computed(() => {
  return teams.value.find(t => t.id === currentTeamId.value) || null;
});

const currentWorkspace = computed(() => {
  return workspaces.value.find(w => w.id === currentWorkspaceId.value) || null;
});

const roleDisplayName = computed(() => {
  if (isSuperAdmin.value) return 'Super Admin';
  if (currentUser.value?.role === 'admin') return 'Quản trị Team';
  return 'Cán bộ';
});

const headerTitle = computed(() => {
  if (!currentUser.value) return 'QLCV';
  if (isSuperAdmin.value) {
    return currentTeam.value 
      ? `QLCV - SUPER ADMIN [${currentTeam.value.name.toUpperCase()}]`
      : 'QLCV - BẢNG ĐIỀU KHIỂN SUPER ADMIN';
  }
  if (isAdmin.value) {
    return currentTeam.value 
      ? `QLCV - QUẢN TRỊ [${currentTeam.value.name.toUpperCase()}]`
      : 'QLCV - BẢNG ĐIỀU KHIỂN QUẢN TRỊ';
  }
  return currentTeam.value 
    ? `QLCV - [${currentTeam.value.name.toUpperCase()}]`
    : 'QLCV - KHÔNG GIAN LÀM VIỆC CÁ NHÂN';
});

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
        await initAppData();
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
    if (isAdmin.value) {
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

async function onLoginSuccess(user) {
  currentUser.value = user;
  toastRef.value?.success(`Chào mừng ${user.full_name} đã đăng nhập!`);
  await initAppData();
}

function onLogout() {
  localStorage.removeItem('qlcv_session');
  sessionStorage.removeItem('qlcv_session');
  currentUser.value = null;
  tasks.value = [];
  users.value = [];
  teams.value = [];
  workspaces.value = [];
  currentTeamId.value = '';
  currentWorkspaceId.value = '';
}

// ==========================================
// DATA INITIALIZATION & RESTORATION
// ==========================================

async function initAppData(retries = 3) {
  if (!currentUser.value) return;

  try {
    // 1. Fetch Teams with automatic retry on startup
    let loadedTeams = null;
    for (let attempt = 1; attempt <= retries; attempt++) {
      try {
        loadedTeams = await api.getTeams();
        break;
      } catch (e) {
        if (attempt === retries) throw e;
        await new Promise(r => setTimeout(r, 400));
      }
    }
    teams.value = loadedTeams || [];

    // 2. Determine active team
    const savedTeamKey = 'qlcv_last_team_' + currentUser.value.email;
    const savedTeamId = localStorage.getItem(savedTeamKey);

    if (savedTeamId && teams.value.some(t => t.id === savedTeamId)) {
      currentTeamId.value = savedTeamId;
    } else if (currentUser.value.team_id && teams.value.some(t => t.id === currentUser.value.team_id)) {
      currentTeamId.value = currentUser.value.team_id;
    } else if (teams.value.length > 0) {
      currentTeamId.value = teams.value[0].id;
    } else {
      currentTeamId.value = '';
    }

    if (currentTeamId.value) {
      localStorage.setItem(savedTeamKey, currentTeamId.value);
    }

    // 3. Fetch Workspaces for current team
    await loadWorkspacesForCurrentTeam();

    // 4. Set default tab to 'tasks' as requested
    currentTab.value = 'tasks';

    // 5. Load tasks, users, and stats
    await refreshScopedData();
  } catch (err) {
    console.error('Error in initAppData:', err);
    toastRef.value?.warning('Đang chờ kết nối dữ liệu từ máy chủ...');
  }
}

async function loadWorkspacesForCurrentTeam() {
  if (!currentTeamId.value) {
    workspaces.value = [];
    currentWorkspaceId.value = '';
    return;
  }

  workspaces.value = await api.getWorkspaces(currentTeamId.value);

  const savedWsKey = `qlcv_last_ws_${currentUser.value.email}_${currentTeamId.value}`;
  const savedWsId = localStorage.getItem(savedWsKey);

  if (savedWsId && workspaces.value.some(w => w.id === savedWsId)) {
    currentWorkspaceId.value = savedWsId;
  } else if (workspaces.value.length > 0) {
    currentWorkspaceId.value = workspaces.value[0].id;
  } else {
    currentWorkspaceId.value = '';
  }

  if (currentWorkspaceId.value) {
    localStorage.setItem(savedWsKey, currentWorkspaceId.value);
  }
}

function updateLocalStats() {
  const now = new Date();
  const list = tasks.value || [];
  stats.value = {
    total: list.length,
    todo: list.filter(t => t.status === 'TODO').length,
    in_progress: list.filter(t => t.status === 'IN_PROGRESS').length,
    review: list.filter(t => t.status === 'REVIEW').length,
    done: list.filter(t => t.status === 'DONE').length,
    overdue: list.filter(t => {
      if (t.status === 'DONE' || !t.due_date) return false;
      return new Date(t.due_date) < now;
    }).length,
  };
}

// Refresh tasks, users, and stats according to selected team and workspace
async function refreshScopedData() {
  if (!currentUser.value) return;

  try {
    const isRegularUser = currentUser.value.role === 'user';
    const forUserEmail = isRegularUser ? currentUser.value.email : null;

    // Load in parallel with Promise.all for 3x faster response
    const [loadedUsers, loadedTasks, loadedStats] = await Promise.all([
      api.getUsers(currentTeamId.value || null),
      api.getTasks(
        currentTeamId.value || null,
        currentWorkspaceId.value || null,
        null,
        null,
        forUserEmail
      ),
      api.getDashboardStats(
        currentTeamId.value || null,
        currentWorkspaceId.value || null,
        forUserEmail
      ),
    ]);

    users.value = loadedUsers;
    tasks.value = loadedTasks;
    stats.value = loadedStats;
  } catch (err) {
    console.error('Error refreshing scoped data:', err);
  }
}

async function refreshData() {
  if (!currentUser.value || isRefreshing.value) return;
  isRefreshing.value = true;

  try {
    // 1. Tải mới toàn bộ danh sách Đội nhóm (Teams)
    teams.value = await api.getTeams();

    // 2. Đồng bộ và kiểm tra Team ID hiện tại
    if (currentUser.value.role !== 'super_admin') {
      if (currentUser.value.team_id) {
        currentTeamId.value = currentUser.value.team_id;
      } else if (teams.value.length > 0 && !teams.value.some(t => t.id === currentTeamId.value)) {
        currentTeamId.value = teams.value[0].id;
      }
    } else {
      // Super Admin: nếu team đang chọn không còn hợp lệ thì fallback về team đầu tiên
      if (currentTeamId.value && !teams.value.some(t => t.id === currentTeamId.value)) {
        currentTeamId.value = teams.value[0]?.id || '';
      } else if (!currentTeamId.value && teams.value.length > 0) {
        currentTeamId.value = teams.value[0].id;
      }
    }

    // 3. Tải mới danh sách Không gian làm việc (Workspaces) của team hiện tại
    await loadWorkspacesForCurrentTeam();

    // 4. Tải mới Công việc (Tasks), Cán bộ (Users) và Thống kê (Stats)
    await refreshScopedData();
  } catch (err) {
    console.error('Lỗi refreshData:', err);
    toastRef.value?.warning('Lỗi làm mới dữ liệu!');
  } finally {
    isRefreshing.value = false;
  }
}

// ==========================================
// SELECTION HANDLERS
// ==========================================

async function onTeamSelectChange() {
  if (!currentUser.value) return;
  localStorage.setItem('qlcv_last_team_' + currentUser.value.email, currentTeamId.value);

  await loadWorkspacesForCurrentTeam();
  await refreshScopedData();
  toastRef.value?.info(`Đang truy cập: ${currentTeam.value?.name || 'Team'}`);
}

async function onWorkspaceSelectChange() {
  if (!currentUser.value) return;
  if (currentTeamId.value) {
    localStorage.setItem(`qlcv_last_ws_${currentUser.value.email}_${currentTeamId.value}`, currentWorkspaceId.value);
  }
  await refreshScopedData();
}

async function onSelectTeamFromView(teamId) {
  currentTeamId.value = teamId;
  await onTeamSelectChange();
  currentTab.value = 'workspaces';
}

async function onSelectTeamFromSettings(teamId) {
  if (currentTeamId.value !== teamId) {
    currentTeamId.value = teamId;
    await onTeamSelectChange();
  }
}

async function onSelectWorkspaceFromView(workspaceId) {
  currentWorkspaceId.value = workspaceId;
  await onWorkspaceSelectChange();
  currentTab.value = 'tasks';
  toastRef.value?.success(`Đang mở: ${currentWorkspace.value?.name || 'Không gian làm việc'}`);
}

// ==========================================
// TEAMS CRUD (SUPER ADMIN)
// ==========================================

async function onCreateTeam({ name, description, enable_review_step, enable_result_column }) {
  try {
    const newTeam = await api.createTeam(name, description, enable_review_step, enable_result_column);
    teams.value = await api.getTeams();
    currentTeamId.value = newTeam.id;
    await onTeamSelectChange();
    toastRef.value?.success(`Đã tạo đội nhóm "${name}" thành công!`);
  } catch (err) {
    toastRef.value?.error(`Lỗi tạo team: ${err}`);
  }
}

async function onUpdateTeam({ id, name, description, enable_review_step, enable_result_column }) {
  try {
    await api.updateTeam(id, name, description, enable_review_step, enable_result_column);
    teams.value = await api.getTeams();
    await refreshScopedData();
    toastRef.value?.success(`Đã cập nhật đội nhóm "${name}"!`);
  } catch (err) {
    toastRef.value?.error(`Lỗi cập nhật team: ${err}`);
  }
}

async function onUpdateTeamSettings({ id, name, description, enable_review_step, enable_result_column }) {
  try {
    await api.updateTeam(id, name, description, enable_review_step, enable_result_column);
    teams.value = await api.getTeams();
    await refreshScopedData();
    toastRef.value?.success('Đã lưu cấu hình cài đặt đội nhóm thành công!');
  } catch (err) {
    toastRef.value?.error(`Lỗi lưu cấu hình: ${err}`);
  }
}

async function onDeleteTeam(teamId) {
  try {
    await api.deleteTeam(teamId);
    teams.value = await api.getTeams();
    if (currentTeamId.value === teamId) {
      currentTeamId.value = teams.value[0]?.id || '';
      await onTeamSelectChange();
    }
    toastRef.value?.success('Đã xóa đội nhóm thành công!');
  } catch (err) {
    toastRef.value?.error(`Lỗi xóa team: ${err}`);
  }
}

// ==========================================
// WORKSPACES CRUD
// ==========================================

async function onCreateWorkspace({ name, description }) {
  try {
    const newWs = await api.createWorkspace(
      currentTeamId.value,
      name,
      description,
      currentUser.value.email
    );
    await loadWorkspacesForCurrentTeam();
    currentWorkspaceId.value = newWs.id;
    await onWorkspaceSelectChange();
    toastRef.value?.success(`Đã tạo không gian làm việc "${name}"!`);
  } catch (err) {
    toastRef.value?.error(`Lỗi tạo không gian: ${err}`);
  }
}

async function onUpdateWorkspace({ id, name, description }) {
  try {
    await api.updateWorkspace(id, name, description);
    await loadWorkspacesForCurrentTeam();
    toastRef.value?.success(`Đã cập nhật không gian làm việc "${name}"!`);
  } catch (err) {
    toastRef.value?.error(`Lỗi cập nhật không gian: ${err}`);
  }
}

async function onDeleteWorkspace(workspaceId) {
  try {
    await api.deleteWorkspace(workspaceId);
    await loadWorkspacesForCurrentTeam();
    await refreshScopedData();
    toastRef.value?.success('Đã xóa không gian làm việc!');
  } catch (err) {
    toastRef.value?.error(`Lỗi xóa không gian: ${err}`);
  }
}

// ==========================================
// TASK HANDLERS
// ==========================================

function openNewTaskModal() {
  selectedTask.value = {
    workspace_id: currentWorkspaceId.value,
    team_id: currentTeamId.value,
  };
  modalMode.value = 'create';
  showTaskModal.value = true;
}

function openViewTaskModal(task) {
  selectedTask.value = { ...task };
  modalMode.value = 'view';
  showTaskModal.value = true;
}

function openEditTaskModal(task) {
  selectedTask.value = { ...task };
  modalMode.value = 'edit';
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
    if (!isAdmin.value && selectedTask.value?.id) {
      // Regular user updates status & result
      await api.updateTaskStatus(selectedTask.value.id, taskFormData.status, taskFormData.result);
      toastRef.value?.success('Đã cập nhật tiến độ công việc!');
    } else if (selectedTask.value?.id) {
      await api.updateTask(selectedTask.value.id, {
        ...taskFormData,
        team_id: taskFormData.team_id || currentTeamId.value,
        workspace_id: taskFormData.workspace_id || currentWorkspaceId.value,
      });
      toastRef.value?.success('Đã cập nhật công việc thành công!');
    } else {
      await api.createTask({
        ...taskFormData,
        team_id: taskFormData.team_id || currentTeamId.value,
        workspace_id: taskFormData.workspace_id || currentWorkspaceId.value,
        created_by: currentUser.value.email,
      });
      toastRef.value?.success('Đã thêm công việc mới thành công!');
    }
    showTaskModal.value = false;
    await refreshScopedData();
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
      await refreshScopedData();
    } catch (err) {
      toastRef.value?.error('Không thể xóa công việc!');
    }
  };
  confirmDialog.show = true;
}

async function quickApproveTask(taskId) {
  await onStatusChange({ id: taskId, status: 'DONE' });
  toastRef.value?.success('Đã phê duyệt hoàn thành công việc!');
}

async function onStatusChange({ id, status, result }) {
  const task = tasks.value.find(t => t.id === id);
  if (!task) return;
  const oldStatus = task.status;
  const oldResult = task.result;
  const oldCompletedAt = task.completed_at;
  if (oldStatus === status && (result === undefined || oldResult === result)) return;

  // 1. Phản hồi giao diện tức thì (Optimistic 0ms UI update)
  task.status = status;
  if (result !== undefined && result !== null) {
    task.result = result;
  }
  if (status === 'DONE') {
    if (!task.completed_at) {
      task.completed_at = new Date().toISOString();
    }
  } else {
    task.completed_at = null;
  }
  updateLocalStats();

  // 2. Đồng bộ ngầm lên MongoDB Atlas mà không gây đứng hình/giật lag
  try {
    await api.updateTaskStatus(id, status, result);
  } catch (err) {
    // Hoàn tác nếu có lỗi mạng
    task.status = oldStatus;
    task.result = oldResult;
    task.completed_at = oldCompletedAt;
    updateLocalStats();
    toastRef.value?.error('Lỗi mạng: Không thể cập nhật trạng thái trên máy chủ!');
  }
}

// ==========================================
// USER HANDLERS
// ==========================================

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
        userFormData.password,
        userFormData.team_id || null,
        userFormData.job_title || null
      );
      toastRef.value?.success('Đã cập nhật thông tin cán bộ!');
    } else {
      await api.createUser(
        userFormData.full_name,
        userFormData.email,
        userFormData.password,
        userFormData.role,
        userFormData.team_id || currentTeamId.value || null,
        userFormData.job_title || null
      );
      toastRef.value?.success('Đã thêm cán bộ mới thành công!');
    }
    showUserModal.value = false;
    await refreshScopedData();
  } catch (err) {
    toastRef.value?.error(`Lỗi: ${typeof err === 'string' ? err : (err.message || 'Thất bại')}`);
  }
}

async function toggleUserStatus(user) {
  try {
    await api.updateUser(user.email, null, null, !user.is_active, null);
    toastRef.value?.success(`Đã ${!user.is_active ? 'mở khóa' : 'khóa'} tài khoản ${user.full_name}!`);
    await refreshScopedData();
  } catch (err) {
    toastRef.value?.error('Lỗi khi thay đổi trạng thái tài khoản!');
  }
}

function confirmDeleteUser(user) {
  confirmDialog.title = 'Xác nhận xóa tài khoản Cán bộ';
  confirmDialog.message = `Bạn có chắc chắn muốn xóa tài khoản cán bộ: "${user.full_name} (${user.email})"?\n\nLưu ý: Cán bộ này cũng sẽ tự động được gỡ bỏ khỏi tất cả các công việc đang được phân công.`;
  confirmDialog.action = async () => {
    try {
      await api.deleteUser(user.email);
      toastRef.value?.success('Đã xóa cán bộ và gỡ khỏi tất cả công việc liên quan!');
      await refreshScopedData();
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

// Updater Check
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

/* ================= HEADER ================= */
.app-header {
  height: 58px;
  background-color: var(--bg-surface);
  border-bottom: 1px solid var(--border);
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 18px;
  flex-shrink: 0;
  gap: 12px;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 10px;
  min-width: 0;
}

.header-icon {
  font-size: 20px;
}

.header-title {
  font-size: 14.5px;
  font-weight: 800;
  color: var(--text-ink);
  letter-spacing: -0.01em;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.header-right {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
}

/* Quick Selectors on Header */
.header-select-group {
  display: flex;
  align-items: center;
  gap: 6px;
  background-color: var(--bg-sub);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  padding: 2px 8px;
}

.header-select-icon {
  font-size: 13px;
}

.header-select {
  border: none;
  background: transparent;
  color: var(--text-ink);
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
  outline: none;
  max-width: 170px;
}

.header-select:focus {
  outline: none;
}

.header-team-select {
  color: #7e22ce;
}
[data-theme="dark"] .header-team-select {
  color: #c084fc;
}

.header-ws-select {
  color: var(--accent);
}

.user-greeting {
  font-size: 12.5px;
  color: var(--text-muted);
  margin-right: 4px;
  display: flex;
  align-items: center;
  gap: 6px;
  white-space: nowrap;
}

.role-badge {
  font-size: 10.5px;
  font-weight: 700;
  padding: 1px 7px;
  border-radius: var(--radius-sm);
  border: 1px solid transparent;
}

.badge-super_admin {
  background-color: #f3e8ff;
  color: #7e22ce;
  border-color: #d8b4fe;
}
[data-theme="dark"] .badge-super_admin {
  background-color: #3b0764;
  color: #e9d5ff;
  border-color: #9333ea;
}

.badge-admin {
  background-color: #fee2e2;
  color: #dc2626;
  border-color: #fca5a5;
}
[data-theme="dark"] .badge-admin {
  background-color: #450a0a;
  color: #f87171;
  border-color: #991b1b;
}

.badge-user {
  background-color: #dbeafe;
  color: #2563eb;
  border-color: #93c5fd;
}
[data-theme="dark"] .badge-user {
  background-color: #172554;
  color: #60a5fa;
  border-color: #1d4ed8;
}

.btn-header {
  padding: 5px 10px;
  font-size: 12px;
  font-weight: 500;
}

.spin-icon {
  display: inline-block;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
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
  cursor: pointer;
  transition: background-color 0.15s ease, color 0.15s ease;
}

.tab-btn:first-child {
  border-top-left-radius: var(--radius-lg);
}

.tab-btn:hover {
  background-color: var(--bg-hover);
  color: var(--text-ink);
}

.tab-btn.active {
  background-color: var(--bg-surface);
  color: var(--accent);
  border: 1px solid var(--border);
  border-bottom-color: var(--bg-surface);
  font-weight: 700;
  z-index: 2;
}

.tab-content {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding: 12px 14px;
  display: flex;
  flex-direction: column;
  background-color: var(--bg-surface);
  border: 1px solid var(--border);
  border-radius: 0 var(--radius-lg) var(--radius-lg) var(--radius-lg);
}

/* ================= AUTO-UPDATER TITLE BUTTON & BANNER ================= */
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

/* Modals */
.modal-backdrop {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 999999;
}

.modal-dialog {
  background-color: var(--bg-surface);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  width: 90%;
  max-width: 440px;
  overflow: hidden;
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.25);
}

.modal-header {
  padding: 12px 16px;
  border-bottom: 1px solid var(--border);
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.modal-title {
  font-size: 14.5px;
  font-weight: 700;
  color: var(--text-ink);
}

.modal-body {
  padding: 16px;
}

.modal-footer {
  padding: 10px 16px;
  border-top: 1px solid var(--border);
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 8px;
  background-color: var(--bg-sub);
}
</style>
