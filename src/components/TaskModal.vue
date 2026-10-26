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
          <!-- Main Content (Title, Description, Attachments) -->
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

            <!-- Attachments Section -->
            <div class="form-group attachments-group">
              <div class="attachments-header">
                <div class="att-title-wrap">
                  <span class="att-icon">📎</span>
                  <span class="att-title">Tệp & Tài liệu đính kèm ({{ attachments.length }})</span>
                  <span v-if="uploading" class="uploading-badge">
                    ⏳ {{ uploadProgressText || 'Đang tải lên...' }}
                  </span>
                </div>

                <!-- Add Attachment Button -->
                <div class="att-actions">
                  <button 
                    type="button" 
                    class="btn-attach" 
                    :disabled="uploading"
                    title="Chọn tệp từ máy tính để đính kèm"
                    @click="triggerFileInput"
                  >
                    <span v-if="uploading">⏳ Đang tải...</span>
                    <span v-else>➕ Thêm tệp</span>
                  </button>
                  <input 
                    ref="fileInputRef" 
                    type="file" 
                    multiple 
                    class="hidden-file-input" 
                    :disabled="uploading"
                    @change="handleFileUpload" 
                  />
                </div>
              </div>

              <!-- Storage Unconfigured Notice -->
              <div v-if="!storageConfigured" class="att-alert-box">
                <span class="att-alert-icon">ℹ️</span>
                <div class="att-alert-text">
                  <strong>Tính năng lưu trữ Cloudflare R2 / S3</strong>
                  <p>Vui lòng cấu hình <code>S3_ENDPOINT</code>, <code>S3_ACCESS_KEY_ID</code>, <code>S3_SECRET_ACCESS_KEY</code>, <code>S3_BUCKET</code> trong file <code>.env</code> để đính kèm tệp lên đám mây.</p>
                </div>
              </div>

              <!-- Upload Error Notice -->
              <div v-if="uploadError" class="att-error-box">
                <span>⚠️ {{ uploadError }}</span>
                <button type="button" class="btn-clear-err" @click="uploadError = ''">✕</button>
              </div>

              <!-- Dropzone: ONLY shown when there are 0 files uploaded -->
              <div 
                v-if="attachments.length === 0 && storageConfigured" 
                class="att-dropzone"
                :class="{ 'dragging': isDragging, 'uploading': uploading }"
                @dragover.prevent="isDragging = true"
                @dragleave.prevent="isDragging = false"
                @drop.prevent="handleFileDrop"
                @click="triggerFileInput"
              >
                <div class="dropzone-content">
                  <span class="dropzone-icon">📁</span>
                  <span class="dropzone-text">
                    {{ uploading ? (uploadProgressText || 'Đang tải tệp lên Cloudflare R2...') : 'Kéo & thả tệp vào đây hoặc bấm để chọn tệp' }}
                  </span>
                  <span class="dropzone-sub">(Hỗ trợ PDF, Word, Excel, Hình ảnh, Zip...)</span>
                </div>
              </div>

              <!-- Attachments List: ONLY list of files is shown when files exist (no dropzone text) -->
              <div v-if="attachments.length > 0" class="attachments-list">
                <div 
                  v-for="att in attachments" 
                  :key="att.id" 
                  class="att-item"
                >
                  <div class="att-type-icon" :class="getFileTypeClass(att.file_name, att.content_type)">
                    {{ getFileEmoji(att.file_name, att.content_type) }}
                  </div>

                  <div class="att-details" @click="openAttachment(att)" :title="'Mở xem: ' + att.file_name">
                    <span class="att-name">{{ att.file_name }}</span>
                    <div class="att-meta">
                      <span class="att-size">{{ formatFileSize(att.file_size) }}</span>
                      <span class="att-dot">•</span>
                      <span class="att-uploader" :title="att.uploaded_by">{{ getAssigneeName(att.uploaded_by) }}</span>
                      <span class="att-dot">•</span>
                      <span class="att-date">{{ formatDateTime(att.uploaded_at) }}</span>
                    </div>
                  </div>

                  <div class="att-btns">
                    <button 
                      type="button" 
                      class="btn-att-action btn-att-open" 
                      title="Mở trực tiếp bằng ứng dụng mặc định trên máy"
                      :disabled="openingId === att.id"
                      @click.stop="openAttachment(att)"
                    >
                      {{ openingId === att.id ? 'Đang mở...' : '👁️ Mở' }}
                    </button>

                    <button 
                      type="button" 
                      class="btn-att-action btn-att-download" 
                      title="Mở liên kết tải về trình duyệt"
                      :disabled="downloadingId === att.id"
                      @click.stop="downloadAttachment(att)"
                    >
                      {{ downloadingId === att.id ? '⏳' : '⬇️ Tải' }}
                    </button>

                    <button 
                      v-if="canDeleteAttachment(att)" 
                      type="button" 
                      class="btn-att-action btn-att-del" 
                      title="Xóa tệp đính kèm này"
                      :disabled="deletingId === att.id"
                      @click.stop="deleteAttachment(att)"
                    >
                      {{ deletingId === att.id ? '...' : '✕' }}
                    </button>
                  </div>
                </div>
              </div>
            </div>
          </section>

          <!-- Sidebar (Priority, Status, Due date, Reminder, Assignees) -->
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
                Khi xong việc, chuyển sang <strong>Chờ duyệt</strong> và đính kèm báo cáo/kết quả.
              </span>
            </div>

            <div class="form-group">
              <label class="form-label">Thời hạn hoàn thành</label>
              <template v-if="isAdmin">
                <div class="date-presets">
                  <button type="button" class="preset-btn" @click="setPreset('today')">Hôm nay 17h</button>
                  <button type="button" class="preset-btn" @click="setPreset('tomorrow')">Ngày mai 17h</button>
                  <button type="button" class="preset-btn" @click="setPreset('3days')">3 ngày tới</button>
                  <button type="button" class="preset-btn" @click="setPreset('1week')">1 tuần tới</button>
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
                <!-- Vietnamese Date Preview for Admin -->
                <div v-if="datePart" class="form-hint date-preview-hint">
                  📅 Hạn chót: <strong>{{ formattedDueDateView }}</strong>
                </div>
              </template>
              <div v-else class="readonly-box">
                📅 {{ formattedDueDateView }}
              </div>
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
        <button type="button" class="btn-primary" :disabled="loading || uploading" @click="handleSubmit">
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
import { openPath, openUrl } from '@tauri-apps/plugin-opener';
import RichEditor from './RichEditor.vue';
import { api } from '../api.js';

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

const emit = defineEmits(['close', 'save', 'attachment-changed']);

const isEdit = Boolean(props.taskData && props.taskData.id);
const isAdmin = computed(() => props.currentUser.role === 'admin');
const loading = ref(false);

const datePart = ref('');
const timePart = ref('17:00');

// Dedicated reactive attachments array for real-time reactivity
const attachments = ref([]);
const storageConfigured = ref(true);
const uploading = ref(false);
const uploadProgressText = ref('');
const uploadError = ref('');
const openingId = ref(null);
const downloadingId = ref(null);
const deletingId = ref(null);
const isDragging = ref(false);
const fileInputRef = ref(null);

const form = reactive({
  title: '',
  description: '',
  priority: 'Medium',
  status: 'TODO',
  assigned_to: [],
  remind_before_days: 2,
});

onMounted(async () => {
  storageConfigured.value = await api.isStorageConfigured();

  if (props.taskData) {
    form.title = props.taskData.title || '';
    form.description = props.taskData.description || '';
    form.priority = props.taskData.priority || 'Medium';
    form.status = props.taskData.status || 'TODO';
    const validEmails = new Set((props.users || []).map((u) => u.email));
    form.assigned_to = (props.taskData.assigned_to || []).filter((email) => validEmails.has(email));
    form.remind_before_days = props.taskData.remind_before_days || 2;
    attachments.value = [...(props.taskData.attachments || [])];

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

// Vietnamese formatted due date display (e.g. "17:00 ngày 25/10/2026")
const formattedDueDateView = computed(() => {
  if (!datePart.value) return 'Không có hạn chót';
  const parts = datePart.value.split('-');
  const time = timePart.value || '17:00';
  if (parts.length === 3) {
    const [y, m, d] = parts;
    return `${time} ngày ${d}/${m}/${y}`;
  }
  return `${time} ${datePart.value}`;
});

// Vietnamese formatted reminder hint
const remindHint = computed(() => {
  if (!datePart.value) return 'Chọn hạn chót để tính ngày nhắc.';
  const due = new Date(`${datePart.value}T${timePart.value || '17:00'}:00`);
  due.setDate(due.getDate() - form.remind_before_days);
  const pad = (n) => String(n).padStart(2, '0');
  const d = pad(due.getDate());
  const m = pad(due.getMonth() + 1);
  const y = due.getFullYear();
  const time = timePart.value || '17:00';
  return `Nhắc lúc ${time} ngày ${d}/${m}/${y} (${form.remind_before_days} ngày trước hạn)`;
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

// ==========================================
// ATTACHMENT ACTIONS & REAL-TIME SYNC
// ==========================================

function triggerFileInput() {
  uploadError.value = '';
  fileInputRef.value?.click();
}

function formatFileSize(bytes) {
  if (!bytes || bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
}

function formatDateTime(isoStr) {
  if (!isoStr) return '';
  try {
    const d = new Date(isoStr);
    const pad = (n) => String(n).padStart(2, '0');
    return `${pad(d.getHours())}:${pad(d.getMinutes())} - ${pad(d.getDate())}/${pad(d.getMonth() + 1)}/${d.getFullYear()}`;
  } catch {
    return '';
  }
}

function getFileTypeClass(fileName = '', mime = '') {
  const ext = fileName.split('.').pop().toLowerCase();
  if (['pdf'].includes(ext)) return 'file-pdf';
  if (['doc', 'docx'].includes(ext)) return 'file-word';
  if (['xls', 'xlsx', 'csv'].includes(ext)) return 'file-excel';
  if (['ppt', 'pptx'].includes(ext)) return 'file-ppt';
  if (['png', 'jpg', 'jpeg', 'gif', 'webp', 'svg'].includes(ext) || mime.startsWith('image/')) return 'file-image';
  if (['zip', 'rar', '7z', 'tar', 'gz'].includes(ext)) return 'file-zip';
  return 'file-generic';
}

function getFileEmoji(fileName = '', mime = '') {
  const ext = fileName.split('.').pop().toLowerCase();
  if (['pdf'].includes(ext)) return '📄';
  if (['doc', 'docx'].includes(ext)) return '📝';
  if (['xls', 'xlsx', 'csv'].includes(ext)) return '📊';
  if (['ppt', 'pptx'].includes(ext)) return '📑';
  if (['png', 'jpg', 'jpeg', 'gif', 'webp', 'svg'].includes(ext) || mime.startsWith('image/')) return '🖼️';
  if (['zip', 'rar', '7z', 'tar', 'gz'].includes(ext)) return '📦';
  return '📎';
}

function canDeleteAttachment(att) {
  return isAdmin.value || props.currentUser.email === att.uploaded_by;
}

async function uploadSingleFile(file) {
  const MAX_SIZE = 100 * 1024 * 1024; // 100MB
  if (file.size > MAX_SIZE) {
    throw new Error(`Tệp "${file.name}" vượt quá giới hạn 100MB`);
  }

  uploadProgressText.value = `Đang tải: ${file.name}...`;

  const buffer = await file.arrayBuffer();
  const bytes = new Uint8Array(buffer);

  const taskId = props.taskData?.id || null;
  const newAttachment = await api.uploadAttachment(
    taskId,
    file.name,
    bytes,
    file.type || 'application/octet-stream',
    props.currentUser.email
  );

  // 1. Immediately update local attachments
  attachments.value.push(newAttachment);

  // 2. Sync to props.taskData if it exists
  if (props.taskData) {
    props.taskData.attachments = [...attachments.value];
  }

  // 3. Emit real-time update to parent (App.vue -> Kanban and Table view update instantly)
  emit('attachment-changed', {
    taskId,
    attachments: attachments.value,
  });
}

async function handleFileUpload(e) {
  const files = Array.from(e.target.files || []);
  if (files.length === 0) return;

  uploading.value = true;
  uploadError.value = '';

  try {
    for (const file of files) {
      await uploadSingleFile(file);
    }
  } catch (err) {
    uploadError.value = typeof err === 'string' ? err : (err.message || 'Lỗi tải tệp lên Cloudflare R2');
  } finally {
    uploading.value = false;
    uploadProgressText.value = '';
    if (fileInputRef.value) {
      fileInputRef.value.value = '';
    }
  }
}

async function handleFileDrop(e) {
  isDragging.value = false;
  const files = Array.from(e.dataTransfer?.files || []);
  if (files.length === 0) return;

  uploading.value = true;
  uploadError.value = '';

  try {
    for (const file of files) {
      await uploadSingleFile(file);
    }
  } catch (err) {
    uploadError.value = typeof err === 'string' ? err : (err.message || 'Lỗi tải tệp lên Cloudflare R2');
  } finally {
    uploading.value = false;
    uploadProgressText.value = '';
  }
}

async function openAttachment(att) {
  openingId.value = att.id;
  try {
    uploadError.value = '';
    // Download file to temp cache and open directly via native system viewer (handled by Rust)
    await api.openAttachment(att.object_key, att.file_name);
  } catch (err) {
    console.warn('Native open failed, attempting to open URL in browser:', err);
    try {
      const url = await api.getAttachmentUrl(att.object_key);
      if (url) {
        await openUrl(url);
        return;
      }
    } catch {}
    uploadError.value = `Không thể mở tệp: ${err}`;
  } finally {
    openingId.value = null;
  }
}

async function downloadAttachment(att) {
  downloadingId.value = att.id;
  try {
    uploadError.value = '';
    // Always fetch a fresh presigned URL from backend to ensure it is not expired
    let url = '';
    try {
      url = await api.getAttachmentUrl(att.object_key);
    } catch (urlErr) {
      console.warn('getAttachmentUrl failed:', urlErr);
      url = att.download_url || '';
    }

    if (url) {
      try {
        // Open presigned URL in default browser which triggers download
        await openUrl(url);
      } catch (openErr) {
        console.warn('openUrl failed, falling back to native cache download:', openErr);
        await api.openAttachment(att.object_key, att.file_name);
      }
    } else {
      // Fallback: download to temp and open
      await api.openAttachment(att.object_key, att.file_name);
    }
  } catch (err) {
    console.error('Download error:', err);
    uploadError.value = `Không thể tải tệp: ${err}`;
  } finally {
    downloadingId.value = null;
  }
}

async function deleteAttachment(att) {
  if (!confirm(`Bạn có chắc chắn muốn xóa tệp "${att.file_name}" không?`)) {
    return;
  }

  deletingId.value = att.id;
  try {
    const taskId = props.taskData?.id || null;
    await api.deleteAttachment(taskId, att.id, att.object_key);

    // 1. Immediately update local attachments
    attachments.value = attachments.value.filter((item) => item.id !== att.id);

    // 2. Sync to props.taskData
    if (props.taskData) {
      props.taskData.attachments = [...attachments.value];
    }

    // 3. Emit real-time update to parent App.vue so Kanban and Table view update instantly
    emit('attachment-changed', {
      taskId,
      attachments: attachments.value,
    });
  } catch (err) {
    uploadError.value = `Lỗi xóa tệp: ${err}`;
  } finally {
    deletingId.value = null;
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
      attachments: attachments.value,
    });
  } else {
    loading.value = true;
    emit('save', {
      status: form.status,
      attachments: attachments.value,
    });
  }
}
</script>

<style scoped>
.ticket-modal {
  max-width: 1040px;
  width: min(1040px, 96vw);
  height: min(780px, 92vh);
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
  overflow-y: auto;
  overflow-x: hidden;
  scrollbar-width: thin;
  scrollbar-color: var(--border) transparent;
}

.ticket-main::-webkit-scrollbar {
  width: 6px;
}
.ticket-main::-webkit-scrollbar-track {
  background: transparent;
}
.ticket-main::-webkit-scrollbar-thumb {
  background-color: var(--border);
  border-radius: var(--radius-pill);
}
.ticket-main::-webkit-scrollbar-thumb:hover {
  background-color: var(--text-sub);
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
  min-height: 420px;
  height: 440px;
  display: flex;
  flex-direction: column;
  margin-bottom: 16px;
  flex-shrink: 0;
}

.editor-group :deep(.rich-editor) {
  height: 100%;
  flex: 1;
  min-height: 400px;
}

/* ==========================================
   ATTACHMENTS STYLING
========================================== */
.attachments-group {
  margin-top: 4px;
  border-top: 1px solid var(--border-sub);
  padding-top: 14px;
  flex-shrink: 0;
}

.attachments-header {
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
  font-size: 15px;
}

.att-title {
  font-size: 13px;
  font-weight: 700;
  color: var(--text-ink);
}

.uploading-badge {
  font-size: 11px;
  color: var(--accent);
  background-color: var(--accent-sub);
  padding: 2px 7px;
  border-radius: var(--radius-sm);
  margin-left: 6px;
}

.hidden-file-input {
  display: none;
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

.att-alert-box {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  background-color: var(--bg-sub);
  border: 1px dashed var(--border);
  border-radius: var(--radius-md);
  padding: 10px 12px;
  font-size: 12px;
  color: var(--text-muted);
}
.att-alert-icon {
  font-size: 16px;
  flex-shrink: 0;
}
.att-alert-text p {
  margin-top: 3px;
  line-height: 1.45;
  color: var(--text-sub);
}
.att-alert-text code {
  background: var(--bg-hover);
  padding: 1px 4px;
  border-radius: 3px;
  font-size: 11px;
}

.att-error-box {
  display: flex;
  align-items: center;
  justify-content: space-between;
  background-color: var(--danger-sub);
  color: var(--danger);
  border: 1px solid var(--danger);
  border-radius: var(--radius-sm);
  padding: 6px 10px;
  font-size: 12px;
  margin-bottom: 8px;
}
.btn-clear-err {
  background: none;
  border: none;
  color: var(--danger);
  cursor: pointer;
  font-weight: bold;
}

.att-dropzone {
  border: 1px dashed var(--border);
  border-radius: var(--radius-md);
  padding: 12px 14px;
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
  max-height: 200px;
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

.att-details {
  flex: 1;
  min-width: 0;
  cursor: pointer;
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
.att-details:hover .att-name {
  color: var(--accent);
  text-decoration: underline;
}

.att-meta {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 11px;
  color: var(--text-sub);
  margin-top: 2px;
}
.att-dot {
  opacity: 0.5;
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
.btn-att-action:hover {
  background: var(--bg-hover);
  color: var(--text-ink);
}
.btn-att-open {
  font-weight: 600;
  color: var(--accent);
  border-color: var(--accent);
}
.btn-att-open:hover {
  background: var(--accent);
  color: #fff;
}
.btn-att-download {
  color: #059669;
  border-color: #a7f3d0;
  background-color: #ecfdf5;
}
.btn-att-download:hover {
  background-color: #10b981;
  color: #fff;
  border-color: #10b981;
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

/* ==========================================
   SIDEBAR STYLING
========================================== */
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
  cursor: pointer;
}
.preset-btn:hover {
  background-color: var(--bg-hover);
}

.datetime-inputs {
  display: flex;
  gap: 8px;
}

.datetime-field {
  position: relative;
  flex: 1;
}

.datetime-icon {
  position: absolute;
  right: 8px;
  top: 50%;
  transform: translateY(-50%);
  pointer-events: none;
  color: var(--text-sub);
}

.date-preview-hint {
  margin-top: 6px;
  color: var(--accent);
  font-size: 12px;
  background-color: var(--accent-sub);
  border: 1px solid rgba(var(--accent-rgb, 14, 165, 233), 0.2);
  padding: 5px 9px;
  border-radius: var(--radius-sm);
  display: flex;
  align-items: center;
  gap: 4px;
}

.remind-btns {
  display: flex;
  gap: 6px;
  margin-bottom: 4px;
}

.remind-btn {
  flex: 1;
  padding: 6px 4px;
  font-size: 12px;
  font-weight: 600;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--bg-surface);
  color: var(--text-muted);
  cursor: pointer;
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
  font-size: 13px;
  color: var(--text-ink);
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
