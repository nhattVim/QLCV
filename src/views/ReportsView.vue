<template>
  <div class="reports-page">
    <!-- Compact Action & Filter Bar (Đã xóa filter không gian) -->
    <div class="filter-toolbar">
      <div class="toolbar-left">
        <!-- Status Filter -->
        <div class="filter-group">
          <label class="filter-label">Trạng thái:</label>
          <select v-model="filterStatus" class="filter-select">
            <option value="ALL">Tất cả</option>
            <option value="DONE">✅ Hoàn thành</option>
            <option v-if="showReviewFilter" value="REVIEW">🔍 Chờ duyệt</option>
            <option value="IN_PROGRESS">⚡ Đang làm</option>
            <option value="TODO">📋 Chưa làm</option>
          </select>
        </div>

        <!-- Staff Assignee Filter -->
        <div class="filter-group">
          <label class="filter-label">Cán bộ:</label>
          <select v-model="filterUser" class="filter-select">
            <option value="ALL">Tất cả cán bộ</option>
            <option v-for="u in assignableUsers" :key="u.email" :value="u.email">
              {{ u.full_name }}
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
            placeholder="🔍 Tìm kiếm nhiệm vụ, cán bộ..." 
            class="search-input"
          />
          <button v-if="searchKeyword" class="btn-clear-search" @click="searchKeyword = ''">✕</button>
        </div>
      </div>

      <div class="toolbar-right">
        <!-- Quick Reset Filter if Active -->
        <button v-if="isFiltered" class="btn-reset-filters" @click="resetFilters">
          ✕ Đặt lại lọc
        </button>

        <!-- Export to Excel Button (Hover chỉ đổi màu, không nổi lên) -->
        <button 
          class="btn-excel" 
          title="Xuất bảng báo cáo này thành tệp Microsoft Excel (.xlsx)"
          :disabled="isExporting"
          @click="exportToExcel"
        >
          <span class="excel-icon">📊</span>
          <span>{{ isExporting ? 'Đang xuất...' : 'Xuất File Excel' }}</span>
        </button>
      </div>
    </div>

    <!-- DOCUMENT-STYLE REPORT TABLE (Matching Attached Formal Layout) -->
    <div class="table-card">
      <div class="table-scroll-wrap">
        <table class="report-table">
          <thead>
            <tr>
              <th class="th-stt">STT</th>
              <th class="th-task">NHIỆM VỤ</th>
              <th class="th-staff">CÁN BỘ THỰC HIỆN</th>
              <th class="th-time">THỜI GIAN HOÀN THÀNH</th>
              <th class="th-result">KẾT QUẢ</th>
              <th class="th-notes">GHI CHÚ</th>
            </tr>
          </thead>
          <tbody>
            <tr 
              v-for="(task, index) in filteredTasks" 
              :key="task.id"
              class="report-row"
              @click="$emit('view-task', task)"
            >
              <!-- 1. STT -->
              <td class="td-stt">{{ index + 1 }}</td>

              <!-- 2. NHIỆM VỤ (Chỉ hiện nội dung) -->
              <td class="td-task">
                <span class="task-title-text">{{ task.title }}</span>
              </td>

              <!-- 3. CÁN BỘ THỰC HIỆN (Chỉ hiện tên không có icon) -->
              <td class="td-staff">
                <span class="staff-names-text">{{ getStaffNamesText(task) }}</span>
              </td>

              <!-- 4. THỜI GIAN HOÀN THÀNH (Thời gian hoàn thành thực tế, không lấy deadline, chưa hoàn thành để trống) -->
              <td class="td-time">
                <span class="time-done-text">{{ formatCompletionDate(task) }}</span>
              </td>

              <!-- 5. KẾT QUẢ (Chỉ hiển thị kết quả nếu hoàn thành, không hiện nút/ô nhập, để trống nếu chưa có) -->
              <td class="td-result">
                <span v-if="task.status === 'DONE' && task.result" class="result-text">{{ task.result }}</span>
              </td>

              <!-- 6. GHI CHÚ (Quản trị tự điền, để trống nếu chưa có) -->
              <td class="td-notes" @click.stop>
                <input 
                  v-if="canManage"
                  v-model="task.note"
                  type="text" 
                  class="note-inline-input"
                  placeholder="Ghi chú..."
                  @blur="saveTaskNote(task)"
                  @keydown.enter="$event.target.blur()"
                />
                <span v-else class="note-readonly">{{ task.note || '' }}</span>
              </td>
            </tr>

            <!-- Empty Row if No Tasks Match -->
            <tr v-if="filteredTasks.length === 0">
              <td colspan="6" class="td-empty">
                <div class="empty-wrap">
                  <span class="empty-icon">📂</span>
                  <p>Không có công việc nào phù hợp với bộ lọc hiện tại.</p>
                </div>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed } from 'vue';
import * as XLSX from 'xlsx';
import { openPath } from '@tauri-apps/plugin-opener';
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
    default: null,
  },
  teamName: {
    type: String,
    default: '',
  },
  currentTeam: {
    type: Object,
    default: null,
  },
  workspaceName: {
    type: String,
    default: '',
  },
  workspaces: {
    type: Array,
    default: () => [],
  },
});

const emit = defineEmits(['view-task', 'task-updated']);

const canManage = computed(() => {
  if (!props.currentUser) return true;
  return props.currentUser.role === 'admin' || props.currentUser.role === 'super_admin';
});

// Filters (Đã xóa filter không gian theo yêu cầu)
const filterStatus = ref('ALL');
const filterUser = ref('ALL');
const filterPriority = ref('ALL');
const searchKeyword = ref('');

// Export state
const isExporting = ref(false);

const assignableUsers = computed(() => {
  return (props.users || []).filter(u => u.role !== 'super_admin');
});

const showReviewFilter = computed(() => {
  return props.currentTeam?.enable_review_step !== false;
});

const isFiltered = computed(() => {
  return (
    filterStatus.value !== 'ALL' ||
    filterUser.value !== 'ALL' ||
    filterPriority.value !== 'ALL' ||
    Boolean(searchKeyword.value.trim())
  );
});

function resetFilters() {
  filterStatus.value = 'ALL';
  filterUser.value = 'ALL';
  filterPriority.value = 'ALL';
  searchKeyword.value = '';
}

// Filtered Tasks (Sau khi filter trên app, không filter thì lấy tất cả)
const filteredTasks = computed(() => {
  return (props.tasks || []).filter(task => {
    // 1. Status
    if (filterStatus.value !== 'ALL') {
      if (task.status !== filterStatus.value) return false;
    }
    // 2. User
    if (filterUser.value !== 'ALL') {
      const assigned = task.assigned_to || [];
      if (!assigned.includes(filterUser.value)) return false;
    }
    // 3. Priority
    if (filterPriority.value !== 'ALL') {
      if (task.priority !== filterPriority.value) return false;
    }
    // 4. Search
    if (searchKeyword.value.trim()) {
      const q = searchKeyword.value.trim().toLowerCase();
      const matchTitle = (task.title || '').toLowerCase().includes(q);
      const matchDesc = (task.description || '').toLowerCase().includes(q);
      const matchResult = (task.result || '').toLowerCase().includes(q);
      const matchStaff = (task.assigned_to || []).some(email => {
        const name = getStaffName(email).toLowerCase();
        return name.includes(q) || email.toLowerCase().includes(q);
      });
      if (!matchTitle && !matchDesc && !matchResult && !matchStaff) return false;
    }
    return true;
  });
});

function getStaffName(email) {
  if (props.userMap && props.userMap[email]) {
    return props.userMap[email];
  }
  const u = props.users.find(user => user.email === email);
  return u ? u.full_name : email;
}

// Format comma-separated staff names WITHOUT any icons/emojis
function getStaffNamesText(task) {
  if (!task.assigned_to || task.assigned_to.length === 0) return '';
  return task.assigned_to.map(email => getStaffName(email)).join(', ');
}

// Format completion date as DD/MM/YYYY only if DONE and has completed_at
// KHÔNG dùng due_date (deadline của ticket) làm ngày hoàn thành!
function formatCompletionDate(task) {
  if (task.status !== 'DONE' || !task.completed_at) return '';
  try {
    const d = new Date(task.completed_at);
    if (isNaN(d.getTime())) return '';
    const pad = n => String(n).padStart(2, '0');
    return `${pad(d.getDate())}/${pad(d.getMonth() + 1)}/${d.getFullYear()}`;
  } catch {
    return '';
  }
}

// Admin inline note save
async function saveTaskNote(task) {
  if (!task.id) return;
  try {
    await api.updateTask(task.id, {
      ...task,
      note: task.note ? task.note.trim() : null,
    });
  } catch (err) {
    console.error('Lỗi lưu ghi chú:', err);
  }
}

// ==========================================
// EXCEL EXPORT (Chỉ 1 bảng duy nhất theo nội dung sau filter)
// ==========================================
async function exportToExcel() {
  if (isExporting.value) return;
  isExporting.value = true;

  try {
    const teamTitle = props.teamName || 'QLCV';
    const now = new Date();
    const pad = n => String(n).padStart(2, '0');
    const fileDate = `${now.getFullYear()}${pad(now.getMonth() + 1)}${pad(now.getDate())}`;

    // Chỉ hiển thị đúng 1 bảng duy nhất, hàng đầu tiên là tiêu đề cột
    const aoaData = [
      ['STT', 'NHIỆM VỤ', 'CÁN BỘ THỰC HIỆN', 'THỜI GIAN HOÀN THÀNH', 'KẾT QUẢ', 'GHI CHÚ']
    ];

    // Nội dung export lấy từ filteredTasks (sau khi filter trên app; nếu không filter thì lấy tất cả)
    filteredTasks.value.forEach((task, idx) => {
      const staffNames = getStaffNamesText(task);
      const timeText = formatCompletionDate(task);
      const resultText = (task.status === 'DONE' && task.result) ? task.result : '';
      const noteText = task.note || '';

      aoaData.push([
        idx + 1,
        task.title,
        staffNames,
        timeText,
        resultText,
        noteText
      ]);
    });

    // Create Worksheet
    const ws = XLSX.utils.aoa_to_sheet(aoaData);

    // Column widths
    ws['!cols'] = [
      { wch: 8 },   // STT
      { wch: 48 },  // NHIỆM VỤ
      { wch: 30 },  // CÁN BỘ THỰC HIỆN
      { wch: 22 },  // THỜI GIAN HOÀN THÀNH
      { wch: 35 },  // KẾT QUẢ
      { wch: 25 }   // GHI CHÚ
    ];

    // Bỏ tính năng lọc (không dùng autofilter) theo yêu cầu người dùng
    const wb = XLSX.utils.book_new();
    XLSX.utils.book_append_sheet(wb, ws, 'Báo Cáo Tiến Độ');

    const cleanTeam = teamTitle.replace(/[^a-zA-Z0-9_\u00C0-\u024F\u1E00-\u1EFF]/g, '_');
    const defaultFilename = `Bao_cao_cong_viec_${cleanTeam}_${fileDate}.xlsx`;
    const wbout = XLSX.write(wb, { bookType: 'xlsx', type: 'array' });
    const uint8Arr = new Uint8Array(wbout);

    try {
      // Hiện hộp thoại File Explorer (SaveFileDialog) cho người dùng chọn nơi lưu và tên tệp
      const savedPath = await api.saveExcelFileWithDialog(defaultFilename, uint8Arr);
      if (savedPath) {
        try {
          await openPath(savedPath);
        } catch (openErr) {
          console.warn('Could not auto open Excel file:', openErr);
        }
      }
    } catch (err) {
      console.error('Tauri save dialog failed, falling back:', err);
      try {
        XLSX.writeFile(wb, defaultFilename);
      } catch (fbErr) {
        alert('Không thể xuất tệp Excel: ' + (err.message || err));
      }
    }
  } catch (err) {
    alert('Lỗi xuất tệp Excel: ' + (err.message || err));
  } finally {
    isExporting.value = false;
  }
}
</script>

<style scoped>
.reports-page {
  display: flex;
  flex-direction: column;
  gap: 10px;
  height: 100%;
  flex: 1;
  min-height: 0;
}

/* Compact Toolbar */
.filter-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  background-color: var(--bg-surface);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  padding: 8px 12px;
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
  background: none;
  border: none;
  cursor: pointer;
}

.btn-reset-filters {
  color: var(--accent);
  text-decoration: underline;
  cursor: pointer;
  font-size: 12px;
  background: none;
  border: none;
  padding: 4px 8px;
}

/* Nút Export Excel: Chỉ hover đổi màu, không nổi lên, không shadow */
.btn-excel {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 14px;
  border-radius: var(--radius-sm);
  background-color: #107c41; /* Microsoft Excel Green */
  color: white;
  font-size: 12.5px;
  font-weight: 700;
  border: none;
  cursor: pointer;
  transition: background-color 0.15s ease;
}

.btn-excel:hover:not(:disabled) {
  background-color: #0e6b37;
}

.btn-excel:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.excel-icon {
  font-size: 14px;
}

/* Document Table Card */
.table-card {
  background-color: var(--bg-surface);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  display: flex;
  flex-direction: column;
  flex: 1;
  min-height: 0;
  overflow: hidden;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.05);
}

.table-scroll-wrap {
  overflow-x: auto;
  overflow-y: auto;
  flex: 1;
  min-height: 0;
  width: 100%;
}

/* Formal Document-Style Table Layout */
.report-table {
  width: 100%;
  border-collapse: separate;
  border-spacing: 0;
  font-size: 13.5px;
  color: var(--text-ink);
  background-color: var(--bg-surface);
}

.report-table th, 
.report-table td {
  border-right: 1px solid #cbd5e1;
  border-bottom: 1px solid #cbd5e1;
  padding: 8px 10px;
  vertical-align: middle;
}

.report-table th:first-child,
.report-table td:first-child {
  border-left: 1px solid #cbd5e1;
}

[data-theme='dark'] .report-table th,
[data-theme='dark'] .report-table td,
[data-theme='dark'] .report-table th:first-child,
[data-theme='dark'] .report-table td:first-child {
  border-color: #334155;
}

.report-table thead th {
  background-color: var(--bg-sub, #f1f5f9);
  color: var(--text-ink);
  font-weight: 700;
  font-size: 12.5px;
  letter-spacing: 0.02em;
  text-align: center;
  position: sticky;
  top: 0;
  z-index: 10;
  border-top: 1px solid #cbd5e1;
  border-bottom: 2px solid #94a3b8;
}

[data-theme='dark'] .report-table thead th {
  background-color: #172033;
  border-top-color: #334155;
  border-bottom-color: #64748b;
}

/* Column specific styles */
.th-stt, .td-stt {
  width: 50px;
  text-align: center;
  font-weight: 600;
}

.th-task {
  text-align: left;
  min-width: 250px;
}

.th-staff {
  text-align: left;
  min-width: 180px;
}

.th-time {
  text-align: center;
  min-width: 150px;
}

.th-result {
  text-align: left;
  min-width: 220px;
}

.th-notes {
  text-align: left;
  min-width: 160px;
}

.report-row {
  transition: background-color 0.1s;
}

.report-row:hover {
  background-color: rgba(37, 99, 235, 0.03);
}

/* Cell typography */
.task-title-text {
  font-weight: 600;
  color: var(--text-ink);
  line-height: 1.4;
}

.staff-names-text {
  font-size: 13px;
  color: var(--text-ink);
}

.time-done-text {
  font-size: 13px;
  color: var(--text-ink);
  text-align: center;
  display: block;
}

/* Result cell: plain text, no buttons */
.result-text {
  font-size: 13px;
  color: var(--text-ink);
  line-height: 1.35;
  white-space: pre-wrap;
}

/* Note inline input */
.note-inline-input {
  width: 100%;
  padding: 4px 6px;
  font-size: 12.5px;
  border: 1px solid transparent;
  border-radius: 3px;
  background: transparent;
  color: var(--text-ink);
  transition: border-color 0.15s, background-color 0.15s;
}
.note-inline-input:hover {
  background-color: var(--bg-sub);
  border-color: var(--border);
}
.note-inline-input:focus {
  background-color: var(--bg-surface);
  border-color: var(--border-focus);
  outline: none;
}

.note-readonly {
  font-size: 12.5px;
  color: var(--text-ink);
}

.td-empty {
  text-align: center;
  padding: 40px !important;
}

.empty-wrap {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  color: var(--text-sub);
}

.empty-icon {
  font-size: 32px;
}
</style>