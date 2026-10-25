<template>
  <div class="modal-overlay" @click.self="$emit('close')">
    <div class="modal-content ticket-modal">
      <div class="modal-header">
        <h3 class="modal-title">
          <template v-if="isAdmin">
            {{ isEdit ? 'Chỉnh sửa công việc' : 'Giao công việc mới' }}
          </template>
          <template v-else>
            Chi tiết công việc
          </template>
        </h3>
        <button class="btn-ghost" @click="$emit('close')">✕</button>
      </div>

      <form class="modal-body ticket-body" @submit.prevent="handleSubmit">
        <div class="ticket-layout">
          <section class="ticket-main">
            <div class="form-group title-group">
              <label class="form-label">Tiêu đề công việc <span v-if="isAdmin" class="required">*</span></label>
              <input
                v-if="isAdmin"
                v-model="form.title"
                type="text"
                class="form-control title-input"
                placeholder="Nhập tên nhiệm vụ, công văn, công việc cần làm..."
                required
                autofocus
              />
              <h2 v-else class="view-title">{{ form.title || 'Không có tiêu đề' }}</h2>
            </div>

            <div class="form-group editor-group">
              <label class="form-label">Nội dung chi tiết</label>
              <RichEditor
                v-model="form.description"
                :readonly="!isAdmin"
                placeholder="Ghi chú hướng dẫn, yêu cầu, số công văn..."
              />
            </div>
          </section>

          <aside class="ticket-side">
            <div class="form-group">
              <label class="form-label">Độ ưu tiên</label>
              <select v-if="isAdmin" v-model="form.priority" class="form-control">
                <option value="Urgent">Khẩn cấp</option>
                <option value="High">Ưu tiên cao</option>
                <option value="Medium">Trung bình</option>
                <option value="Low">Thấp</option>
              </select>
              <div v-else class="readonly-box">
                <span class="badge" :class="getPriorityBadgeClass(form.priority)">
                  {{ getPriorityLabel(form.priority) }}
                </span>
              </div>
            </div>

            <div v-if="isEdit" class="form-group">
              <label class="form-label">Trạng thái</label>
              <select v-model="form.status" class="form-control">
                <option value="TODO">Chưa làm</option>
                <option value="IN_PROGRESS">Đang làm</option>
                <option value="REVIEW">Chờ duyệt</option>
                <option v-if="isAdmin" value="DONE">Hoàn thành</option>
              </select>
              <span v-if="!isAdmin" class="form-hint">
                Khi xong việc, chuyển sang <strong>Chờ duyệt</strong> để quản trị phê duyệt.
              </span>
            </div>

            <div class="form-group">
              <label class="form-label">Thời hạn hoàn thành</label>
              <template v-if="isAdmin">
                <div class="date-presets">
                  <button type="button" class="preset-btn" @click="setPreset('today')">Hôm nay 17h</button>
                  <button type="button" class="preset-btn" @click="setPreset('tomorrow')">Ngày mai</button>
                  <button type="button" class="preset-btn" @click="setPreset('3days')">3 ngày</button>
                  <button type="button" class="preset-btn" @click="setPreset('1week')">1 tuần</button>
                </div>
                <div class="datetime-inputs">
                  <div class="datetime-field">
                    <input v-model="datePart" type="date" class="form-control" required />
                    <Calendar class="datetime-icon" :size="16" :stroke-width="2" />
                  </div>
                  <div class="datetime-field">
                    <input v-model="timePart" type="time" class="form-control" required />
                    <Clock class="datetime-icon" :size="16" :stroke-width="2" />
                  </div>
                </div>
              </template>
              <div v-else class="readonly-box">{{ formattedDueDateView }}</div>
            </div>

            <div class="form-group">
              <label class="form-label">Nhắc trước hạn</label>
              <div class="remind-btns">
                <button
                  v-for="d in [1, 2, 3]"
                  :key="d"
                  type="button"
                  class="remind-btn"
                  :class="{ active: form.remind_before_days === d }"
                  :disabled="!isAdmin"
                  @click="form.remind_before_days = d"
                >
                  {{ d }} ngày
                </button>
              </div>
              <span class="form-hint">{{ remindHint }}</span>
            </div>

            <div class="form-group">
              <label class="form-label">Phân công người thực hiện</label>
              <div v-if="isAdmin" class="assignee-select-box">
                <div
                  v-for="user in users"
                  :key="user.email"
                  class="assignee-checkbox-item"
                  :class="{ selected: form.assigned_to.includes(user.email) }"
                  @click="toggleAssignee(user.email)"
                >
                  <input
                    type="checkbox"
                    :checked="form.assigned_to.includes(user.email)"
                    @click.stop
                    @change="toggleAssignee(user.email)"
                  />
                  <span class="user-item-avatar">{{ user.full_name ? user.full_name.charAt(0) : 'C' }}</span>
                  <div class="user-item-info">
                    <span class="user-item-name">{{ user.full_name }}</span>
                    <span class="user-item-email">{{ user.email }}</span>
                  </div>
                </div>
              </div>
              <div v-else class="view-assignees-box">
                <div v-for="email in form.assigned_to" :key="email" class="view-assignee-chip">
                  {{ getAssigneeName(email) }}
                </div>
                <div v-if="form.assigned_to.length === 0" class="text-muted">Chưa phân công</div>
              </div>
              <span v-if="isAdmin" class="form-hint">Đã phân công: {{ form.assigned_to.length }} cán bộ</span>
            </div>
          </aside>
        </div>
      </form>

      <div class="modal-footer">
        <button type="button" class="btn-secondary" @click="$emit('close')">
          {{ isAdmin ? 'Hủy bỏ' : 'Đóng' }}
        </button>
        <button type="button" class="btn-primary" :disabled="loading" @click="handleSubmit">
          <span v-if="loading">Đang lưu...</span>
          <span v-else-if="isAdmin">{{ isEdit ? 'Lưu thay đổi' : 'Tạo công việc' }}</span>
          <span v-else>Cập nhật tiến độ</span>
        </button>
      </div>
    </div>
  </div>
</template>

<script setup>
import { computed, onMounted, reactive, ref } from 'vue';
import { Calendar, Clock } from 'lucide-vue-next';
import RichEditor from './RichEditor.vue';

const props = defineProps({
  taskData: {
    type: Object,
    default: null,
  },
  users: {
    type: Array,
    default: () => [],
  },
  currentUser: {
    type: Object,
    required: true,
  },
});

const emit = defineEmits(['close', 'save']);

const isEdit = Boolean(props.taskData && props.taskData.id);
const isAdmin = computed(() => props.currentUser.role === 'admin');
const loading = ref(false);

const datePart = ref('');
const timePart = ref('17:00');

const form = reactive({
  title: '',
  description: '',
  priority: 'Medium',
  status: 'TODO',
  assigned_to: [],
  remind_before_days: 2,
});

onMounted(() => {
  if (props.taskData) {
    form.title = props.taskData.title || '';
    form.description = props.taskData.description || '';
    form.priority = props.taskData.priority || 'Medium';
    form.status = props.taskData.status || 'TODO';
    form.assigned_to = [...(props.taskData.assigned_to || [])];
    form.remind_before_days = props.taskData.remind_before_days || 2;

    if (props.taskData.due_date) {
      try {
        const d = new Date(props.taskData.due_date);
        const pad = (n) => String(n).padStart(2, '0');
        datePart.value = `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`;
        timePart.value = `${pad(d.getHours())}:${pad(d.getMinutes())}`;
      } catch {
        setPreset('today');
      }
    } else {
      setPreset('today');
    }
  } else {
    setPreset('today');
  }
});

const formattedDueDateView = computed(() => {
  if (!datePart.value) return 'Không có hạn';
  return `${datePart.value} ${timePart.value}`;
});

const remindHint = computed(() => {
  if (!datePart.value) return 'Chọn hạn chót để tính ngày nhắc.';
  const due = new Date(`${datePart.value}T${timePart.value || '17:00'}:00`);
  due.setDate(due.getDate() - form.remind_before_days);
  const pad = (n) => String(n).padStart(2, '0');
  return `Nhắc lúc ${pad(due.getDate())}/${pad(due.getMonth() + 1)}/${due.getFullYear()} ${timePart.value}`;
});

function setPreset(preset) {
  const d = new Date();
  const pad = (n) => String(n).padStart(2, '0');

  if (preset === 'tomorrow') {
    d.setDate(d.getDate() + 1);
  } else if (preset === '3days') {
    d.setDate(d.getDate() + 3);
  } else if (preset === '1week') {
    d.setDate(d.getDate() + 7);
  }

  datePart.value = `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`;
  timePart.value = '17:00';
}

function toggleAssignee(email) {
  if (!isAdmin.value) return;
  const idx = form.assigned_to.indexOf(email);
  if (idx > -1) {
    form.assigned_to.splice(idx, 1);
  } else {
    form.assigned_to.push(email);
  }
}

function getAssigneeName(email) {
  const user = props.users.find((u) => u.email === email);
  return user ? user.full_name : email;
}

function getPriorityLabel(prio) {
  switch (prio) {
    case 'Urgent':
      return 'Khẩn cấp';
    case 'High':
      return 'Ưu tiên cao';
    case 'Medium':
      return 'Trung bình';
    default:
      return 'Thấp';
  }
}

function getPriorityBadgeClass(prio) {
  switch (prio) {
    case 'Urgent':
      return 'badge-urgent';
    case 'High':
      return 'badge-high';
    case 'Medium':
      return 'badge-medium';
    default:
      return 'badge-low';
  }
}

function handleSubmit() {
  if (isAdmin.value) {
    if (!form.title.trim()) return;

    let dueIso = null;
    if (datePart.value) {
      dueIso = new Date(`${datePart.value}T${timePart.value || '17:00'}:00`).toISOString();
    }

    loading.value = true;
    emit('save', {
      title: form.title.trim(),
      description: form.description || '',
      priority: form.priority,
      status: form.status,
      due_date: dueIso,
      assigned_to: form.assigned_to,
      remind_before_days: form.remind_before_days,
    });
  } else {
    loading.value = true;
    emit('save', {
      status: form.status,
    });
  }
}
</script>

<style scoped>
.ticket-modal {
  max-width: 1040px;
  width: min(1040px, 96vw);
  height: min(760px, 90vh);
}

.ticket-body {
  padding: 0;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.ticket-layout {
  display: grid;
  grid-template-columns: 2fr 1fr;
  min-height: 0;
  flex: 1;
  height: 100%;
}

.ticket-main {
  padding: 16px 18px;
  border-right: 1px solid var(--border);
  display: flex;
  flex-direction: column;
  min-width: 0;
  min-height: 0;
}

.ticket-side {
  padding: 16px 16px 12px;
  background: var(--bg-sub);
  overflow-y: auto;
  min-width: 0;
}

.required {
  color: var(--danger);
}

.title-input {
  font-size: 15px;
  font-weight: 700;
}

.view-title {
  font-size: 16px;
  font-weight: 800;
  line-height: 1.35;
}

.title-group {
  margin-bottom: 10px;
}

.editor-group {
  flex: 1;
  min-height: 0;
  margin-bottom: 0;
}

.date-presets {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  margin-bottom: 6px;
}

.preset-btn {
  font-size: 11px;
  padding: 2px 7px;
  border-radius: var(--radius-sm);
  background-color: var(--bg-surface);
  border: 1px solid var(--border);
  color: var(--text-muted);
}
.preset-btn:hover {
  background-color: var(--accent-sub);
  color: var(--accent);
  border-color: var(--accent);
}

.datetime-inputs {
  display: grid;
  grid-template-columns: 1.3fr 0.9fr;
  gap: 8px;
}

.datetime-field {
  position: relative;
}

.datetime-field .form-control {
  padding-right: 32px;
}

.datetime-field input[type='date']::-webkit-calendar-picker-indicator,
.datetime-field input[type='time']::-webkit-calendar-picker-indicator {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  margin: 0;
  padding: 0;
  opacity: 0;
  cursor: pointer;
}

.datetime-icon {
  position: absolute;
  right: 8px;
  top: 50%;
  transform: translateY(-50%);
  pointer-events: none;
  color: var(--text-ink);
}

.remind-btns {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 6px;
}

.remind-btn {
  padding: 6px 4px;
  font-size: 12px;
  font-weight: 600;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--bg-surface);
  color: var(--text-muted);
}
.remind-btn.active {
  background: var(--accent-sub);
  border-color: var(--accent);
  color: var(--accent);
}
.remind-btn:disabled {
  cursor: default;
  opacity: 0.9;
}

.readonly-box {
  padding: 8px 10px;
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  background: var(--bg-surface);
  font-weight: 600;
}

.assignee-select-box {
  max-height: 180px;
  overflow-y: auto;
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  background-color: var(--bg-surface);
}

.assignee-checkbox-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 10px;
  cursor: pointer;
  border-bottom: 1px solid var(--border-sub);
}
.assignee-checkbox-item:last-child {
  border-bottom: none;
}
.assignee-checkbox-item:hover {
  background-color: var(--bg-hover);
}
.assignee-checkbox-item.selected {
  background-color: var(--accent-sub);
}

.user-item-avatar {
  width: 22px;
  height: 22px;
  border-radius: 50%;
  background-color: var(--accent);
  color: #fff;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 11px;
  font-weight: 700;
  flex-shrink: 0;
}

.user-item-info {
  display: flex;
  flex-direction: column;
  min-width: 0;
}

.user-item-name {
  font-size: 12px;
  font-weight: 600;
}

.user-item-email {
  font-size: 10.5px;
  color: var(--text-sub);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.form-hint {
  font-size: 11.5px;
  color: var(--text-sub);
  line-height: 1.4;
}

.view-assignees-box {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.view-assignee-chip {
  background-color: var(--bg-surface);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  padding: 4px 10px;
  font-size: 12px;
  font-weight: 600;
  color: var(--accent);
}

.text-muted {
  color: var(--text-sub);
}
</style>
