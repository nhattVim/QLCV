<template>
  <div class="reports-page">
    <!-- Overview Metric Cards -->
    <div class="reports-header-grid">
      <div class="report-kpi-card">
        <span class="report-kpi-label">TỔNG KHỐI LƯỢNG CÔNG VIỆC</span>
        <div class="report-kpi-val tabular-nums">{{ tasks.length }}</div>
        <span class="report-kpi-sub">Số lượng nhiệm vụ đã phát sinh</span>
      </div>

      <div class="report-kpi-card">
        <span class="report-kpi-label">TỈ LỆ HOÀN THÀNH</span>
        <div class="report-kpi-val text-done tabular-nums">{{ completionRate }}%</div>
        <span class="report-kpi-sub">{{ completedCount }} / {{ tasks.length }} nhiệm vụ hoàn tất</span>
      </div>

      <div class="report-kpi-card">
        <span class="report-kpi-label">ĐANG XỬ LÝ & CHỜ DUYỆT</span>
        <div class="report-kpi-val text-progress tabular-nums">{{ inProgressAndReviewCount }}</div>
        <span class="report-kpi-sub">Nhiệm vụ đang trên tiến trình</span>
      </div>

      <div class="report-kpi-card">
        <span class="report-kpi-label">CÔNG VIỆC QUÁ HẠN</span>
        <div class="report-kpi-val text-overdue tabular-nums">{{ overdueCount }}</div>
        <span class="report-kpi-sub">{{ overdueRate }}% trên tổng nhiệm vụ</span>
      </div>
    </div>

    <!-- Staff Workload Analysis -->
    <div class="report-card">
      <div class="report-card-header">
        <h3 class="report-card-title">👥 Phân bổ khối lượng & Hiệu suất Cán bộ</h3>
        <span class="report-card-sub">Thống kê số lượng việc được giao và tỉ lệ hoàn thành theo từng cán bộ</span>
      </div>

      <div class="workload-list">
        <div v-for="item in staffWorkload" :key="item.email" class="workload-item">
          <div class="workload-info">
            <div class="workload-avatar">{{ item.name.charAt(0) }}</div>
            <div class="workload-meta">
              <span class="workload-name">{{ item.name }}</span>
              <span class="workload-email">{{ item.email }}</span>
            </div>
            <div class="workload-counts tabular-nums">
              <span class="count-done">{{ item.done }} hoàn thành</span> / 
              <span class="count-total">{{ item.total }} được giao</span>
              <span v-if="item.overdue > 0" class="count-overdue">({{ item.overdue }} quá hạn)</span>
            </div>
          </div>

          <div class="workload-bar-track">
            <div 
              class="workload-bar-fill" 
              :style="{ width: `${item.rate}%` }"
              :class="{ 'bar-high': item.rate >= 80, 'bar-mid': item.rate >= 40 && item.rate < 80, 'bar-low': item.rate < 40 }"
            ></div>
          </div>
          <span class="workload-rate tabular-nums">{{ item.rate }}%</span>
        </div>
      </div>
    </div>

    <!-- Status & Priority Distributions -->
    <div class="distribution-grid">
      <!-- Status Distribution -->
      <div class="report-card">
        <div class="report-card-header">
          <h3 class="report-card-title">📊 Phân bổ theo Trạng thái</h3>
        </div>
        <div class="dist-list">
          <div class="dist-item">
            <span class="dist-label">✓ Hoàn thành</span>
            <div class="dist-bar-track">
              <div class="dist-bar-fill bar-done" :style="{ width: `${getPercentage(completedCount)}%` }"></div>
            </div>
            <span class="dist-val tabular-nums">{{ completedCount }} ({{ getPercentage(completedCount) }}%)</span>
          </div>

          <div class="dist-item">
            <span class="dist-label">🕒 Chờ phê duyệt</span>
            <div class="dist-bar-track">
              <div class="dist-bar-fill bar-review" :style="{ width: `${getPercentage(reviewCount)}%` }"></div>
            </div>
            <span class="dist-val tabular-nums">{{ reviewCount }} ({{ getPercentage(reviewCount) }}%)</span>
          </div>

          <div class="dist-item">
            <span class="dist-label">⚡ Đang thực hiện</span>
            <div class="dist-bar-track">
              <div class="dist-bar-fill bar-progress" :style="{ width: `${getPercentage(inProgressCount)}%` }"></div>
            </div>
            <span class="dist-val tabular-nums">{{ inProgressCount }} ({{ getPercentage(inProgressCount) }}%)</span>
          </div>

          <div class="dist-item">
            <span class="dist-label">⏳ Chưa thực hiện</span>
            <div class="dist-bar-track">
              <div class="dist-bar-fill bar-todo" :style="{ width: `${getPercentage(todoCount)}%` }"></div>
            </div>
            <span class="dist-val tabular-nums">{{ todoCount }} ({{ getPercentage(todoCount) }}%)</span>
          </div>
        </div>
      </div>

      <!-- Priority Distribution -->
      <div class="report-card">
        <div class="report-card-header">
          <h3 class="report-card-title">⚡ Phân bổ theo Mức độ ưu tiên</h3>
        </div>
        <div class="dist-list">
          <div class="dist-item">
            <span class="dist-label">🔥 Khẩn cấp</span>
            <div class="dist-bar-track">
              <div class="dist-bar-fill bar-urgent" :style="{ width: `${getPercentage(urgentCount)}%` }"></div>
            </div>
            <span class="dist-val tabular-nums">{{ urgentCount }}</span>
          </div>

          <div class="dist-item">
            <span class="dist-label">⚡ Ưu tiên cao</span>
            <div class="dist-bar-track">
              <div class="dist-bar-fill bar-high-prio" :style="{ width: `${getPercentage(highCount)}%` }"></div>
            </div>
            <span class="dist-val tabular-nums">{{ highCount }}</span>
          </div>

          <div class="dist-item">
            <span class="dist-label">🔹 Trung bình</span>
            <div class="dist-bar-track">
              <div class="dist-bar-fill bar-medium" :style="{ width: `${getPercentage(mediumCount)}%` }"></div>
            </div>
            <span class="dist-val tabular-nums">{{ mediumCount }}</span>
          </div>

          <div class="dist-item">
            <span class="dist-label">🟢 Thấp</span>
            <div class="dist-bar-track">
              <div class="dist-bar-fill bar-low-prio" :style="{ width: `${getPercentage(lowCount)}%` }"></div>
            </div>
            <span class="dist-val tabular-nums">{{ lowCount }}</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { computed } from 'vue';

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
});

const now = new Date();

const completedCount = computed(() => props.tasks.filter(t => t.status === 'DONE').length);
const reviewCount = computed(() => props.tasks.filter(t => t.status === 'REVIEW').length);
const inProgressCount = computed(() => props.tasks.filter(t => t.status === 'IN_PROGRESS').length);
const todoCount = computed(() => props.tasks.filter(t => t.status === 'TODO').length);
const inProgressAndReviewCount = computed(() => reviewCount.value + inProgressCount.value);

const overdueCount = computed(() => {
  return props.tasks.filter(t => {
    if (t.status === 'DONE' || !t.due_date) return false;
    return new Date(t.due_date) < now;
  }).length;
});

const completionRate = computed(() => {
  if (props.tasks.length === 0) return 0;
  return Math.round((completedCount.value / props.tasks.length) * 100);
});

const overdueRate = computed(() => {
  if (props.tasks.length === 0) return 0;
  return Math.round((overdueCount.value / props.tasks.length) * 100);
});

const urgentCount = computed(() => props.tasks.filter(t => t.priority === 'Urgent').length);
const highCount = computed(() => props.tasks.filter(t => t.priority === 'High').length);
const mediumCount = computed(() => props.tasks.filter(t => t.priority === 'Medium').length);
const lowCount = computed(() => props.tasks.filter(t => t.priority === 'Low').length);

function getPercentage(count) {
  if (props.tasks.length === 0) return 0;
  return Math.round((count / props.tasks.length) * 100);
}

const staffWorkload = computed(() => {
  return props.users.map(u => {
    const userTasks = props.tasks.filter(t => (t.assigned_to || []).includes(u.email));
    const total = userTasks.length;
    const done = userTasks.filter(t => t.status === 'DONE').length;
    const overdue = userTasks.filter(t => {
      if (t.status === 'DONE' || !t.due_date) return false;
      return new Date(t.due_date) < now;
    }).length;

    const rate = total > 0 ? Math.round((done / total) * 100) : 0;

    return {
      name: u.full_name,
      email: u.email,
      total,
      done,
      overdue,
      rate,
    };
  }).sort((a, b) => b.total - a.total);
});
</script>

<style scoped>
.reports-page {
  display: flex;
  flex-direction: column;
  gap: 20px;
  padding-bottom: 36px;
}

.reports-header-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 14px;
}

@media (max-width: 1000px) {
  .reports-header-grid {
    grid-template-columns: repeat(2, 1fr);
  }
}

.report-kpi-card {
  background-color: var(--bg-surface);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.report-kpi-label {
  font-size: 11px;
  font-weight: 700;
  color: var(--text-sub);
  letter-spacing: 0.04em;
}

.report-kpi-val {
  font-size: 26px;
  font-weight: 800;
  color: var(--text-ink);
  line-height: 1.1;
}

.text-done { color: #059669; }
.text-progress { color: #2563eb; }
.text-overdue { color: #dc2626; }

.report-kpi-sub {
  font-size: 11px;
  color: var(--text-sub);
}

.report-card {
  background-color: var(--bg-surface);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.report-card-title {
  font-size: 15px;
  font-weight: 800;
  color: var(--text-ink);
}

.report-card-sub {
  font-size: 12px;
  color: var(--text-sub);
}

.workload-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.workload-item {
  display: grid;
  grid-template-columns: 280px 1fr 45px;
  align-items: center;
  gap: 16px;
  padding: 8px 0;
  border-bottom: 1px solid var(--border);
}

.workload-item:last-child {
  border-bottom: none;
}

.workload-info {
  display: flex;
  align-items: center;
  gap: 10px;
}

.workload-avatar {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  background-color: var(--accent);
  color: #fff;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 12px;
  font-weight: 700;
  flex-shrink: 0;
}

.workload-meta {
  display: flex;
  flex-direction: column;
  min-width: 0;
}

.workload-name {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-ink);
}

.workload-email {
  font-size: 11px;
  color: var(--text-sub);
}

.workload-counts {
  margin-left: auto;
  font-size: 11px;
  color: var(--text-muted);
}
.count-done { color: #059669; font-weight: 600; }
.count-overdue { color: #dc2626; font-weight: 600; }

.workload-bar-track {
  height: 8px;
  background-color: var(--bg-sub);
  border-radius: var(--radius-pill);
  overflow: hidden;
}

.workload-bar-fill {
  height: 100%;
  border-radius: var(--radius-pill);
  transition: width 0.3s ease;
}

.bar-high { background-color: #10b981; }
.bar-mid { background-color: #3b82f6; }
.bar-low { background-color: #f59e0b; }

.workload-rate {
  font-size: 12px;
  font-weight: 700;
  color: var(--text-ink);
  text-align: right;
}

.distribution-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 16px;
}

@media (max-width: 900px) {
  .distribution-grid {
    grid-template-columns: 1fr;
  }
}

.dist-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.dist-item {
  display: grid;
  grid-template-columns: 140px 1fr 100px;
  align-items: center;
  gap: 12px;
  font-size: 12px;
}

.dist-label {
  font-weight: 600;
  color: var(--text-ink);
}

.dist-bar-track {
  height: 7px;
  background-color: var(--bg-sub);
  border-radius: var(--radius-pill);
  overflow: hidden;
}

.dist-bar-fill {
  height: 100%;
  border-radius: var(--radius-pill);
}

.bar-done { background-color: #10b981; }
.bar-review { background-color: #f59e0b; }
.bar-progress { background-color: #3b82f6; }
.bar-todo { background-color: #64748b; }

.bar-urgent { background-color: #ef4444; }
.bar-high-prio { background-color: #f97316; }
.bar-medium { background-color: #3b82f6; }
.bar-low-prio { background-color: #10b981; }

.dist-val {
  text-align: right;
  color: var(--text-muted);
}
</style>
