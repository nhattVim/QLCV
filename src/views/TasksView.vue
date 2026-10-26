<template>
  <div class="tasks-container">
    <!-- Action & Filter Bar (Matching Python Toolbar) -->
    <div class="filter-toolbar">
      <div class="toolbar-left">
        <!-- Add Task Button (Admin or Super Admin) -->
        <button 
          v-if="canManage"
          class="btn-success"
          @click="$emit('add-task')"
        >
          ➕ Thêm Công Việc Mới
        </button>

        <!-- Status Filter -->
        <div class="filter-group">
          <label class="filter-label">Trạng thái:</label>
          <select v-model="filterStatus" class="filter-select">
            <option value="ALL">Tất cả</option>
            <option value="TODO">Chưa làm</option>
            <option value="IN_PROGRESS">Đang làm</option>
            <option value="REVIEW">Chờ duyệt</option>
            <option value="DONE">Hoàn thành</option>
          </select>
        </div>

        <!-- Assignee Filter (Admin only) -->
        <div v-if="canManage" class="filter-group">
          <label class="filter-label">Cán bộ:</label>
          <select v-model="filterUser" class="filter-select">
            <option value="ALL">Tất cả cán bộ</option>
            <option v-for="user in users" :key="user.email" :value="user.email">
              {{ user.full_name }}
            </option>
          </select>
        </div>

        <!-- Priority Filter -->
        <div class="filter-group">
          <label class="filter-label">Độ ưu tiên:</label>
          <select v-model="filterPriority" class="filter-select">
            <option value="ALL">Tất cả</option>
            <option value="Urgent">🔥 Khẩn cấp</option>
            <option value="High">⚡ Cao</option>
            <option value="Medium">🔹 Trung bình</option>
            <option value="Low">🟢 Thấp</option>
          </select>
        </div>

        <!-- Search Keyword -->
        <div class="search-box">
          <input 
            v-model="searchKeyword" 
            type="text" 
            placeholder="🔍 Tìm kiếm công việc, cán bộ..." 
            class="search-input"
          />
          <button v-if="searchKeyword" class="btn-clear-search" @click="searchKeyword = ''">✕</button>
        </div>
      </div>

      <div class="toolbar-right">
        <!-- View Mode: Kanban vs Table -->
        <div class="view-switch">
          <button 
            class="switch-btn" 
            :class="{ active: viewMode === 'kanban' }"
            title="Bố trí cột dọc kiểu Jira / Trello"
            @click="viewMode = 'kanban'"
          >
            📋 Cột Kanban
          </button>
          <button 
            class="switch-btn" 
            :class="{ active: viewMode === 'table' }"
            title="Dạng bảng danh sách"
            @click="viewMode = 'table'"
          >
            ☰ Danh sách bảng
          </button>
        </div>
      </div>
    </div>

    <!-- Active Filter Summary -->
    <div class="filter-summary">
      <div class="summary-left">
        <span>
          Tổng số: <strong>{{ filteredTasks.length }}</strong> công việc
          <span v-if="isFiltered" class="filter-active-tag">(đang lọc)</span>
        </span>
      </div>
      <button v-if="isFiltered" class="btn-reset" @click="resetFilters">
        Đặt lại bộ lọc
      </button>
    </div>

    <!-- KANBAN BOARD (Vertical Columns like Jira / Trello - Hỗ trợ kéo thả thẻ) -->
    <div v-if="viewMode === 'kanban'" class="kanban-board">
      <div 
        v-for="col in kanbanColumns" 
        :key="col.key" 
        class="kanban-col"
        :class="`col-${col.key.toLowerCase()}`"
      >
        <!-- Column Header (Clean: Title, Icon, Count Badge Only) -->
        <div class="kanban-col-header">
          <div class="col-title-wrap">
            <span class="col-icon">{{ col.icon }}</span>
            <span class="col-title">{{ col.label }}</span>
            <span class="col-count">{{ getColumnTasks(col.key).length }}</span>
          </div>
        </div>

        <!-- Vertical Stack of Compact Task Cards -->
        <div 
          class="kanban-col-body"
          :data-status="col.key"
        >
          <TaskCard 
            v-for="task in getColumnTasks(col.key)" 
            :key="task.id" 
            :task="task" 
            :user-map="userMap"
            :current-user="currentUser"
            @view="$emit('view-task', $event)"
            @edit="$emit('edit-task', $event)"
            @delete="$emit('delete-task', $event)"
            @approve="$emit('approve-task', $event)"
            @status-change="$emit('status-change', $event)"
          />

          <!-- Empty Column Notice -->
          <div class="kanban-empty">
            <span>Không có việc</span>
          </div>
        </div>
      </div>
    </div>

    <!-- TABLE VIEW (Alternative List) -->
    <div v-else class="table-container">
      <table class="tasks-table">
        <thead>
          <tr>
            <th style="width: 60px;">Mã</th>
            <th>Tên công việc</th>
            <th style="width: 120px;">Độ ưu tiên</th>
            <th style="width: 120px;">Trạng thái</th>
            <th style="width: 180px;">Cán bộ phụ trách</th>
            <th class="col-due">Hạn chót</th>
            <th class="col-actions">Thao tác</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="task in filteredTasks" :key="task.id">
            <td class="cell-id">#{{ task.id ? task.id.slice(-4) : '' }}</td>
            <td class="cell-title">
              <div class="tbl-title" @click="$emit('view-task', task)">
                {{ task.title }}
                <span v-if="task.attachments && task.attachments.length > 0" class="tbl-att-badge" :title="`${task.attachments.length} tệp đính kèm`">
                  📎 {{ task.attachments.length }}
                </span>
              </div>
              <div v-if="plainDescription(task.description)" class="tbl-desc">{{ plainDescription(task.description) }}</div>
            </td>
            <td>
              <span class="badge" :class="getPriorityBadgeClass(task.priority)">
                {{ getPriorityText(task.priority) }}
              </span>
            </td>
            <td>
              <span class="badge-status" :class="`badge-${task.status.toLowerCase()}`">
                ● {{ getStatusText(task.status) }}
              </span>
            </td>
            <td class="cell-assignees">
              {{ getAssigneesText(task.assigned_to) }}
            </td>
            <td class="cell-due">
              <span class="due-text" :class="{ 'text-danger': isOverdue(task) }">
                {{ formatDueDate(task.due_date) }}
              </span>
            </td>
            <td class="cell-actions">
              <div class="action-row">
                <template v-if="canManage">
                  <button
                    v-if="task.status === 'REVIEW'"
                    class="btn-success btn-icon-action"
                    title="Duyệt hoàn thành"
                    @click="$emit('approve-task', task.id)"
                  >✓</button>
                  <button
                    class="btn-secondary btn-icon-action"
                    title="Sửa công việc"
                    @click="$emit('edit-task', task)"
                  >Sửa</button>
                  <button
                    class="btn-danger btn-icon-action"
                    title="Xóa công việc"
                    @click="$emit('delete-task', task)"
                  >Xóa</button>
                </template>
                <template v-else>
                  <button
                    class="btn-secondary btn-icon-action"
                    title="Xem chi tiết"
                    @click="$emit('view-task', task)"
                  >Xem</button>
                </template>
              </div>
            </td>
          </tr>
          <tr v-if="filteredTasks.length === 0">
            <td colspan="7" class="cell-empty">
              Không có công việc nào phù hợp với bộ lọc hiện tại.
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- Generous Bottom Whitespace Spacer -->
    <div class="bottom-spacer"></div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted, watch, nextTick } from 'vue';
import Sortable from 'sortablejs';
import TaskCard from '../components/TaskCard.vue';

const props = defineProps({
  tasks: {
    type: Array,
    default: () => [],
  },
  users: {
    type: Array,
    default: () => [],
  },
  userMap: {
    type: Object,
    default: () => ({}),
  },
  currentUser: {
    type: Object,
    required: true,
  },
  workspaceName: {
    type: String,
    default: '',
  },
  teamName: {
    type: String,
    default: '',
  },
});

const emit = defineEmits(['add-task', 'view-task', 'edit-task', 'delete-task', 'approve-task', 'status-change']);

const canManage = computed(() => props.currentUser?.role === 'admin' || props.currentUser?.role === 'super_admin');

const filterStatus = ref('ALL');
const filterPriority = ref('ALL');
const filterUser = ref('ALL');
const searchKeyword = ref('');
const viewMode = ref('kanban'); // 'kanban' | 'table'

const kanbanColumns = [
  { key: 'TODO', label: 'Chưa làm', icon: '📋' },
  { key: 'IN_PROGRESS', label: 'Đang thực hiện', icon: '⚡' },
  { key: 'REVIEW', label: 'Chờ duyệt', icon: '🔍' },
  { key: 'DONE', label: 'Hoàn thành', icon: '✅' },
];

// SortableJS Drag & Drop Management
let sortableInstances = [];
let isDraggingCard = false;

function cleanupStrayFallbacks() {
  if (isDraggingCard) return;
  const fallbacks = document.querySelectorAll('.kanban-sortable-fallback');
  fallbacks.forEach(el => {
    try { el.remove(); } catch {}
  });
}

function destroySortables() {
  isDraggingCard = false;
  cleanupStrayFallbacks();
  sortableInstances.forEach(inst => {
    try { inst.destroy(); } catch {}
  });
  sortableInstances = [];
}

function initSortables() {
  if (viewMode.value !== 'kanban') return;
  if (isDraggingCard) return;

  destroySortables();

  const colBodies = document.querySelectorAll('.kanban-col-body');
  if (colBodies.length === 0) return;

  colBodies.forEach(el => {
    const sortable = new Sortable(el, {
      group: 'kanban-board',
      animation: 150,
      forceFallback: true,
      fallbackClass: 'kanban-sortable-fallback',
      fallbackOnBody: true,
      fallbackTolerance: 3,
      ghostClass: 'kanban-sortable-ghost',
      chosenClass: 'kanban-sortable-chosen',
      dragClass: 'kanban-sortable-drag',
      draggable: '.task-card',
      filter: '.admin-actions, .card-workflow, button, a',
      preventOnFilter: false,
      swapThreshold: 0.65,
      onStart() {
        isDraggingCard = true;
      },
      onEnd(evt) {
        isDraggingCard = false;
        setTimeout(() => {
          if (!isDraggingCard) {
            cleanupStrayFallbacks();
          }
        }, 80);

        const taskId = evt.item?.getAttribute('data-task-id');
        const fromStatus = evt.from?.getAttribute('data-status');
        const toStatus = evt.to?.getAttribute('data-status');

        if (!taskId) return;

        // Kéo thả sang cột khác -> đổi trạng thái
        if (fromStatus !== toStatus && toStatus) {
          if (toStatus === 'DONE') {
            if (canManage.value) {
              emit('approve-task', taskId);
            } else {
              // Cán bộ thường không được tự duyệt Hoàn thành, đưa vào REVIEW
              emit('status-change', { id: taskId, status: 'REVIEW' });
            }
          } else {
            // Các trạng thái khác: TODO, IN_PROGRESS, REVIEW
            emit('status-change', { id: taskId, status: toStatus });
          }
        }
      }
    });
    sortableInstances.push(sortable);
  });
}

function onWindowBlur() {
  isDraggingCard = false;
  cleanupStrayFallbacks();
}

onMounted(() => {
  nextTick(initSortables);
  window.addEventListener('blur', onWindowBlur);
});

onUnmounted(() => {
  destroySortables();
  window.removeEventListener('blur', onWindowBlur);
});

// Chỉ cần khởi tạo lại Sortable khi chuyển qua lại giữa chế độ Kanban và Table
watch(viewMode, (mode) => {
  if (mode === 'kanban') {
    nextTick(initSortables);
  } else {
    destroySortables();
  }
});

const isFiltered = computed(() => {
  return filterStatus.value !== 'ALL' || 
         filterPriority.value !== 'ALL' || 
         filterUser.value !== 'ALL' || 
         searchKeyword.value.trim() !== '';
});

const filteredTasks = computed(() => {
  return props.tasks.filter(t => {
    // Status Filter
    if (filterStatus.value !== 'ALL' && t.status !== filterStatus.value) {
      return false;
    }
    // Priority Filter
    if (filterPriority.value !== 'ALL' && t.priority !== filterPriority.value) {
      return false;
    }
    // Assignee Filter
    if (filterUser.value !== 'ALL') {
      if (!t.assigned_to || !t.assigned_to.includes(filterUser.value)) {
        return false;
      }
    }
    // Search Query
    if (searchKeyword.value.trim()) {
      const q = searchKeyword.value.trim().toLowerCase();
      const inTitle = t.title.toLowerCase().includes(q);
      const inDesc = (t.description || '').toLowerCase().includes(q);
      const inAssignee = (t.assigned_to || []).some(email => {
        const name = (props.userMap[email] || email).toLowerCase();
        return name.includes(q);
      });
      if (!inTitle && !inDesc && !inAssignee) {
        return false;
      }
    }
    return true;
  });
});

function getColumnTasks(statusKey) {
  return filteredTasks.value.filter(t => t.status === statusKey);
}

function resetFilters() {
  filterStatus.value = 'ALL';
  filterPriority.value = 'ALL';
  filterUser.value = 'ALL';
  searchKeyword.value = '';
}

function getPriorityText(prio) {
  switch (prio) {
    case 'Urgent': return 'Khẩn cấp';
    case 'High': return 'Cao';
    case 'Medium': return 'Vừa';
    default: return 'Thấp';
  }
}

function getPriorityBadgeClass(prio) {
  switch (prio) {
    case 'Urgent': return 'badge-urgent';
    case 'High': return 'badge-high';
    case 'Medium': return 'badge-medium';
    default: return 'badge-low';
  }
}

function getStatusText(st) {
  switch (st) {
    case 'TODO': return 'Chưa làm';
    case 'IN_PROGRESS': return 'Đang làm';
    case 'REVIEW': return 'Chờ duyệt';
    case 'DONE': return 'Hoàn thành';
    default: return st;
  }
}

function getAssigneesText(assignees) {
  if (!assignees || assignees.length === 0) return 'Chưa phân công';
  const hasUsers = Object.keys(props.userMap).length > 0;
  const list = hasUsers
    ? assignees.filter(e => Boolean(props.userMap[e])).map(e => props.userMap[e])
    : assignees.map(e => props.userMap[e] || e);
  return list.length > 0 ? list.join(', ') : 'Chưa phân công';
}

function plainDescription(html) {
  if (!html) return '';
  return html.replace(/<[^>]+>/g, '').replace(/&nbsp;/g, ' ').replace(/\s+/g, ' ').trim();
}

function formatDueDate(dueStr) {
  if (!dueStr) return '—';
  try {
    const d = new Date(dueStr);
    const pad = (n) => String(n).padStart(2, '0');
    return `${pad(d.getHours())}:${pad(d.getMinutes())} - ${pad(d.getDate())}/${pad(d.getMonth() + 1)}/${d.getFullYear()}`;
  } catch {
    return dueStr;
  }
}

function isOverdue(task) {
  if (!task.due_date || task.status === 'DONE') return false;
  return new Date(task.due_date) < new Date();
}
</script>

<style scoped>
.tasks-container {
  display: flex;
  flex-direction: column;
  gap: 12px;
  height: 100%;
}

.filter-toolbar {
  background-color: var(--bg-surface);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  padding: 8px 12px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  flex-wrap: wrap;
}

.toolbar-left {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
  flex: 1;
}

.toolbar-right {
  display: flex;
  align-items: center;
  gap: 8px;
}

.filter-group {
  display: flex;
  align-items: center;
  gap: 6px;
}

.filter-label {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-muted);
}

.filter-select {
  padding: 5px 8px;
  border-radius: var(--radius-sm);
  border: 1px solid var(--border);
  background-color: var(--bg-surface);
  color: var(--text-ink);
  font-size: 12.5px;
  font-weight: 500;
}

.search-box {
  position: relative;
  display: flex;
  align-items: center;
  min-width: 200px;
}

.search-input {
  width: 100%;
  padding: 5px 24px 5px 9px;
  border-radius: var(--radius-sm);
  border: 1px solid var(--border);
  background-color: var(--bg-surface);
  color: var(--text-ink);
  font-size: 12.5px;
}
.search-input:focus {
  border-color: var(--border-focus);
}

.btn-clear-search {
  position: absolute;
  right: 6px;
  font-size: 11px;
  color: var(--text-sub);
}

.view-switch {
  display: flex;
  align-items: center;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  overflow: hidden;
  background-color: var(--bg-sub);
}

.switch-btn {
  padding: 5px 10px;
  font-size: 12px;
  font-weight: 600;
  color: var(--text-muted);
  border: none;
  background: transparent;
}
.switch-btn.active {
  background-color: var(--bg-surface);
  color: var(--accent);
  font-weight: 700;
}

.filter-summary {
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: 12px;
  color: var(--text-sub);
  padding: 0 4px;
}

.summary-left {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
}

.badge-current-ws {
  background-color: var(--accent-sub);
  color: var(--accent);
  padding: 2px 8px;
  border-radius: var(--radius-sm);
  border: 1px solid var(--accent);
  font-size: 12px;
}

.badge-team-name {
  font-weight: 400;
  opacity: 0.85;
  margin-left: 3px;
}

.filter-active-tag {
  color: var(--warning);
  font-weight: 600;
  margin-left: 4px;
}

.btn-reset {
  color: var(--accent);
  text-decoration: underline;
  cursor: pointer;
  font-size: 12px;
}

/* ================= JIRA / TRELLO KANBAN BOARD ================= */
.kanban-board {
  display: grid;
  grid-template-columns: repeat(4, minmax(240px, 1fr));
  gap: 12px;
  flex: 1;
  min-height: 480px;
  align-items: stretch;
}

.kanban-col {
  background-color: var(--bg-sub);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  border-top-width: 3px;
  transition: background-color 0.15s ease, border-color 0.15s ease, transform 0.15s ease, box-shadow 0.15s ease;
}


.col-todo {
  border-top-color: var(--todo);
}
.col-in_progress {
  border-top-color: var(--inprogress);
}
.col-review {
  border-top-color: var(--review);
}
.col-done {
  border-top-color: var(--done);
}

.kanban-col-header {
  padding: 8px 12px;
  border-bottom: 1px solid var(--border);
  display: flex;
  align-items: center;
  justify-content: space-between;
  background-color: var(--bg-surface);
}

.col-title-wrap {
  display: flex;
  align-items: center;
  gap: 6px;
}

.col-icon {
  font-size: 13px;
}

.col-title {
  font-size: 13px;
  font-weight: 700;
  color: var(--text-ink);
}

.col-count {
  font-size: 11px;
  font-weight: 700;
  padding: 1px 6px;
  border-radius: var(--radius-pill);
  background-color: var(--bg-sub);
  color: var(--text-muted);
  border: 1px solid var(--border);
}

.kanban-col-body {
  position: relative;
  padding: 8px;
  display: flex;
  flex-direction: column;
  gap: 8px;
  overflow-y: auto;
  flex: 1;
  min-height: 250px;
  max-height: calc(100vh - 310px);
}

.kanban-empty {
  display: none;
  position: absolute;
  top: 8px;
  left: 8px;
  right: 8px;
  padding: 24px 12px;
  text-align: center;
  border: 1px dashed var(--border);
  border-radius: var(--radius-sm);
  color: var(--text-sub);
  font-size: 12px;
  pointer-events: none;
}

/* Hiện ô "Không có việc" khi trong cột không có bất kỳ thẻ nào (.task-card) và không có bóng mờ (.kanban-sortable-ghost) */
.kanban-col-body:not(:has(.task-card)):not(:has(.kanban-sortable-ghost)) .kanban-empty {
  display: block !important;
}

/* Ẩn ô "Không có việc" khi trong cột có thẻ hoặc có bóng mờ */
.kanban-col-body:has(.task-card) .kanban-empty,
.kanban-col-body:has(.kanban-sortable-ghost) .kanban-empty {
  display: none !important;
}

/* ================= TABLE VIEW ================= */
.table-container {
  background-color: var(--bg-surface);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  overflow-x: auto;
}

.tasks-table {
  width: 100%;
  min-width: 960px;
  table-layout: fixed;
  border-collapse: collapse;
  font-size: 12.5px;
  text-align: left;
}

.col-due {
  width: 108px;
}
.col-actions {
  width: 148px;
  text-align: right;
}

.tasks-table th {
  background-color: var(--bg-sub);
  color: var(--text-muted);
  font-weight: 700;
  font-size: 11px;
  text-transform: uppercase;
  padding: 9px 12px;
  border-bottom: 1px solid var(--border);
}

.tasks-table td {
  padding: 9px 12px;
  border-bottom: 1px solid var(--border-sub);
  color: var(--text-ink);
  vertical-align: middle;
}

.tasks-table tr:hover td {
  background-color: var(--bg-hover);
}

.cell-id {
  font-family: monospace;
  color: var(--text-sub);
  font-size: 11px;
}

.tbl-title {
  font-weight: 600;
  cursor: pointer;
}
.tbl-title:hover {
  color: var(--accent);
  text-decoration: underline;
}

.tbl-att-badge {
  display: inline-flex;
  align-items: center;
  margin-left: 6px;
  font-size: 11px;
  font-weight: 700;
  color: var(--accent);
  background-color: var(--accent-sub);
  padding: 1px 6px;
  border-radius: var(--radius-sm);
  vertical-align: middle;
}

.tbl-desc {
  font-size: 11.5px;
  color: var(--text-sub);
  max-width: 320px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.cell-assignees {
  font-weight: 500;
  color: var(--accent);
}

.cell-due {
  white-space: nowrap;
}

.due-text {
  display: inline-block;
  white-space: nowrap;
  font-variant-numeric: tabular-nums;
}

.cell-actions {
  white-space: nowrap;
  text-align: right;
  width: 148px;
}

.action-row {
  display: inline-flex;
  flex-wrap: nowrap;
  align-items: center;
  justify-content: flex-end;
  gap: 6px;
}

.btn-icon-action {
  padding: 4px 8px;
  font-size: 11px;
  line-height: 1;
  border-radius: var(--radius-sm);
  flex-shrink: 0;
  white-space: nowrap;
}

.cell-empty {
  text-align: center;
  padding: 30px;
  color: var(--text-sub);
}

.bottom-spacer {
  height: 36px;
  flex-shrink: 0;
}
</style>
