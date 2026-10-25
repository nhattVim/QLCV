<template>
  <div class="modal-overlay" @click.self="$emit('close')">
    <div class="modal-content">
      <!-- Modal Header -->
      <div class="modal-header">
        <h3 class="modal-title">
          {{ isEdit ? '✏️ Chỉnh sửa thông tin cán bộ' : '👤 Thêm mới cán bộ' }}
        </h3>
        <button class="btn-ghost" @click="$emit('close')">✕</button>
      </div>

      <!-- Modal Body -->
      <form class="modal-body" @submit.prevent="handleSubmit">
        <div class="form-group">
          <label class="form-label">Họ và tên cán bộ <span class="required">*</span></label>
          <input 
            v-model="form.full_name" 
            type="text" 
            class="form-control" 
            placeholder="Ví dụ: Nguyễn Văn An"
            required 
            autofocus
          />
        </div>

        <div class="form-group">
          <label class="form-label">Email tài khoản <span class="required">*</span></label>
          <input 
            v-model="form.email" 
            type="email" 
            class="form-control" 
            placeholder="an.nguyen@qlcv.com"
            :disabled="isEdit"
            required 
          />
          <span v-if="isEdit" class="form-hint">Email tài khoản không thể thay đổi sau khi tạo</span>
        </div>

        <div class="form-group">
          <label class="form-label">
            Mật khẩu đăng nhập 
            <span v-if="!isEdit" class="required">*</span>
            <span v-else class="form-hint">(Để trống nếu không muốn đổi mật khẩu)</span>
          </label>
          <input 
            v-model="form.password" 
            type="password" 
            class="form-control" 
            :placeholder="isEdit ? '••••••••' : 'Nhập mật khẩu ban đầu...'"
            :required="!isEdit"
          />
        </div>

        <div class="form-row">
          <div class="form-group flex-1">
            <label class="form-label">Phân quyền vai trò</label>
            <select v-model="form.role" class="form-control">
              <option value="user">Cán bộ chuyên viên</option>
              <option value="admin">Quản trị viên hệ thống</option>
            </select>
          </div>

          <div v-if="isEdit" class="form-group flex-1">
            <label class="form-label">Trạng thái hoạt động</label>
            <select v-model="form.is_active" class="form-control">
              <option :value="true">🟢 Đang hoạt động</option>
              <option :value="false">🔴 Đã khóa truy cập</option>
            </select>
          </div>
        </div>
      </form>

      <!-- Modal Footer -->
      <div class="modal-footer">
        <button type="button" class="btn-secondary" @click="$emit('close')">Hủy bỏ</button>
        <button type="button" class="btn-primary" :disabled="loading" @click="handleSubmit">
          {{ loading ? 'Đang lưu...' : (isEdit ? 'Lưu thay đổi' : 'Thêm cán bộ') }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup>
import { reactive, ref, onMounted } from 'vue';

const props = defineProps({
  userData: {
    type: Object,
    default: null,
  },
});

const emit = defineEmits(['close', 'save']);

const isEdit = Boolean(props.userData && props.userData.email);
const loading = ref(false);

const form = reactive({
  full_name: '',
  email: '',
  password: '',
  role: 'user',
  is_active: true,
});

onMounted(() => {
  if (props.userData) {
    form.full_name = props.userData.full_name || '';
    form.email = props.userData.email || '';
    form.role = props.userData.role || 'user';
    form.is_active = props.userData.is_active !== false;
    form.password = '';
  }
});

function handleSubmit() {
  if (!form.full_name.trim() || !form.email.trim()) return;
  if (!isEdit && !form.password) return;

  loading.value = true;
  emit('save', { ...form });
}
</script>

<style scoped>
.required {
  color: var(--danger);
}

.form-row {
  display: flex;
  gap: 12px;
}

.flex-1 {
  flex: 1;
}

.form-hint {
  font-size: 11px;
  color: var(--text-sub);
}
</style>
