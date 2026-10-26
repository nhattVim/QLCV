<template>
  <div class="workspaces-container">
    <!-- Toolbar -->
    <div class="workspaces-toolbar">
      <div class="toolbar-left">
        <h2 class="view-title">📁 Danh Sách Không Gian Làm Việc</h2>
        <span class="view-subtitle">
          Chọn không gian làm việc để cập nhật danh sách các ticket/công việc tương ứng
          <strong v-if="teamName">({{ teamName }})</strong>
        </span>
      </div>

      <div class="toolbar-right">
        <div class="search-box">
          <input 
            v-model="searchKeyword" 
            type="text" 
            placeholder="🔍 Tìm không gian..." 
            class="search-input"
          />
          <button v-if="searchKeyword" class="btn-clear" @click="searchKeyword = ''">✕</button>
        </div>

        <!-- Only Admin and Super Admin can create workspace -->
        <button 
          v-if="canManage" 
          class="btn-primary" 
          @click="openCreateModal"
        >
          ➕ Thêm Không Gian Mới
        </button>
      </div>
    </div>

    <!-- Workspaces Grid -->
    <div v-if="filteredWorkspaces.length > 0" class="workspaces-grid">
      <div 
        v-for="ws in filteredWorkspaces" 
        :key="ws.id" 
        class="ws-card"
        :class="{ 'ws-card-active': currentWorkspaceId === ws.id }"
        @click="selectWorkspace(ws)"
      >
        <!-- Card Top -->
        <div class="ws-card-top">
          <div class="ws-title-wrap">
            <span class="ws-folder-icon">📂</span>
            <div>
              <h3 class="ws-name">{{ ws.name }}</h3>
              <span class="ws-date">Tạo: {{ formatDate(ws.created_at) }}</span>
            </div>
          </div>

          <span v-if="currentWorkspaceId === ws.id" class="badge-active-ws">
            ● Đang mở
          </span>
        </div>

        <!-- Description -->
        <p class="ws-desc">
          {{ ws.description || 'Chưa có mô tả cho không gian làm việc này.' }}
        </p>

        <!-- Metrics -->
        <div class="ws-meta">
          <div class="ws-meta-item">
            <span class="ws-meta-val">{{ ws.task_count || 0 }}</span>
            <span class="ws-meta-lbl">📋 Công việc</span>
          </div>
          <div v-if="ws.created_by" class="ws-creator" :title="`Tạo bởi: ${ws.created_by}`">
            👤 {{ ws.created_by }}
          </div>
        </div>

        <!-- Actions -->
        <div class="ws-card-actions" @click.stop>
          <button 
            class="btn-open-ws"
            :class="{ 'btn-open-active': currentWorkspaceId === ws.id }"
            @click="selectWorkspace(ws)"
          >
            {{ currentWorkspaceId === ws.id ? '✓ Đang xem công việc' : '🚀 Mở không gian này ➔' }}
          </button>

          <!-- Only Admin or Super Admin can edit/delete workspace -->
          <div v-if="canManage" class="actions-group">
            <button class="btn-action-icon" title="Chỉnh sửa tên và mô tả không gian" @click="openEditModal(ws)">
              ✏️
            </button>
            <button class="btn-action-icon btn-action-danger" title="Xóa không gian này" @click="confirmDelete(ws)">
              🗑️
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Empty State -->
    <div v-else class="empty-state">
      <span class="empty-icon">📁</span>
      <p class="empty-text">Chưa có không gian làm việc nào phù hợp.</p>
      <button v-if="canManage" class="btn-primary" @click="openCreateModal">
        ➕ Tạo Không Gian Làm Việc Mới
      </button>
    </div>

    <!-- Modal Create / Edit Workspace (Admin Only) -->
    <div v-if="showModal && canManage" class="modal-backdrop" @click.self="showModal = false">
      <div class="modal-content">
        <div class="modal-header">
          <h3>{{ isEditing ? '✏️ Chỉnh Sửa Không Gian Làm Việc' : '➕ Tạo Không Gian Làm Việc Mới' }}</h3>
          <button class="btn-close" @click="showModal = false">✕</button>
        </div>

        <form class="modal-body" @submit.prevent="saveWorkspace">
          <div class="form-group">
            <label class="form-label">Tên không gian làm việc: <span class="text-danger">*</span></label>
            <input 
              v-model="form.name" 
              type="text" 
              class="form-input" 
              placeholder="Ví dụ: Tháng 9, Tháng 10, Dự Án Website..." 
              required
              autofocus
            />
          </div>

          <div class="form-group">
            <label class="form-label">Mô tả công việc trong không gian:</label>
            <textarea 
              v-model="form.description" 
              class="form-textarea" 
              rows="3" 
              placeholder="Ví dụ: Những việc làm trong tháng 9, các hạng mục chuẩn bị..."
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
  workspaces: {
    type: Array,
    default: () => [],
  },
  currentWorkspaceId: {
    type: String,
    default: '',
  },
  currentUser: {
    type: Object,
    required: true,
  },
  teamName: {
    type: String,
    default: '',
  },
});

const emit = defineEmits(['select-workspace', 'create-workspace', 'update-workspace', 'delete-workspace']);

const searchKeyword = ref('');
const showModal = ref(false);
const isEditing = ref(false);
const isSaving = ref(false);
const editingId = ref(null);

const form = ref({
  name: '',
  description: '',
});

// Chỉ có Admin hoặc Super Admin mới được sửa/xóa/tạo workspace
const canManage = computed(() => {
  return props.currentUser.role === 'admin' || props.currentUser.role === 'super_admin';
});

const filteredWorkspaces = computed(() => {
  const q = searchKeyword.value.trim().toLowerCase();
  if (!q) return props.workspaces;
  return props.workspaces.filter(w => 
    (w.name || '').toLowerCase().includes(q) || 
    (w.description || '').toLowerCase().includes(q)
  );
});

function selectWorkspace(ws) {
  emit('select-workspace', ws.id);
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

function openEditModal(ws) {
  isEditing.value = true;
  editingId.value = ws.id;
  form.value = {
    name: ws.name,
    description: ws.description || '',
  };
  showModal.value = true;
}

async function saveWorkspace() {
  if (!form.value.name.trim()) return;
  isSaving.value = true;
  try {
    if (isEditing.value) {
      await emit('update-workspace', {
        id: editingId.value,
        name: form.value.name.trim(),
        description: form.value.description.trim(),
      });
    } else {
      await emit('create-workspace', {
        name: form.value.name.trim(),
        description: form.value.description.trim(),
      });
    }
    showModal.value = false;
  } finally {
    isSaving.value = false;
  }
}

function confirmDelete(ws) {
  if (confirm(`Bạn có chắc chắn muốn xóa không gian làm việc "${ws.name}"? Tất cả các ticket thuộc không gian này cũng sẽ bị xóa vĩnh viễn!`)) {
    emit('delete-workspace', ws.id);
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
.workspaces-container {
  display: flex;
  flex-direction: column;
  gap: 16px;
  height: 100%;
}

.workspaces-toolbar {
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
.workspaces-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 12px;
}

.ws-card {
  background-color: var(--bg-card);
  border: 1px solid var(--border);
  border-left: 3.5px solid var(--border);
  border-radius: var(--radius-md);
  padding: 12px 14px;
  display: flex;
  flex-direction: column;
  gap: 9px;
  cursor: pointer;
  transition: border-color 0.12s ease;
  position: relative;
}

.ws-card:hover {
  border-color: var(--accent);
}

.ws-card-active {
  border-color: var(--accent);
  border-left-color: var(--accent);
  background-color: var(--bg-card);
}

.ws-card-top {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 8px;
}

.ws-title-wrap {
  display: flex;
  align-items: center;
  gap: 8px;
}

.ws-folder-icon {
  font-size: 18px;
  line-height: 1;
}

.ws-name {
  font-size: 14px;
  font-weight: 700;
  color: var(--text-ink);
  line-height: 1.3;
}

.ws-date {
  font-size: 11px;
  color: var(--text-sub);
}

.badge-active-ws {
  font-size: 11px;
  font-weight: 600;
  color: var(--accent);
  background-color: var(--accent-sub);
  border: 1px solid var(--accent);
  padding: 1px 7px;
  border-radius: var(--radius-sm);
}

.ws-desc {
  font-size: 12px;
  color: var(--text-muted);
  line-height: 1.45;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
  min-height: 32px;
}

.ws-meta {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 2px 0;
}

.ws-meta-item {
  display: flex;
  align-items: baseline;
  gap: 5px;
}

.ws-meta-val {
  font-size: 13px;
  font-weight: 700;
  color: var(--text-ink);
}

.ws-meta-lbl {
  font-size: 11px;
  color: var(--text-muted);
}

.ws-creator {
  font-size: 11px;
  color: var(--text-sub);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 140px;
}

.ws-card-actions {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  padding-top: 8px;
  border-top: 1px solid var(--border);
  margin-top: 2px;
}

.btn-open-ws {
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

.btn-open-ws:hover {
  background-color: var(--bg-hover);
  border-color: var(--border-focus);
}

.btn-open-active {
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
