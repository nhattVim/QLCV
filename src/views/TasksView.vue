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
            <option v-if="enableReview" value="REVIEW">Chờ duyệt</option>
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
    <div 
      v-if="viewMode === 'kanban'" 
      class="kanban-board"
      :style="{ '--col-count': kanbanColumns.length }"
    >
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
            :enable-review="enableReview"
            :enable-result="enableResult"
            @view="$emit('view-task', $event)"
            @edit="$emit('edit-task', $event)"
            @delete="$emit('delete-task', $event)"
            @approve="$emit('approve-task', $event)"
            @status-change="$emit('status-change', $event)"
            @request-complete="handleRequestComplete"
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

    <!-- Complete Task Modal with Result Input -->
    <div v-if="completingTask" class="complete-modal-backdrop" @click.self="cancelComplete">
      <div class="complete-modal-card">
        <div class="complete-head">
          <div class="complete-icon-wrap">🎉</div>
          <div>
            <h3 class="complete-title">Xác nhận Hoàn thành Công việc</h3>
            <span class="complete-sub">Ghi nhận kết quả thực hiện cho nhiệm vụ này</span>
          </div>
          <button class="btn-close-modal" @click="cancelComplete">✕</button>
        </div>

        <div class="complete-body">
          <div class="complete-task-name">
            <strong>Nhiệm vụ:</strong> {{ completingTask.title }}
          </div>

          <div class="form-group">
            <label class="complete-label">
              Kết quả thực hiện / Sản phẩm bàn giao:
              <span v-if="enableResult" class="required-star text-danger">* (Bắt buộc)</span>
              <span v-else class="optional-tag">(Tùy chọn)</span>
            </label>
            <textarea 
              v-model="completionResult"
              class="complete-textarea"
              rows="4"
              placeholder="Ví dụ: Đã hoàn tất biên soạn tài liệu kỹ thuật, nghiệm thu và bàn giao cho khách hàng thành công..."
              autofocus
            ></textarea>
          </div>

          <!-- Result File Attachments (Tệp đính kèm khi hoàn thành) -->
          <div class="form-group att-group">
            <div class="att-header">
              <div class="att-title-wrap">
                <span class="att-icon">📎</span>
                <span class="att-title">Tệp & Tài liệu đính kèm ({{ completingAttachments.length }})</span>
                <span v-if="isUploadingResult" class="uploading-badge">
                  ⏳ Đang tải lên...
                </span>
              </div>

              <div class="att-actions">
                <button 
                  type="button" 
                  class="btn-attach" 
                  :disabled="isUploadingResult"
                  title="Chọn tệp từ máy tính để đính kèm"
                  @click="triggerResultFileInput"
                >
                  <span v-if="isUploadingResult">⏳ Đang tải...</span>
                  <span v-else>➕ Thêm tệp</span>
                </button>
                <input 
                  ref="resultFileInputRef" 
                  type="file" 
                  multiple 
                  class="hidden-file-input" 
                  @change="handleResultFileUpload" 
                />
              </div>
            </div>

            <!-- Dropzone when no files attached -->
            <div 
              v-if="completingAttachments.length === 0" 
              class="att-dropzone"
              :class="{ 'dragging': isDraggingResultDrop, 'uploading': isUploadingResult }"
              @dragover.prevent="isDraggingResultDrop = true"
              @dragleave.prevent="isDraggingResultDrop = false"
              @drop.prevent="handleResultDrop"
              @click="triggerResultFileInput"
            >
              <div class="dropzone-content">
                <span class="dropzone-icon">📁</span>
                <span class="dropzone-text">
                  {{ isUploadingResult ? 'Đang tải tệp lên Cloudflare R2...' : 'Kéo & thả tệp vào đây hoặc bấm để chọn tệp' }}
                </span>
                <span class="dropzone-sub">(Hỗ trợ PDF, Word, Excel, Hình ảnh, Zip...)</span>
              </div>
            </div>

            <!-- List of uploaded result attachments -->
            <div v-else class="attachments-list">
              <div 
                v-for="att in completingAttachments" 
                :key="att.id" 
                class="att-item"
              >
                <div class="att-type-icon" :class="getFileTypeClass(att.file_name, att.content_type)">
                  {{ getFileEmoji(att.file_name, att.content_type) }}
                </div>
                <div class="att-details">
                  <span class="att-name" :title="att.file_name">{{ att.file_name }}</span>
                  <div class="att-meta">
                    <span class="att-size">{{ formatFileSize(att.file_size) }}</span>
                  </div>
                </div>
                <div class="att-btns">
                  <button 
                    type="button" 
                    class="btn-att-action btn-att-del" 
                    title="Xóa tệp này"
                    @click.stop="removeCompletingAttachment(att)"
                  >✕</button>
                </div>
              </div>
            </div>
          </div>
        </div>

        <div class="complete-foot">
          <button type="button" class="btn-cancel-modal" @click="cancelComplete">
            Hủy
          </button>
          <button type="button" class="btn-submit-done" @click="submitComplete">
            ✓ Hoàn thành Công việc
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted, watch, nextTick } from 'vue';
import Sortable from 'sortablejs';
import TaskCard from '../components/TaskCard.vue';
import { api } from '../api.js';

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
  currentTeam: {
    type: Object,
    default: null,
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
const enableReview = computed(() => props.currentTeam?.enable_review_step !== false);
const enableResult = computed(() => props.currentTeam?.enable_result_column !== false);

const filterStatus = ref('ALL');
const filterPriority = ref('ALL');
const filterUser = ref('ALL');
const searchKeyword = ref('');
const viewMode = ref('kanban'); // 'kanban' | 'table'

// Complete Modal state for recording result and deliverables
const completingTask = ref(null);
const completionResult = ref('');
const completingAttachments = ref([]);
const isUploadingResult = ref(false);
const resultFileInputRef = ref(null);
const isDraggingResultDrop = ref(false);

function triggerResultFileInput() {
  resultFileInputRef.value?.click();
}

function formatFileSize(bytes) {
  if (!bytes) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
}

function getFileTypeClass(fileName = '', mime = '') {
  const ext = fileName.split('.').pop().toLowerCase();
  if (['pdf'].includes(ext)) return 'file-pdf';
  if (['doc', 'docx'].includes(ext)) return 'file-word';
  if (['xls', 'xlsx', 'csv'].includes(ext)) return 'file-excel';
  if (['ppt', 'pptx'].includes(ext)) return 'file-ppt';
  if (['png', 'jpg', 'jpeg', 'gif', 'webp', 'svg'].includes(ext) || (mime && mime.startsWith('image/'))) return 'file-image';
  if (['zip', 'rar', '7z', 'tar', 'gz'].includes(ext)) return 'file-zip';
  return 'file-generic';
}

function getFileEmoji(fileName = '', mime = '') {
  const ext = fileName.split('.').pop().toLowerCase();
  if (['pdf'].includes(ext)) return '📄';
  if (['doc', 'docx'].includes(ext)) return '📝';
  if (['xls', 'xlsx', 'csv'].includes(ext)) return '📊';
  if (['ppt', 'pptx'].includes(ext)) return '📑';
  if (['png', 'jpg', 'jpeg', 'gif', 'webp', 'svg'].includes(ext) || (mime && mime.startsWith('image/'))) return '🖼️';
  if (['zip', 'rar', '7z', 'tar', 'gz'].includes(ext)) return '📦';
  return '📎';
}

async function processResultFiles(files) {
  if (!files || files.length === 0 || !completingTask.value) return;
  isUploadingResult.value = true;
  try {
    const teamId = completingTask.value.team_id || props.currentTeam?.id || null;
    const workspaceId = completingTask.value.workspace_id || null;
    const taskId = completingTask.value.id;

    for (let i = 0; i < files.length; i++) {
      const file = files[i];
      const buffer = await file.arrayBuffer();
      const newAtt = await api.uploadAttachment(
        teamId,
        workspaceId,
        taskId,
        file.name,
        buffer,
        file.type,
        props.currentUser.email,
        'result'
      );
      completingAttachments.value.push(newAtt);
      if (!completingTask.value.attachments) {
        completingTask.value.attachments = [];
      }
      completingTask.value.attachments.push(newAtt);
    }
  } catch (err) {
    alert('Lỗi tải tệp kết quả: ' + (err.message || err));
  } finally {
    isUploadingResult.value = false;
    if (resultFileInputRef.value) {
      resultFileInputRef.value.value = '';
    }
  }
}

async function handleResultFileUpload(e) {
  const files = e.target.files;
  await processResultFiles(files);
}

async function handleResultDrop(e) {
  isDraggingResultDrop.value = false;
  const files = e.dataTransfer?.files;
  await processResultFiles(files);
}

async function removeCompletingAttachment(att) {
  if (!completingTask.value) return;
  try {
    await api.deleteAttachment(completingTask.value.id, att.id, att.object_key);
    completingAttachments.value = completingAttachments.value.filter(a => a.id !== att.id);
    if (completingTask.value.attachments) {
      completingTask.value.attachments = completingTask.value.attachments.filter(a => a.id !== att.id);
    }
  } catch (err) {
    alert('Lỗi xóa tệp: ' + (err.message || err));
  }
}

function handleRequestComplete(task) {
  completingTask.value = task;
  completionResult.value = task.result || '';
  completingAttachments.value = (task.attachments || []).filter(a => a.category === 'result');
}

function submitComplete() {
  if (!completingTask.value) return;
  const res = completionResult.value.trim();
  if (enableResult.value && !res) {
    alert('Vui lòng nhập kết quả thực hiện trước khi xác nhận hoàn thành công việc!');
    return;
  }
  emit('status-change', {
    id: completingTask.value.id,
    status: 'DONE',
    result: res,
  });
  completingTask.value = null;
  completionResult.value = '';
  completingAttachments.value = [];
}

function cancelComplete() {
  if (completingTask.value) {
    const taskId = completingTask.value.id;
    // Khi hủy nhập kết quả, bắt buộc trả ticket về trạng thái Đang làm theo yêu cầu
    emit('status-change', { id: taskId, status: 'IN_PROGRESS' });
  }
  completingTask.value = null;
  completionResult.value = '';
  completingAttachments.value = [];
  destroySortables();
  nextTick(() => {
    initSortables();
  });
}

// Dynamic Kanban columns: 3 columns if team disabled REVIEW, 4 columns if enabled
const kanbanColumns = computed(() => {
  const cols = [
    { key: 'TODO', label: 'Chưa làm', icon: '📋' },
    { key: 'IN_PROGRESS', label: 'Đang thực hiện', icon: '⚡' },
  ];
  if (enableReview.value) {
    cols.push({ key: 'REVIEW', label: 'Chờ duyệt', icon: '🔍' });
  }
  cols.push({ key: 'DONE', label: 'Hoàn thành', icon: '✅' });
  return cols;
});

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
            const taskObj = props.tasks.find(t => t.id === taskId);
            if (enableReview.value && !canManage.value) {
              // Cán bộ thường không được tự duyệt Hoàn thành khi quy trình có bước duyệt -> đưa vào REVIEW
              emit('status-change', { id: taskId, status: 'REVIEW' });
            } else if (taskObj) {
              // Hoàn nguyên ngay lập tức thẻ về vị trí ban đầu để không bị nhảy sang cột Hoàn thành trước khi xác nhận kết quả
              if (evt.from && evt.item) {
                evt.from.insertBefore(evt.item, evt.from.children[evt.oldIndex] || null);
              }
              handleRequestComplete(taskObj);
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

watch(kanbanColumns, () => {
  if (viewMode.value === 'kanban') {
    nextTick(initSortables);
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
  gap: 10px;
  height: 100%;
  flex: 1;
  min-height: 0;
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
  grid-template-columns: repeat(var(--col-count, 4), minmax(280px, 1fr));
  gap: 12px;
  flex: 1;
  min-height: 0;
  height: 100%;
  align-items: stretch;
  overflow-x: auto;
  padding-bottom: 2px;
}

.kanban-col {
  background-color: var(--bg-sub);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  border-top-width: 4px;
  transition: background-color 0.15s ease, border-color 0.15s ease, transform 0.15s ease, box-shadow 0.15s ease;
  min-width: 270px;
  height: 100%;
  min-height: 0;
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
  padding: 8px 8px 24px 8px;
  display: flex;
  flex-direction: column;
  gap: 8px;
  overflow-y: auto;
  flex: 1;
  min-height: 0;
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

/* ================= COMPLETE TASK MODAL ================= */
.complete-modal-backdrop {
  position: fixed;
  inset: 0;
  background-color: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 999;
}

.complete-modal-card {
  background-color: var(--bg-surface);
  border-radius: var(--radius-md);
  box-shadow: 0 10px 25px rgba(0, 0, 0, 0.25);
  width: min(540px, 94vw);
  border: 1px solid var(--border);
  overflow: hidden;
  animation: popIn 0.18s ease-out;
}

@keyframes popIn {
  from { transform: scale(0.95); opacity: 0; }
  to { transform: scale(1); opacity: 1; }
}

.complete-head {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 16px 20px;
  border-bottom: 1px solid var(--border);
}

.complete-icon-wrap {
  font-size: 26px;
}

.complete-title {
  font-size: 16px;
  font-weight: 700;
  color: var(--text-ink);
  margin-bottom: 2px;
}

.complete-sub {
  font-size: 12.5px;
  color: var(--text-sub);
}

.btn-close-modal {
  margin-left: auto;
  background: none;
  border: none;
  font-size: 16px;
  color: var(--text-sub);
  cursor: pointer;
}

.complete-body {
  padding: 18px 20px;
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.complete-task-name {
  font-size: 13.5px;
  color: var(--text-ink);
  padding: 10px 14px;
  background-color: var(--bg-sub);
  border-radius: var(--radius-sm);
  border: 1px solid var(--border);
}

.complete-label {
  display: block;
  font-size: 13px;
  font-weight: 600;
  color: var(--text-ink);
  margin-bottom: 6px;
}

.optional-tag {
  font-size: 11.5px;
  font-weight: 400;
  color: var(--text-sub);
  margin-left: 4px;
}

.complete-textarea {
  width: 100%;
  padding: 10px 12px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background-color: var(--bg-surface);
  color: var(--text-ink);
  font-size: 13.5px;
  resize: vertical;
  line-height: 1.5;
}

.complete-textarea:focus {
  border-color: #10b981;
  outline: none;
}

/* Result Attachments Section inside Complete Modal (Matches TaskModal create/edit layout) */
.hidden-file-input {
  display: none !important;
}

.att-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 8px;
}

.att-title-wrap {
  display: flex;
  align-items: center;
  gap: 6px;
}

.att-icon {
  font-size: 14px;
}

.att-title {
  font-size: 13px;
  font-weight: 700;
  color: var(--text-ink);
}

.uploading-badge {
  font-size: 11px;
  color: var(--accent);
  margin-left: 6px;
}

.att-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.btn-attach {
  font-size: 12px;
  font-weight: 600;
  color: var(--accent);
  background-color: var(--accent-sub);
  border: 1px solid var(--accent);
  border-radius: var(--radius-sm);
  padding: 4px 10px;
  cursor: pointer;
  transition: all 0.15s ease;
}
.btn-attach:hover {
  background-color: var(--accent);
  color: #fff;
}
.btn-attach:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.att-dropzone {
  border: 1px dashed var(--border);
  border-radius: var(--radius-md);
  padding: 14px 16px;
  text-align: center;
  background-color: var(--bg-surface);
  cursor: pointer;
  transition: all 0.18s ease;
}
.att-dropzone:hover, .att-dropzone.dragging {
  border-color: var(--accent);
  background-color: var(--accent-sub);
}
.att-dropzone.uploading {
  cursor: wait;
  opacity: 0.8;
}

.dropzone-content {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  flex-wrap: wrap;
}
.dropzone-icon {
  font-size: 18px;
}
.dropzone-text {
  font-size: 12.5px;
  font-weight: 600;
  color: var(--text-ink);
}
.dropzone-sub {
  font-size: 11px;
  color: var(--text-sub);
}

.attachments-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
  max-height: 180px;
  overflow-y: auto;
  padding-right: 2px;
  scrollbar-width: thin;
}

.att-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 10px;
  background-color: var(--bg-surface);
  border: 1px solid var(--border-sub);
  border-radius: var(--radius-md);
  transition: all 0.15s ease;
}
.att-item:hover {
  border-color: var(--border);
  box-shadow: 0 1px 4px rgba(0,0,0,0.04);
}

.att-type-icon {
  width: 32px;
  height: 32px;
  border-radius: var(--radius-sm);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 16px;
  flex-shrink: 0;
  background-color: var(--bg-hover);
}
.att-type-icon.file-pdf { background-color: #fef2f2; }
.att-type-icon.file-word { background-color: #eff6ff; }
.att-type-icon.file-excel { background-color: #f0fdf4; }
.att-type-icon.file-image { background-color: #faf5ff; }
.att-type-icon.file-zip { background-color: #fffbeb; }

[data-theme="dark"] .att-type-icon.file-pdf { background-color: rgba(239, 68, 68, 0.2); }
[data-theme="dark"] .att-type-icon.file-word { background-color: rgba(59, 130, 246, 0.2); }
[data-theme="dark"] .att-type-icon.file-excel { background-color: rgba(16, 185, 129, 0.2); }
[data-theme="dark"] .att-type-icon.file-image { background-color: rgba(168, 85, 247, 0.2); }
[data-theme="dark"] .att-type-icon.file-zip { background-color: rgba(245, 158, 11, 0.2); }

.att-details {
  flex: 1;
  min-width: 0;
}
.att-name {
  display: block;
  font-size: 12.5px;
  font-weight: 600;
  color: var(--text-ink);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.att-meta {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 11px;
  color: var(--text-sub);
  margin-top: 2px;
}

.att-btns {
  display: flex;
  align-items: center;
  gap: 5px;
  flex-shrink: 0;
}

.btn-att-action {
  font-size: 11.5px;
  padding: 3px 8px;
  border-radius: var(--radius-sm);
  border: 1px solid var(--border);
  background: var(--bg-surface);
  color: var(--text-muted);
  cursor: pointer;
  transition: all 0.12s ease;
}
.btn-att-del {
  color: var(--danger);
  border-color: transparent;
  padding: 3px 6px;
}
.btn-att-del:hover {
  background: var(--danger-sub);
  border-color: var(--danger);
}

.complete-foot {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 10px;
  padding: 14px 20px;
  border-top: 1px solid var(--border);
  background-color: var(--bg-sub);
}

.btn-cancel-modal {
  padding: 8px 16px;
  border-radius: var(--radius-sm);
  border: 1px solid var(--border);
  background-color: var(--bg-surface);
  color: var(--text-ink);
  font-size: 13px;
  cursor: pointer;
}

.btn-submit-done {
  padding: 8px 20px;
  border-radius: var(--radius-sm);
  border: none;
  background-color: #16a34a;
  color: white;
  font-size: 13.5px;
  font-weight: 700;
  cursor: pointer;
  box-shadow: 0 2px 4px rgba(22, 163, 74, 0.25);
  transition: background-color 0.15s;
}

.btn-submit-done:hover {
  background-color: #15803d;
}
</style>
