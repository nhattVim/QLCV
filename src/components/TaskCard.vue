<template>
  <div 
    class="task-card" 
    :class="cardClass" 
    :data-task-id="task.id"
    @click="$emit('view', task)"
  >
    <!-- Top Row: Priority Badge & ID & Admin Tools -->
    <div class="card-header">
      <div class="header-left">
        <span class="badge-priority" :class="priorityBadgeClass">
          {{ priorityLabel }}
        </span>
        <span class="task-id">#{{ shortId }}</span>
      </div>

      <!-- Admin Actions Only -->
      <div v-if="canManage" class="admin-actions">
        <button class="btn-icon" title="Chỉnh sửa công việc" @click.stop="$emit('edit', task)">
          ✏️
        </button>
        <button class="btn-icon btn-icon-danger" title="Xóa công việc" @click.stop="$emit('delete', task)">
          🗑️
        </button>
      </div>
    </div>

    <!-- Task Title -->
    <div class="card-title" :title="task.title">
      {{ task.title }}
    </div>

    <!-- Due Date & Assignee (Inline Jira/Trello Style, No Clutter) -->
    <div class="card-inline-meta">
      <span class="meta-date" :class="{ 'overdue-text': isOverdue, 'near-text': isNearDeadline }" :title="'Thời hạn hoàn thành: ' + fullFormattedDueDate">
        📅 {{ formattedDueDate }}
      </span>
      <div class="meta-right">
        <span v-if="task.attachments && task.attachments.length > 0" class="meta-attachments" :title="`${task.attachments.length} tệp đính kèm`">
          📎 {{ task.attachments.length }}
        </span>
        <span class="meta-assignee" :title="assigneesTooltip">
          👤 {{ assigneesShort }}
        </span>
      </div>
    </div>

    <!-- Workflow Move Actions Row -->
    <div class="card-workflow">
      <!-- Status: TODO -->
      <template v-if="task.status === 'TODO'">
        <button 
          class="btn-flow btn-flow-primary"
          title="Bắt đầu thực hiện"
          @click.stop="moveStatus('IN_PROGRESS')"
        >
          Bắt đầu ➔
        </button>
      </template>

      <!-- Status: IN_PROGRESS -->
      <template v-else-if="task.status === 'IN_PROGRESS'">
        <button 
          class="btn-flow"
          title="Đưa lại về Chưa làm"
          @click.stop="moveStatus('TODO')"
        >
          ⬅ Chờ
        </button>
        <button 
          class="btn-flow btn-flow-warning"
          title="Gửi báo cáo chờ quản trị duyệt"
          @click.stop="moveStatus('REVIEW')"
        >
          Gửi duyệt ➔
        </button>
      </template>

      <!-- Status: REVIEW (ADMIN HAS APPROVE, USER ONLY HAS RECALL) -->
      <template v-else-if="task.status === 'REVIEW'">
        <!-- Only Admin can approve -->
        <template v-if="canManage">
          <button 
            class="btn-flow"
            title="Trả lại yêu cầu làm tiếp"
            @click.stop="moveStatus('IN_PROGRESS')"
          >
            ⬅ Trả lại
          </button>
          <button 
            class="btn-flow btn-flow-success"
            title="Phê duyệt hoàn thành công việc"
            @click.stop="$emit('approve', task.id)"
          >
            ✓ Duyệt
          </button>
        </template>
        <!-- Regular User CANNOT approve! Only see waiting tag or recall -->
        <template v-else>
          <span class="waiting-tag">⏳ Chờ duyệt</span>
          <button 
            class="btn-flow"
            title="Thu hồi để tiếp tục bổ sung"
            @click.stop="moveStatus('IN_PROGRESS')"
          >
            ⬅ Thu hồi
          </button>
        </template>
      </template>

      <!-- Status: DONE -->
      <template v-else-if="task.status === 'DONE'">
        <span class="done-tag">✓ Hoàn thành</span>
        <button 
          v-if="canManage"
          class="btn-flow"
          title="Mở lại công việc"
          @click.stop="moveStatus('IN_PROGRESS')"
        >
          ↺ Mở lại
        </button>
      </template>
    </div>
  </div>
</template>

<script setup>
import { ref, computed } from 'vue';

const props = defineProps({
  task: {
    type: Object,
    required: true,
  },
  userMap: {
    type: Object,
    default: () => ({}),
  },
  currentUser: {
    type: Object,
    required: true,
  },
});

const emit = defineEmits(['edit', 'delete', 'approve', 'status-change', 'view']);

const canManage = computed(() => props.currentUser?.role === 'admin' || props.currentUser?.role === 'super_admin');

const now = new Date();

const dueDateTime = computed(() => {
  if (!props.task.due_date) return null;
  const d = new Date(props.task.due_date);
  return isNaN(d.getTime()) ? null : d;
});

const isOverdue = computed(() => {
  if (!dueDateTime.value || props.task.status === 'DONE') return false;
  return dueDateTime.value < now;
});

const isNearDeadline = computed(() => {
  if (!dueDateTime.value || props.task.status === 'DONE' || isOverdue.value) return false;
  const diffHours = (dueDateTime.value.getTime() - now.getTime()) / (1000 * 60 * 60);
  return diffHours >= 0 && diffHours <= 48;
});

const cardClass = computed(() => {
  if (props.task.status === 'DONE') return 'border-done';
  if (isOverdue.value) return 'border-overdue';
  if (isNearDeadline.value) return 'border-near';
  return 'border-normal';
});

const priorityLabel = computed(() => {
  switch (props.task.priority) {
    case 'Urgent': return '🔥 Khẩn';
    case 'High': return '⚡ Cao';
    case 'Medium': return '🔹 Vừa';
    default: return '🟢 Thấp';
  }
});

const priorityBadgeClass = computed(() => {
  switch (props.task.priority) {
    case 'Urgent': return 'prio-urgent';
    case 'High': return 'prio-high';
    case 'Medium': return 'prio-medium';
    default: return 'prio-low';
  }
});

const assigneesList = computed(() => {
  if (!props.task.assigned_to || props.task.assigned_to.length === 0) {
    return [];
  }
  const hasUsers = Object.keys(props.userMap).length > 0;
  if (hasUsers) {
    return props.task.assigned_to
      .filter(email => Boolean(props.userMap[email]))
      .map(email => props.userMap[email]);
  }
  return props.task.assigned_to.map(email => props.userMap[email] || email);
});

const assigneesShort = computed(() => {
  const list = assigneesList.value;
  if (list.length === 0) return 'Chưa gán';
  if (list.length === 1) return list[0];
  return `${list[0]} (+${list.length - 1})`;
});

const assigneesTooltip = computed(() => {
  return assigneesList.value.join(', ') || 'Chưa phân công';
});

const formattedDueDate = computed(() => {
  if (!dueDateTime.value) return 'Không hạn';
  const d = dueDateTime.value;
  const pad = (n) => String(n).padStart(2, '0');
  const time = `${pad(d.getHours())}:${pad(d.getMinutes())}`;
  return `${time} ${pad(d.getDate())}/${pad(d.getMonth() + 1)}`;
});

const fullFormattedDueDate = computed(() => {
  if (!dueDateTime.value) return 'Không có thời hạn';
  const d = dueDateTime.value;
  const pad = (n) => String(n).padStart(2, '0');
  const time = `${pad(d.getHours())}:${pad(d.getMinutes())}`;
  return `${time} ngày ${pad(d.getDate())}/${pad(d.getMonth() + 1)}/${d.getFullYear()}`;
});

const shortId = computed(() => {
  if (!props.task.id) return '';
  return props.task.id.slice(-4);
});

function moveStatus(newStatus) {
  emit('status-change', { id: props.task.id, status: newStatus });
}
</script>

<style scoped>
.task-card {
  background-color: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  padding: 10px 11px;
  display: flex;
  flex-direction: column;
  gap: 7px;
  border-left-width: 3.5px;
  cursor: grab;
  user-select: none;
  -webkit-user-select: none;
  -webkit-user-drag: none;
  transition: transform 0.15s ease, box-shadow 0.15s ease, opacity 0.15s ease, border-color 0.12s ease;
}

.task-card:active {
  cursor: grabbing;
}


.task-card:hover {
  border-color: var(--accent);
}

.border-normal {
  border-left-color: var(--border);
}
.border-overdue {
  border-left-color: var(--danger) !important;
}
.border-near {
  border-left-color: var(--warning) !important;
}
.border-done {
  border-left-color: var(--success) !important;
  opacity: 0.88;
}

.card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 6px;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 6px;
}

.badge-priority {
  font-size: 10.5px;
  font-weight: 700;
  padding: 1px 6px;
  border-radius: var(--radius-sm);
  line-height: 1.3;
}
.prio-urgent {
  background-color: #fee2e2;
  color: #dc2626;
}
[data-theme="dark"] .prio-urgent {
  background-color: #450a0a;
  color: #f87171;
}

.prio-high {
  background-color: #ffedd5;
  color: #c2410c;
}
[data-theme="dark"] .prio-high {
  background-color: #431407;
  color: #fb923c;
}

.prio-medium {
  background-color: #dbeafe;
  color: #1d4ed8;
}
[data-theme="dark"] .prio-medium {
  background-color: #172554;
  color: #60a5fa;
}

.prio-low {
  background-color: #d1fae5;
  color: #047857;
}
[data-theme="dark"] .prio-low {
  background-color: #064e3b;
  color: #34d399;
}

.task-id {
  font-size: 11px;
  color: var(--text-sub);
  font-family: monospace;
}

.admin-actions {
  display: flex;
  align-items: center;
  gap: 2px;
}

.btn-icon {
  padding: 2px 4px;
  font-size: 11px;
  border-radius: var(--radius-sm);
  background: transparent;
  border: 1px solid transparent;
  color: var(--text-muted);
}
.btn-icon:hover {
  background-color: var(--bg-hover);
  border-color: var(--border);
  color: var(--text-ink);
}
.btn-icon-danger:hover {
  background-color: var(--danger-sub);
  color: var(--danger);
  border-color: var(--danger);
}

.card-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-ink);
  line-height: 1.35;
  word-break: break-word;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.card-inline-meta {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 6px;
  font-size: 11.5px;
  color: var(--text-muted);
}

.meta-date {
  font-size: 11.5px;
  white-space: nowrap;
}

.overdue-text {
  color: var(--danger);
  font-weight: 700;
}
.near-text {
  color: var(--warning);
  font-weight: 700;
}

.meta-right {
  display: flex;
  align-items: center;
  gap: 6px;
  min-width: 0;
}

.meta-attachments {
  font-size: 10.5px;
  font-weight: 700;
  color: var(--accent);
  background-color: var(--accent-sub);
  padding: 1px 5px;
  border-radius: var(--radius-sm);
  white-space: nowrap;
  flex-shrink: 0;
}

.meta-assignee {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 140px;
  color: var(--accent);
  font-weight: 500;
}

.card-workflow {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 5px;
  padding-top: 5px;
  border-top: 1px solid var(--border-sub);
  margin-top: 1px;
}

.btn-flow {
  padding: 2px 7px;
  font-size: 11px;
  font-weight: 600;
  border-radius: var(--radius-sm);
  background-color: var(--bg-surface);
  border: 1px solid var(--border);
  color: var(--text-ink);
}
.btn-flow:hover {
  background-color: var(--bg-hover);
  border-color: var(--text-sub);
}

.btn-flow-primary {
  background-color: var(--accent-sub);
  color: var(--accent);
  border-color: var(--accent);
}
.btn-flow-primary:hover {
  background-color: var(--accent);
  color: #fff;
}

.btn-flow-warning {
  background-color: var(--warning-sub);
  color: var(--warning);
  border-color: var(--warning);
}
.btn-flow-warning:hover {
  background-color: var(--warning);
  color: #fff;
}

.btn-flow-success {
  background-color: var(--success-sub);
  color: var(--success);
  border-color: var(--success);
}
.btn-flow-success:hover {
  background-color: var(--success);
  color: #fff;
}

.waiting-tag {
  font-size: 11px;
  font-weight: 600;
  color: var(--warning);
  margin-right: auto;
}

.done-tag {
  font-size: 11px;
  font-weight: 700;
  color: var(--success);
  margin-right: auto;
}
</style>
