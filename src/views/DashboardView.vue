<template>
  <div class="dashboard-page">
    <!-- Top Stats Row -->
    <div class="stats-grid">
      <div class="stat-card">
        <div class="stat-header">
          <span class="stat-label">TỔNG CÔNG VIỆC</span>
          <span class="stat-icon icon-total">📋</span>
        </div>
        <div class="stat-value tabular-nums">{{ stats.total }}</div>
        <div class="stat-sub">Toàn bộ nhiệm vụ trong hệ thống</div>
      </div>

      <div class="stat-card">
        <div class="stat-header">
          <span class="stat-label">CHƯA THỰC HIỆN</span>
          <span class="stat-icon icon-todo">⏳</span>
        </div>
        <div class="stat-value tabular-nums">{{ stats.todo }}</div>
        <div class="stat-sub">Đang chờ cán bộ tiếp nhận</div>
      </div>

      <div class="stat-card">
        <div class="stat-header">
          <span class="stat-label">ĐANG THỰC HIỆN</span>
          <span class="stat-icon icon-progress">⚡</span>
        </div>
        <div class="stat-value tabular-nums text-progress">{{ stats.in_progress }}</div>
        <div class="stat-sub">Đang được xử lý tích cực</div>
      </div>

      <div class="stat-card">
        <div class="stat-header">
          <span class="stat-label">CHỜ PHÊ DUYỆT</span>
          <span class="stat-icon icon-review">🕒</span>
        </div>
        <div class="stat-value tabular-nums text-review">{{ stats.review }}</div>
        <div class="stat-sub">Cán bộ đã nộp kết quả</div>
      </div>

      <div class="stat-card">
        <div class="stat-header">
          <span class="stat-label">HOÀN THÀNH</span>
          <span class="stat-icon icon-done">✓</span>
        </div>
        <div class="stat-value tabular-nums text-done">{{ stats.done }}</div>
        <div class="stat-sub">Đã nghiệm thu đạt yêu cầu</div>
      </div>

      <div class="stat-card" :class="{ 'stat-overdue-alert': stats.overdue > 0 }">
        <div class="stat-header">
          <span class="stat-label">⚠️ ĐÃ QUÁ HẠN</span>
          <span class="stat-icon icon-overdue">🔥</span>
        </div>
        <div class="stat-value tabular-nums text-overdue">{{ stats.overdue }}</div>
        <div class="stat-sub">Cần đôn đốc xử lý gấp!</div>
      </div>
    </div>

    <!-- Progress Rate Banner -->
    <div class="progress-banner">
      <div class="banner-info">
        <span class="banner-title">Tiến độ hoàn thành tổng thể</span>
        <span class="banner-percentage tabular-nums">{{ completionPercentage }}%</span>
      </div>
      <div class="progress-bar-track">
        <div class="progress-bar-fill" :style="{ width: `${completionPercentage}%` }"></div>
      </div>
    </div>

    <!-- Urgent Section -->
    <div v-if="urgentTasks.length > 0" class="dashboard-section">
      <div class="section-header">
        <div class="section-title-wrap">
          <h2 class="section-title">🚨 Công việc cần chú ý khẩn cấp</h2>
          <span class="section-badge">{{ urgentTasks.length }}</span>
        </div>
        <button class="btn-ghost" @click="$emit('view-all-tasks')">Xem toàn bộ →</button>
      </div>

      <div class="task-grid-2col">
        <TaskCard 
          v-for="task in urgentTasks" 
          :key="task.id" 
          :task="task" 
          :user-map="userMap"
          :current-user="currentUser"
          @edit="$emit('edit-task', $event)"
          @delete="$emit('delete-task', $event)"
          @approve="$emit('approve-task', $event)"
          @status-change="$emit('status-change', $event)"
        />
      </div>
    </div>

    <!-- Recent Tasks Section -->
    <div class="dashboard-section">
      <div class="section-header">
        <div class="section-title-wrap">
          <h2 class="section-title">📋 Công việc phân công gần đây</h2>
          <span class="section-badge">{{ recentTasks.length }}</span>
        </div>
        <button class="btn-ghost" @click="$emit('view-all-tasks')">Xem tất cả công việc →</button>
      </div>

      <div v-if="recentTasks.length === 0" class="empty-state">
        <span class="empty-icon">🎉</span>
        <div class="empty-title">Không có công việc nào cần thực hiện!</div>
        <p class="empty-desc">Toàn bộ công việc đã hoàn thành hoặc chưa có nhiệm vụ mới được giao.</p>
      </div>

      <div v-else class="task-grid-2col">
        <TaskCard 
          v-for="task in recentTasks" 
          :key="task.id" 
          :task="task" 
          :user-map="userMap"
          :current-user="currentUser"
          @edit="$emit('edit-task', $event)"
          @delete="$emit('delete-task', $event)"
          @approve="$emit('approve-task', $event)"
          @status-change="$emit('status-change', $event)"
        />
      </div>
    </div>
  </div>
</template>

<script setup>
import { computed } from 'vue';
import TaskCard from '../components/TaskCard.vue';

const props = defineProps({
  stats: {
    type: Object,
    default: () => ({
      total: 0,
      todo: 0,
      in_progress: 0,
      review: 0,
      done: 0,
      overdue: 0,
    }),
  },
  tasks: {
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
});

defineEmits(['view-all-tasks', 'edit-task', 'delete-task', 'approve-task', 'status-change']);

const completionPercentage = computed(() => {
  if (!props.stats.total || props.stats.total === 0) return 0;
  return Math.round((props.stats.done / props.stats.total) * 100);
});

const now = new Date();

const urgentTasks = computed(() => {
  return props.tasks.filter(t => {
    if (t.status === 'DONE') return false;
    if (t.priority === 'Urgent') return true;
    if (t.due_date) {
      const d = new Date(t.due_date);
      if (!isNaN(d.getTime())) {
        const diffHours = (d.getTime() - now.getTime()) / (1000 * 60 * 60);
        return diffHours <= 48; // quá hạn hoặc sắp đến hạn dưới 48h
      }
    }
    return false;
  }).slice(0, 4);
});

const recentTasks = computed(() => {
  return props.tasks.slice(0, 6);
});
</script>

<style scoped>
.dashboard-page {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(6, 1fr);
  gap: 12px;
}

@media (max-width: 1200px) {
  .stats-grid {
    grid-template-columns: repeat(3, 1fr);
  }
}

.stat-card {
  background-color: var(--bg-surface);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  padding: 14px 16px;
  display: flex;
  flex-direction: column;
  gap: 6px;
  box-shadow: var(--shadow-sm);
}

.stat-overdue-alert {
  border-color: var(--danger) !important;
  background-color: var(--danger-sub);
}

.stat-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.stat-label {
  font-size: 11px;
  font-weight: 700;
  color: var(--text-sub);
  letter-spacing: 0.04em;
}

.stat-icon {
  font-size: 14px;
}

.stat-value {
  font-size: 26px;
  font-weight: 800;
  color: var(--text-ink);
  line-height: 1.1;
}

.text-progress { color: #2563eb; }
.text-review { color: #d97706; }
.text-done { color: #059669; }
.text-overdue { color: #dc2626; }

.stat-sub {
  font-size: 11px;
  color: var(--text-sub);
}

.progress-banner {
  background-color: var(--bg-surface);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  padding: 14px 18px;
  display: flex;
  flex-direction: column;
  gap: 8px;
  box-shadow: var(--shadow-sm);
}

.banner-info {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.banner-title {
  font-size: 13px;
  font-weight: 700;
  color: var(--text-ink);
}

.banner-percentage {
  font-size: 16px;
  font-weight: 800;
  color: var(--accent);
}

.progress-bar-track {
  height: 8px;
  background-color: var(--bg-sub);
  border-radius: var(--radius-pill);
  overflow: hidden;
}

.progress-bar-fill {
  height: 100%;
  background: linear-gradient(90deg, #3b82f6, #10b981);
  border-radius: var(--radius-pill);
  transition: width 0.4s ease;
}

.dashboard-section {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.section-title-wrap {
  display: flex;
  align-items: center;
  gap: 8px;
}

.section-title {
  font-size: 15px;
  font-weight: 800;
  color: var(--text-ink);
}

.section-badge {
  font-size: 11px;
  font-weight: 700;
  padding: 2px 7px;
  border-radius: var(--radius-pill);
  background-color: var(--bg-sub);
  color: var(--text-muted);
}

/* Modern 2-Column Task Grid */
.task-grid-2col {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 16px;
}

@media (max-width: 900px) {
  .task-grid-2col {
    grid-template-columns: 1fr;
  }
}

.empty-state {
  background-color: var(--bg-surface);
  border: 1px dashed var(--border);
  border-radius: var(--radius-md);
  padding: 40px;
  text-align: center;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
}

.empty-icon {
  font-size: 32px;
}

.empty-title {
  font-size: 15px;
  font-weight: 700;
  color: var(--text-ink);
}

.empty-desc {
  font-size: 13px;
  color: var(--text-sub);
  max-width: 400px;
}
</style>
