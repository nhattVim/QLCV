<template>
  <div class="teams-container">
    <!-- Header & Action Bar -->
    <div class="teams-toolbar">
      <div class="toolbar-left">
        <h2 class="view-title">🏢 Quản lý Các Đội Nhóm (Teams)</h2>
        <span class="view-subtitle">Super Admin có thể xem, tạo, chuyển đổi và quản lý toàn bộ các Team trong hệ thống</span>
      </div>

      <div class="toolbar-right">
        <div class="search-box">
          <input 
            v-model="searchKeyword" 
            type="text" 
            placeholder="🔍 Tìm kiếm đội nhóm..." 
            class="search-input"
          />
          <button v-if="searchKeyword" class="btn-clear" @click="searchKeyword = ''">✕</button>
        </div>

        <button class="btn-primary" @click="openCreateModal">
          ➕ Thêm Đội Nhóm Mới
        </button>
      </div>
    </div>

    <!-- Teams Grid -->
    <div v-if="filteredTeams.length > 0" class="teams-grid">
      <div 
        v-for="team in filteredTeams" 
        :key="team.id" 
        class="team-card"
        :class="{ 'team-card-active': currentTeamId === team.id }"
        @click="selectTeam(team)"
      >
        <!-- Card Header -->
        <div class="card-top">
          <div class="team-badge-wrap">
            <span class="team-avatar-icon">👥</span>
            <div>
              <h3 class="team-name">{{ team.name }}</h3>
              <span class="team-date">Tạo ngày: {{ formatDate(team.created_at) }}</span>
            </div>
          </div>

          <span v-if="currentTeamId === team.id" class="badge-active-team">
            ● Đang chọn
          </span>
        </div>

        <!-- Description -->
        <p class="team-desc">
          {{ team.description || 'Chưa có mô tả cho đội nhóm này.' }}
        </p>

        <!-- Metrics Row -->
        <div class="team-metrics">
          <div class="metric-item" title="Số lượng cán bộ trong team">
            <span class="metric-num">{{ team.member_count || 0 }}</span>
            <span class="metric-label">👤 Cán bộ</span>
          </div>

          <div class="metric-item" title="Số không gian làm việc">
            <span class="metric-num">{{ team.workspace_count || 0 }}</span>
            <span class="metric-label">📁 Không gian</span>
          </div>

          <div class="metric-item" title="Tổng số công việc">
            <span class="metric-num">{{ team.task_count || 0 }}</span>
            <span class="metric-label">📋 Công việc</span>
          </div>
        </div>

        <!-- Card Actions -->
        <div class="card-actions" @click.stop>
          <button 
            class="btn-switch-team"
            :class="{ 'btn-switch-active': currentTeamId === team.id }"
            @click="selectTeam(team)"
          >
            {{ currentTeamId === team.id ? '✓ Đang làm việc' : '🚀 Truy cập Team này' }}
          </button>

          <div class="actions-group">
            <button class="btn-action-icon" title="Chỉnh sửa thông tin Team" @click="openEditModal(team)">
              ✏️
            </button>
            <button class="btn-action-icon btn-action-danger" title="Xóa Team này" @click="confirmDelete(team)">
              🗑️
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Empty State -->
    <div v-else class="empty-state">
      <span class="empty-icon">🏢</span>
      <p class="empty-text">Chưa có đội nhóm nào phù hợp.</p>
      <button class="btn-primary" @click="openCreateModal">
        ➕ Tạo Đội Nhóm Đầu Tiên
      </button>
    </div>

    <!-- Modal Create / Edit Team -->
    <div v-if="showModal" class="modal-backdrop" @click.self="showModal = false">
      <div class="modal-content">
        <div class="modal-header">
          <h3>{{ isEditing ? '✏️ Chỉnh Sửa Đội Nhóm' : '➕ Tạo Đội Nhóm Mới' }}</h3>
          <button class="btn-close" @click="showModal = false">✕</button>
        </div>

        <form class="modal-body" @submit.prevent="saveTeam">
          <div class="form-group">
            <label class="form-label">Tên đội nhóm: <span class="text-danger">*</span></label>
            <input 
              v-model="form.name" 
              type="text" 
              class="form-input" 
              placeholder="Ví dụ: Đội Kỹ Thuật, Phòng Marketing, Tổ Hành Chính..." 
              required
              autofocus
            />
          </div>

          <div class="form-group">
            <label class="form-label">Mô tả đội nhóm:</label>
            <textarea 
              v-model="form.description" 
              class="form-textarea" 
              rows="3" 
              placeholder="Mô tả chức năng, nhiệm vụ hoặc mục đích của team..."
            ></textarea>
          </div>

          <div class="modal-actions">
            <button type="button" class="btn-secondary" @click="showModal = false">
              Hủy
            </button>
            <button type="submit" class="btn-primary" :disabled="isSaving">
              {{ isSaving ? 'Đang lưu...' : (isEditing ? 'Cập nhật' : 'Tạo mới') }}
            </button>
          </div>
        </form>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed } from 'vue';

const props = defineProps({
  teams: {
    type: Array,
    default: () => [],
  },
  currentTeamId: {
    type: String,
    default: '',
  },
});

const emit = defineEmits(['select-team', 'create-team', 'update-team', 'delete-team']);

const searchKeyword = ref('');
const showModal = ref(false);
const isEditing = ref(false);
const isSaving = ref(false);
const editingId = ref(null);

const form = ref({
  name: '',
  description: '',
});

const filteredTeams = computed(() => {
  const q = searchKeyword.value.trim().toLowerCase();
  if (!q) return props.teams;
  return props.teams.filter(t => 
    (t.name || '').toLowerCase().includes(q) || 
    (t.description || '').toLowerCase().includes(q)
  );
});

function selectTeam(team) {
  emit('select-team', team.id);
}

function openCreateModal() {
  isEditing.value = false;
  editingId.value = null;
  form.value = {
    name: '',
    description: '',
  };
  showModal.value = true;
}

function openEditModal(team) {
  isEditing.value = true;
  editingId.value = team.id;
  form.value = {
    name: team.name,
    description: team.description || '',
  };
  showModal.value = true;
}

async function saveTeam() {
  if (!form.value.name.trim()) return;
  isSaving.value = true;
  try {
    if (isEditing.value) {
      await emit('update-team', {
        id: editingId.value,
        name: form.value.name.trim(),
        description: form.value.description.trim(),
      });
    } else {
      await emit('create-team', {
        name: form.value.name.trim(),
        description: form.value.description.trim(),
      });
    }
    showModal.value = false;
  } finally {
    isSaving.value = false;
  }
}

function confirmDelete(team) {
  if (confirm(`Bạn có chắc chắn muốn xóa đội nhóm "${team.name}"? Toàn bộ không gian làm việc và công việc trong team này cũng sẽ bị xóa vĩnh viễn!`)) {
    emit('delete-team', team.id);
  }
}

function formatDate(dateStr) {
  if (!dateStr) return '—';
  try {
    const d = new Date(dateStr);
    const pad = (n) => String(n).padStart(2, '0');
    return `${pad(d.getDate())}/${pad(d.getMonth() + 1)}/${d.getFullYear()}`;
  } catch {
    return dateStr;
  }
}
</script>

<style scoped>
.teams-container {
  display: flex;
  flex-direction: column;
  gap: 16px;
  height: 100%;
}

.teams-toolbar {
  background-color: var(--bg-surface);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  padding: 12px 16px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  flex-wrap: wrap;
}

.view-title {
  font-size: 16px;
  font-weight: 700;
  color: var(--text-ink);
  margin-bottom: 2px;
}

.view-subtitle {
  font-size: 12px;
  color: var(--text-muted);
}

.toolbar-right {
  display: flex;
  align-items: center;
  gap: 10px;
}

.search-box {
  position: relative;
  display: flex;
  align-items: center;
  min-width: 220px;
}

.search-input {
  width: 100%;
  padding: 6px 26px 6px 10px;
  border-radius: var(--radius-sm);
  border: 1px solid var(--border);
  background-color: var(--bg-surface);
  color: var(--text-ink);
  font-size: 12.5px;
}

.btn-clear {
  position: absolute;
  right: 8px;
  background: transparent;
  border: none;
  color: var(--text-sub);
  font-size: 12px;
  cursor: pointer;
}

/* Grid Layout */
.teams-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(290px, 1fr));
  gap: 12px;
}

.team-card {
  background-color: var(--bg-card);
  border: 1px solid var(--border);
  border-left: 3.5px solid var(--border);
  border-radius: var(--radius-md);
  padding: 12px 14px;
  display: flex;
  flex-direction: column;
  gap: 10px;
  cursor: pointer;
  transition: border-color 0.12s ease;
  position: relative;
}

.team-card:hover {
  border-color: var(--accent);
}

.team-card-active {
  border-color: var(--accent);
  border-left-color: var(--accent);
  background-color: var(--bg-card);
}

.card-top {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 8px;
}

.team-badge-wrap {
  display: flex;
  align-items: center;
  gap: 8px;
}

.team-avatar-icon {
  font-size: 18px;
  line-height: 1;
}

.team-name {
  font-size: 14px;
  font-weight: 700;
  color: var(--text-ink);
  line-height: 1.3;
}

.team-date {
  font-size: 11px;
  color: var(--text-sub);
}

.badge-active-team {
  font-size: 11px;
  font-weight: 600;
  color: var(--accent);
  background-color: var(--accent-sub);
  border: 1px solid var(--accent);
  padding: 1px 7px;
  border-radius: var(--radius-sm);
}

.team-desc {
  font-size: 12px;
  color: var(--text-muted);
  line-height: 1.45;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
  min-height: 32px;
}

.team-metrics {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 6px;
  background-color: var(--bg-surface);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  padding: 6px 4px;
  text-align: center;
}

.metric-item {
  display: flex;
  flex-direction: column;
  gap: 1px;
}

.metric-num {
  font-size: 13px;
  font-weight: 700;
  color: var(--text-ink);
}

.metric-label {
  font-size: 10.5px;
  color: var(--text-muted);
}

.card-actions {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  padding-top: 8px;
  border-top: 1px solid var(--border);
  margin-top: 2px;
}

.btn-switch-team {
  flex: 1;
  padding: 5px 10px;
  font-size: 12px;
  font-weight: 500;
  border-radius: var(--radius-sm);
  background-color: var(--bg-surface);
  border: 1px solid var(--border);
  color: var(--text-ink);
  cursor: pointer;
  transition: all 0.12s ease;
}

.btn-switch-team:hover {
  background-color: var(--bg-hover);
  border-color: var(--border-focus);
}

.btn-switch-active {
  background-color: var(--accent-sub);
  color: var(--accent);
  border-color: var(--accent);
  font-weight: 600;
}

.actions-group {
  display: flex;
  align-items: center;
  gap: 4px;
}

.btn-action-icon {
  padding: 4px 7px;
  font-size: 12px;
  border-radius: var(--radius-sm);
  border: 1px solid var(--border);
  background: var(--bg-surface);
  cursor: pointer;
  transition: background-color 0.12s ease;
}

.btn-action-icon:hover {
  background-color: var(--bg-hover);
}

.btn-action-danger:hover {
  background-color: var(--danger-sub);
  color: var(--danger);
  border-color: var(--danger);
}

/* Empty State */
.empty-state {
  text-align: center;
  padding: 60px 20px;
  background-color: var(--bg-surface);
  border: 1px dashed var(--border);
  border-radius: var(--radius-md);
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
}

.empty-icon {
  font-size: 48px;
}

.empty-text {
  font-size: 14px;
  color: var(--text-muted);
}

/* Modal */
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

.modal-content {
  background-color: var(--bg-surface);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  width: 90%;
  max-width: 480px;
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

.modal-header h3 {
  font-size: 14px;
  font-weight: 700;
  color: var(--text-ink);
}

.btn-close {
  background: transparent;
  border: none;
  font-size: 14px;
  color: var(--text-sub);
  cursor: pointer;
}

.modal-body {
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.form-label {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-muted);
}

.form-input, .form-textarea {
  padding: 8px 10px;
  border-radius: var(--radius-sm);
  border: 1px solid var(--border);
  background-color: var(--bg-surface);
  color: var(--text-ink);
  font-size: 13px;
  font-family: inherit;
}

.form-input:focus, .form-textarea:focus {
  border-color: var(--border-focus);
  outline: none;
}

.modal-actions {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 8px;
}
</style>
