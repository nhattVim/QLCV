<template>
  <div class="users-page">
    <!-- Toolbar -->
    <div class="users-toolbar">
      <div class="toolbar-left">
        <h2 class="section-title">Danh sách tài khoản Cán bộ ({{ users.length }})</h2>
        <span class="section-sub">Phân quyền vai trò và quản lý trạng thái tài khoản hệ thống</span>
      </div>

      <div class="toolbar-right">
        <button class="btn-primary" @click="$emit('add-user')">
          ➕ Thêm cán bộ mới
        </button>
      </div>
    </div>

    <!-- Users Table Container -->
    <div class="users-table-container">
      <table class="users-table">
        <thead>
          <tr>
            <th style="width: 200px;">Cán bộ</th>
            <th style="min-width: 200px;">Email</th>
            <th style="width: 150px;">Đội nhóm (Team)</th>
            <th style="width: 140px;">Vai trò</th>
            <th style="width: 130px;">Trạng thái</th>
            <th style="width: 120px;">Ngày tham gia</th>
            <th style="width: 200px; text-align: right;">Thao tác</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="user in filteredUsers" :key="user.email">
            <!-- User Name & Avatar -->
            <td>
              <div class="user-cell">
                <div class="user-avatar">{{ user.full_name ? user.full_name.charAt(0) : 'U' }}</div>
                <div class="user-cell-info">
                  <span class="user-cell-name">{{ user.full_name }}</span>
                  <span v-if="user.email === currentUser.email" class="self-badge">(Tôi)</span>
                </div>
              </div>
            </td>

            <!-- Email -->
            <td class="email-cell">{{ user.email }}</td>

            <!-- Team Name -->
            <td>
              <span class="team-chip">
                {{ getTeamName(user.team_id) }}
              </span>
            </td>

            <!-- Role Badge -->
            <td>
              <span 
                class="badge" 
                :class="user.role === 'super_admin' ? 'role-badge-super' : (user.role === 'admin' ? 'role-badge-admin' : 'role-badge-user')"
              >
                {{ user.role === 'super_admin' ? '⚡ Super Admin' : (user.role === 'admin' ? '👑 Quản trị viên' : '👤 Cán bộ') }}
              </span>
            </td>

            <!-- Active Status -->
            <td>
              <span class="badge" :class="user.is_active ? 'badge-active' : 'badge-inactive'">
                {{ user.is_active ? '🟢 Hoạt động' : '🔴 Đã khóa' }}
              </span>
            </td>

            <!-- Created Date -->
            <td class="date-cell">{{ formatDate(user.created_at) }}</td>

            <!-- Actions Wrapper inside normal td -->
            <td class="td-actions">
              <div class="actions-wrapper">
                <button 
                  class="btn-action btn-edit" 
                  title="Chỉnh sửa thông tin cán bộ"
                  @click="$emit('edit-user', user)"
                >
                  ✏️ Sửa
                </button>

                <button 
                  v-if="user.email !== currentUser.email && user.role !== 'super_admin'"
                  class="btn-action btn-lock"
                  :title="user.is_active ? 'Khóa tài khoản này' : 'Mở khóa tài khoản'"
                  @click="$emit('toggle-status', user)"
                >
                  {{ user.is_active ? '🔒 Khóa' : '🔓 Mở' }}
                </button>

                <button 
                  v-if="user.email !== currentUser.email && user.role !== 'super_admin'"
                  class="btn-action btn-delete"
                  title="Xóa tài khoản khỏi hệ thống"
                  @click="$emit('delete-user', user)"
                >
                  🗑️ Xóa
                </button>
              </div>
            </td>
          </tr>
          <tr v-if="filteredUsers.length === 0">
            <td colspan="7" class="empty-cell">Không tìm thấy cán bộ nào phù hợp</td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>

<script setup>
import { computed } from 'vue';

const props = defineProps({
  users: {
    type: Array,
    default: () => [],
  },
  teams: {
    type: Array,
    default: () => [],
  },
  currentUser: {
    type: Object,
    required: true,
  },
  searchQuery: {
    type: String,
    default: '',
  },
});

defineEmits(['add-user', 'edit-user', 'toggle-status', 'delete-user']);

const filteredUsers = computed(() => {
  if (!props.searchQuery) return props.users;
  const q = props.searchQuery.toLowerCase();
  return props.users.filter(u => 
    u.full_name.toLowerCase().includes(q) || 
    u.email.toLowerCase().includes(q)
  );
});

function getTeamName(teamId) {
  if (!teamId) return 'Toàn hệ thống';
  const t = (props.teams || []).find(item => item.id === teamId);
  return t ? t.name : '—';
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
.users-page {
  display: flex;
  flex-direction: column;
  gap: 16px;
  height: 100%;
}

.users-toolbar {
  background-color: var(--bg-surface);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  padding: 12px 16px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
}

.section-title {
  font-size: 15px;
  font-weight: 700;
  color: var(--text-ink);
  margin-bottom: 2px;
}

.section-sub {
  font-size: 12px;
  color: var(--text-muted);
}

.users-table-container {
  background-color: var(--bg-surface);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  overflow-x: auto;
}

.users-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 12.5px;
  text-align: left;
}

.users-table th {
  background-color: var(--bg-sub);
  color: var(--text-muted);
  font-weight: 700;
  font-size: 11px;
  text-transform: uppercase;
  padding: 10px 14px;
  border-bottom: 1px solid var(--border);
}

.users-table td {
  padding: 10px 14px;
  border-bottom: 1px solid var(--border-sub);
  color: var(--text-ink);
  vertical-align: middle;
}

.users-table tr:hover td {
  background-color: var(--bg-hover);
}

.user-cell {
  display: flex;
  align-items: center;
  gap: 10px;
}

.user-avatar {
  width: 32px;
  height: 32px;
  border-radius: var(--radius-pill);
  background-color: var(--accent-sub);
  color: var(--accent);
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: 700;
  font-size: 13px;
  border: 1px solid var(--accent);
}

.user-cell-info {
  display: flex;
  align-items: center;
  gap: 6px;
}

.user-cell-name {
  font-weight: 600;
}

.self-badge {
  font-size: 11px;
  color: var(--accent);
  font-weight: 700;
}

.email-cell {
  font-family: monospace;
  font-size: 12px;
}

.team-chip {
  display: inline-block;
  padding: 2px 8px;
  font-size: 11.5px;
  border-radius: var(--radius-sm);
  background-color: var(--bg-sub);
  border: 1px solid var(--border);
  color: var(--text-muted);
}

.badge {
  padding: 3px 8px;
  border-radius: var(--radius-pill);
  font-size: 11px;
  font-weight: 700;
  display: inline-block;
}

.role-badge-super {
  background-color: #f3e8ff;
  color: #7e22ce;
  border: 1px solid #c084fc;
}
[data-theme="dark"] .role-badge-super {
  background-color: #3b0764;
  color: #d8b4fe;
  border-color: #9333ea;
}

.role-badge-admin {
  background-color: #e0e7ff;
  color: #3730a3;
}
[data-theme="dark"] .role-badge-admin {
  background-color: #1e1b4b;
  color: #818cf8;
}

.role-badge-user {
  background-color: var(--bg-sub);
  color: var(--text-muted);
  border: 1px solid var(--border);
}

.badge-active {
  background-color: #dcfce7;
  color: #15803d;
}
[data-theme="dark"] .badge-active {
  background-color: #064e3b;
  color: #34d399;
}

.badge-inactive {
  background-color: #fee2e2;
  color: #b91c1c;
}
[data-theme="dark"] .badge-inactive {
  background-color: #450a0a;
  color: #f87171;
}

.date-cell {
  color: var(--text-sub);
  font-size: 11.5px;
}

.actions-wrapper {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 6px;
}

.btn-action {
  padding: 4px 8px;
  font-size: 11.5px;
  font-weight: 600;
  border-radius: var(--radius-sm);
  border: 1px solid var(--border);
  background-color: var(--bg-surface);
  cursor: pointer;
  color: var(--text-ink);
}

.btn-action:hover {
  background-color: var(--bg-hover);
}

.btn-lock:hover {
  background-color: var(--warning-sub);
  color: var(--warning);
}

.btn-delete:hover {
  background-color: var(--danger-sub);
  color: var(--danger);
  border-color: var(--danger);
}

.empty-cell {
  text-align: center;
  padding: 30px;
  color: var(--text-sub);
}
</style>
