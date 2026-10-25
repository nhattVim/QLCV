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
            <th style="width: 220px;">Cán bộ</th>
            <th style="min-width: 220px;">Email</th>
            <th style="width: 140px;">Vai trò</th>
            <th style="width: 130px;">Trạng thái</th>
            <th style="width: 130px;">Ngày tham gia</th>
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

            <!-- Role Badge -->
            <td>
              <span class="badge" :class="user.role === 'admin' ? 'role-badge-admin' : 'role-badge-user'">
                {{ user.role === 'admin' ? '👑 Quản trị viên' : '👤 Cán bộ' }}
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
                  v-if="user.email !== currentUser.email"
                  class="btn-action btn-lock"
                  :title="user.is_active ? 'Khóa tài khoản này' : 'Mở khóa tài khoản'"
                  @click="$emit('toggle-status', user)"
                >
                  {{ user.is_active ? '🔒 Khóa' : '🔓 Mở' }}
                </button>

                <button 
                  v-if="user.email !== currentUser.email"
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
            <td colspan="6" class="empty-cell">Không tìm thấy cán bộ nào phù hợp</td>
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
  gap: 14px;
  padding-bottom: 32px;
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
  font-size: 14.5px;
  font-weight: 800;
  color: var(--text-ink);
}

.section-sub {
  font-size: 12px;
  color: var(--text-sub);
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
  text-align: left;
  font-size: 13px;
  table-layout: auto;
}

.users-table th {
  background-color: var(--bg-sub);
  color: var(--text-sub);
  font-weight: 700;
  font-size: 11px;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  padding: 10px 14px;
  border-bottom: 1px solid var(--border);
  white-space: nowrap;
}

.users-table td {
  padding: 10px 14px;
  border-bottom: 1px solid var(--border-sub);
  color: var(--text-ink);
  vertical-align: middle;
}

.users-table tr:last-child td {
  border-bottom: none;
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
  width: 28px;
  height: 28px;
  border-radius: 50%;
  background-color: var(--accent);
  color: #fff;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 11.5px;
  font-weight: 700;
  flex-shrink: 0;
}

.user-cell-info {
  display: flex;
  align-items: center;
  gap: 6px;
}

.user-cell-name {
  font-weight: 600;
  white-space: nowrap;
}

.self-badge {
  font-size: 10px;
  font-weight: 700;
  color: var(--accent);
  background-color: var(--accent-sub);
  padding: 1px 5px;
  border-radius: var(--radius-pill);
}

.email-cell {
  color: var(--text-muted);
}

.date-cell {
  color: var(--text-sub);
  font-size: 12px;
  white-space: nowrap;
}

.role-badge-admin {
  background-color: #fee2e2;
  color: #dc2626;
  border-color: #fca5a5;
}
[data-theme="dark"] .role-badge-admin {
  background-color: #450a0a;
  color: #f87171;
}

.role-badge-user {
  background-color: #dbeafe;
  color: #2563eb;
  border-color: #93c5fd;
}
[data-theme="dark"] .role-badge-user {
  background-color: #172554;
  color: #60a5fa;
}

.badge-active {
  background-color: #d1fae5;
  color: #059669;
  border-color: #a7f3d0;
}
[data-theme="dark"] .badge-active {
  background-color: #064e3b;
  color: #34d399;
}

.badge-inactive {
  background-color: #fee2e2;
  color: #dc2626;
  border-color: #fca5a5;
}
[data-theme="dark"] .badge-inactive {
  background-color: #450a0a;
  color: #f87171;
}

.td-actions {
  text-align: right;
  white-space: nowrap;
}

.actions-wrapper {
  display: inline-flex;
  align-items: center;
  justify-content: flex-end;
  gap: 6px;
}

.btn-action {
  padding: 4px 8px;
  font-size: 11.5px;
  font-weight: 500;
  border-radius: var(--radius-sm);
  border: 1px solid var(--border);
  background-color: var(--bg-surface);
  color: var(--text-ink);
  transition: all 0.12s ease;
}
.btn-action:hover {
  background-color: var(--bg-hover);
  border-color: var(--text-sub);
}

.btn-delete {
  color: var(--danger);
  border-color: var(--danger);
  background-color: var(--danger-sub);
}
.btn-delete:hover {
  background-color: var(--danger);
  color: #fff;
}

.empty-cell {
  text-align: center;
  padding: 30px;
  color: var(--text-sub);
}
</style>
