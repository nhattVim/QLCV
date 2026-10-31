<template>
  <div class="team-settings-container">
    <div class="settings-header">
      <div class="header-info">
        <h2 class="settings-title">⚙️ Cài đặt & Tùy biến Đội nhóm</h2>
        <p class="settings-subtitle">
          Cấu hình quy trình làm việc Kanban, bước phê duyệt và theo dõi kết quả thực hiện cho đội nhóm: 
          <strong>{{ activeTeam?.name || 'Chưa chọn team' }}</strong>
        </p>
      </div>
    </div>

    <form class="settings-body" @submit.prevent="handleSave">
      <!-- 1. General Team Info Card -->
      <div class="settings-card">
        <div class="card-head">
          <span class="card-icon">🏢</span>
          <div>
            <h3 class="card-title">Thông tin Đội nhóm</h3>
            <span class="card-desc">Tên hiển thị và mô tả hoạt động của đội ngũ</span>
          </div>
        </div>

        <div class="form-fields">
          <div class="form-field">
            <label class="field-label">Tên đội nhóm <span class="required">*</span></label>
            <input 
              v-model="form.name" 
              type="text" 
              class="form-input" 
              placeholder="Ví dụ: Phòng Kỹ Thuật & Giải Pháp..." 
              required
            />
          </div>

          <div class="form-field">
            <label class="field-label">Mô tả đội nhóm</label>
            <textarea 
              v-model="form.description" 
              class="form-textarea" 
              rows="2" 
              placeholder="Chức năng, mục tiêu hoạt động của đội ngũ..."
            ></textarea>
          </div>
        </div>
      </div>

      <!-- 2. Kanban Workflow Configuration Card (Cần duyệt hay không cần duyệt) -->
      <div class="settings-card">
        <div class="card-head">
          <span class="card-icon">📋</span>
          <div>
            <h3 class="card-title">Quy trình Kanban & Phê duyệt Công việc</h3>
            <span class="card-desc">Linh động bật/tắt bước Chờ duyệt tùy theo đặc thù quản lý của từng đội nhóm</span>
          </div>
        </div>

        <div class="toggle-setting-row">
          <div class="toggle-info">
            <div class="toggle-title-wrap">
              <strong class="toggle-title">Bao gồm bước Phê duyệt (Chờ duyệt / REVIEW)</strong>
              <span 
                class="badge-pill" 
                :class="form.enable_review_step ? 'pill-active' : 'pill-inactive'"
              >
                {{ form.enable_review_step ? '4 Cột Kanban (Có duyệt)' : '3 Cột Kanban (Không cần duyệt)' }}
              </span>
            </div>
            <p class="toggle-explain">
              <template v-if="form.enable_review_step">
                ● <strong>Đang BẬT:</strong> Khi cán bộ hoàn tất nhiệm vụ, công việc sẽ chuyển sang cột <strong>"Chờ duyệt"</strong>. 
                Chỉ Quản trị viên mới có thẩm quyền thẩm định và bấm Phê duyệt Hoàn thành.
              </template>
              <template v-else>
                ● <strong>Đang TẮT:</strong> Bỏ qua bước kiểm duyệt. Cột <strong>"Chờ duyệt"</strong> sẽ được tự động ẩn khỏi bảng Kanban. 
                Cán bộ thực hiện có thể tự đánh dấu hoàn thành nhiệm vụ ngay khi làm xong.
              </template>
            </p>
          </div>

          <!-- Switch Toggle -->
          <label class="switch-toggle" title="Bật/Tắt bước phê duyệt">
            <input v-model="form.enable_review_step" type="checkbox" />
            <span class="slider"></span>
          </label>
        </div>

        <!-- Workflow Visual Diagram -->
        <div class="workflow-preview">
          <span class="preview-label">Mô hình cột Kanban hiển thị:</span>
          <div class="pipeline-track">
            <div class="pipe-node pipe-todo">
              <span class="pipe-icon">📋</span>
              <span class="pipe-name">Chưa làm</span>
            </div>
            <span class="pipe-arrow">➔</span>
            <div class="pipe-node pipe-progress">
              <span class="pipe-icon">⚡</span>
              <span class="pipe-name">Đang thực hiện</span>
            </div>
            <template v-if="form.enable_review_step">
              <span class="pipe-arrow">➔</span>
              <div class="pipe-node pipe-review">
                <span class="pipe-icon">🔍</span>
                <span class="pipe-name">Chờ duyệt</span>
              </div>
            </template>
            <span class="pipe-arrow">➔</span>
            <div class="pipe-node pipe-done">
              <span class="pipe-icon">✅</span>
              <span class="pipe-name">Hoàn thành</span>
            </div>
          </div>
        </div>
      </div>

      <!-- 3. Result Tracking Configuration Card (Cột Kết quả khi hoàn thành) -->
      <div class="settings-card">
        <div class="card-head">
          <span class="card-icon">🎯</span>
          <div>
            <h3 class="card-title">Theo dõi Kết quả Thực hiện (Cột Kết Quả)</h3>
            <span class="card-desc">Cán bộ tự điền kết quả / sản phẩm đạt được khi hoàn thành công việc</span>
          </div>
        </div>

        <div class="toggle-setting-row">
          <div class="toggle-info">
            <div class="toggle-title-wrap">
              <strong class="toggle-title">Kích hoạt Ghi nhận & Báo cáo Cột Kết quả</strong>
              <span 
                class="badge-pill" 
                :class="form.enable_result_column ? 'pill-active' : 'pill-inactive'"
              >
                {{ form.enable_result_column ? 'Đang kích hoạt' : 'Không bắt buộc' }}
              </span>
            </div>
            <p class="toggle-explain">
              <template v-if="form.enable_result_column">
                ● <strong>Đang BẬT:</strong> Khi cán bộ hoàn thành công việc, hệ thống sẽ mở hộp thoại để cán bộ nhập 
                tóm tắt kết quả, sản phẩm nghiệm thu hoặc văn bản bàn giao. Kết quả này sẽ hiển thị ở cột <strong>"KẾT QUẢ"</strong> 
                trên bảng Báo cáo tiến độ và xuất ra file Excel.
              </template>
              <template v-else>
                ● <strong>Đang TẮT:</strong> Không bắt buộc cán bộ nhập kết quả khi hoàn thành công việc.
              </template>
            </p>
          </div>

          <!-- Switch Toggle -->
          <label class="switch-toggle" title="Bật/Tắt ghi nhận kết quả">
            <input v-model="form.enable_result_column" type="checkbox" />
            <span class="slider"></span>
          </label>
        </div>
      </div>

      <!-- Bottom Save Action Bar -->
      <div class="settings-footer">
        <span v-if="hasChanges" class="unsaved-hint">
          ⚠️ Có thay đổi cài đặt chưa được lưu. Hãy bấm "Lưu Cài Đặt" để áp dụng ngay.
        </span>
        <button 
          type="submit" 
          class="btn-save-settings" 
          :disabled="isSaving"
        >
          <span v-if="isSaving">⏳ Đang lưu...</span>
          <span v-else>💾 Lưu Cài Đặt Đội Nhóm</span>
        </button>
      </div>
    </form>
  </div>
</template>

<script setup>
import { ref, computed, watch, onMounted } from 'vue';

const props = defineProps({
  currentTeam: {
    type: Object,
    default: null,
  },
  teams: {
    type: Array,
    default: () => [],
  },
  currentUser: {
    type: Object,
    required: true,
  },
  isSuperAdmin: {
    type: Boolean,
    default: false,
  },
});

const emit = defineEmits(['update-team-settings', 'select-team']);

const selectedTeamId = ref('');
const isSaving = ref(false);

const form = ref({
  name: '',
  description: '',
  enable_review_step: true,
  enable_result_column: true,
});

const originalState = ref('');

const activeTeam = computed(() => {
  if (selectedTeamId.value) {
    const found = props.teams.find(t => t.id === selectedTeamId.value);
    if (found) return found;
  }
  return props.currentTeam;
});

function syncFromTeam(team) {
  if (!team) return;
  form.value.name = team.name || '';
  form.value.description = team.description || '';
  form.value.enable_review_step = team.enable_review_step !== false;
  form.value.enable_result_column = team.enable_result_column !== false;
  originalState.value = JSON.stringify(form.value);
}

onMounted(() => {
  if (props.currentTeam) {
    selectedTeamId.value = props.currentTeam.id || '';
    syncFromTeam(props.currentTeam);
  } else if (props.teams && props.teams.length > 0) {
    selectedTeamId.value = props.teams[0].id || '';
    syncFromTeam(props.teams[0]);
  }
});

watch(
  () => props.currentTeam,
  (newTeam) => {
    if (newTeam) {
      selectedTeamId.value = newTeam.id || '';
      syncFromTeam(newTeam);
    }
  },
  { deep: true }
);

watch(
  () => props.teams,
  (newTeams) => {
    if (newTeams && newTeams.length > 0 && !selectedTeamId.value) {
      selectedTeamId.value = props.currentTeam?.id || newTeams[0]?.id || '';
      const t = newTeams.find(team => team.id === selectedTeamId.value) || newTeams[0];
      syncFromTeam(t);
    }
  },
  { deep: true }
);

function onTeamSelect() {
  const team = props.teams.find(t => t.id === selectedTeamId.value);
  if (team) {
    syncFromTeam(team);
    emit('select-team', team.id);
  }
}

const hasChanges = computed(() => {
  return JSON.stringify(form.value) !== originalState.value;
});

async function handleSave() {
  if (!activeTeam.value?.id) return;
  isSaving.value = true;
  try {
    await emit('update-team-settings', {
      id: activeTeam.value.id,
      name: form.value.name.trim(),
      description: form.value.description.trim(),
      enable_review_step: form.value.enable_review_step,
      enable_result_column: form.value.enable_result_column,
    });
    originalState.value = JSON.stringify(form.value);
  } finally {
    isSaving.value = false;
  }
}
</script>

<style scoped>
.team-settings-container {
  display: flex;
  flex-direction: column;
  gap: 18px;
  max-width: 960px;
  margin: 0 auto;
  padding: 12px 16px 40px;
}

.settings-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
  flex-wrap: wrap;
  padding-bottom: 12px;
  border-bottom: 1px solid var(--border);
}

.settings-title {
  font-size: 20px;
  font-weight: 700;
  color: var(--text-ink);
  margin-bottom: 4px;
}

.settings-subtitle {
  font-size: 13.5px;
  color: var(--text-sub);
}

.team-switcher-box {
  display: flex;
  align-items: center;
  gap: 8px;
  background-color: var(--bg-surface);
  border: 1px solid var(--border);
  padding: 6px 12px;
  border-radius: var(--radius-sm);
}

.switcher-label {
  font-size: 12.5px;
  font-weight: 600;
  color: var(--text-muted);
}

.switcher-select {
  padding: 4px 8px;
  border-radius: 4px;
  border: 1px solid var(--border);
  background-color: var(--bg-sub);
  color: var(--text-ink);
  font-weight: 600;
  font-size: 13px;
}

.settings-body {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.settings-card {
  background-color: var(--bg-surface);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  padding: 20px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.05);
}

.card-head {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 16px;
  padding-bottom: 12px;
  border-bottom: 1px solid var(--border-light, rgba(0,0,0,0.06));
}

.card-icon {
  font-size: 24px;
}

.card-title {
  font-size: 16px;
  font-weight: 700;
  color: var(--text-ink);
  margin-bottom: 2px;
}

.card-desc {
  font-size: 12.5px;
  color: var(--text-sub);
}

.form-fields {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.form-field {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.field-label {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-muted);
}

.required {
  color: var(--danger, #dc2626);
}

.form-input, .form-textarea {
  width: 100%;
  padding: 8px 12px;
  border-radius: var(--radius-sm);
  border: 1px solid var(--border);
  background-color: var(--bg-surface);
  color: var(--text-ink);
  font-size: 13.5px;
  transition: border-color 0.15s;
}

.form-input:focus, .form-textarea:focus {
  border-color: var(--accent);
  outline: none;
}

/* Toggle row */
.toggle-setting-row {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 20px;
  padding: 12px 0;
}

.toggle-info {
  flex: 1;
}

.toggle-title-wrap {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 6px;
  flex-wrap: wrap;
}

.toggle-title {
  font-size: 14.5px;
  color: var(--text-ink);
}

.badge-pill {
  font-size: 11.5px;
  font-weight: 700;
  padding: 2px 8px;
  border-radius: 12px;
}

.pill-active {
  background-color: rgba(22, 163, 74, 0.12);
  color: #16a34a;
  border: 1px solid rgba(22, 163, 74, 0.3);
}

.pill-inactive {
  background-color: rgba(100, 116, 139, 0.12);
  color: #64748b;
  border: 1px solid rgba(100, 116, 139, 0.3);
}

.toggle-explain {
  font-size: 13px;
  color: var(--text-sub);
  line-height: 1.5;
  margin: 0;
}

/* Switch */
.switch-toggle {
  position: relative;
  display: inline-block;
  width: 52px;
  height: 28px;
  flex-shrink: 0;
}

.switch-toggle input {
  opacity: 0;
  width: 0;
  height: 0;
}

.slider {
  position: absolute;
  cursor: pointer;
  inset: 0;
  background-color: var(--border, #cbd5e1);
  transition: 0.25s;
  border-radius: 28px;
}

.slider:before {
  position: absolute;
  content: "";
  height: 22px;
  width: 22px;
  left: 3px;
  bottom: 3px;
  background-color: white;
  transition: 0.25s;
  border-radius: 50%;
  box-shadow: 0 1px 3px rgba(0,0,0,0.25);
}

input:checked + .slider {
  background-color: var(--accent, #2563eb);
}

input:checked + .slider:before {
  transform: translateX(24px);
}

/* Workflow visual diagram */
.workflow-preview {
  margin-top: 14px;
  padding: 14px;
  background-color: var(--bg-sub);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
}

.preview-label {
  display: block;
  font-size: 12px;
  font-weight: 600;
  color: var(--text-muted);
  margin-bottom: 10px;
}

.pipeline-track {
  display: flex;
  align-items: center;
  gap: 10px;
  overflow-x: auto;
  padding-bottom: 4px;
}

.pipe-node {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 14px;
  border-radius: 6px;
  font-size: 13px;
  font-weight: 600;
  border: 1px solid var(--border);
  background-color: var(--bg-surface);
  white-space: nowrap;
}

.pipe-todo {
  border-color: #94a3b8;
  color: var(--text-ink);
}

.pipe-progress {
  border-color: #3b82f6;
  color: #2563eb;
  background-color: rgba(37, 99, 235, 0.05);
}

.pipe-review {
  border-color: #f59e0b;
  color: #d97706;
  background-color: rgba(217, 119, 6, 0.05);
}

.pipe-done {
  border-color: #10b981;
  color: #059669;
  background-color: rgba(16, 185, 129, 0.05);
}

.pipe-arrow {
  color: var(--text-muted);
  font-size: 14px;
  font-weight: bold;
}

/* Footer Action Bar */
.settings-footer {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 16px;
  padding-top: 10px;
}

.unsaved-hint {
  font-size: 13px;
  color: #d97706;
  font-weight: 600;
}

.btn-save-settings {
  padding: 10px 24px;
  border-radius: var(--radius-sm);
  background-color: var(--accent, #2563eb);
  color: white;
  font-size: 14px;
  font-weight: 700;
  border: none;
  cursor: pointer;
  box-shadow: 0 2px 4px rgba(37, 99, 235, 0.25);
  transition: background-color 0.15s, transform 0.1s;
}

.btn-save-settings:hover:not(:disabled) {
  background-color: var(--accent-hover, #1d4ed8);
  transform: translateY(-1px);
}

.btn-save-settings:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}
</style>

