<template>
  <div class="rich-editor" :class="{ readonly }">
    <div v-if="!readonly && editor" class="rich-toolbar">
      <button type="button" class="rich-btn" title="Hoàn tác" @mousedown.prevent="run((e) => e.undo())">
        <Undo :size="15" :stroke-width="2" />
      </button>
      <button type="button" class="rich-btn" title="Làm lại" @mousedown.prevent="run((e) => e.redo())">
        <Redo :size="15" :stroke-width="2" />
      </button>

      <span class="rich-sep"></span>

      <select class="rich-select" :value="headingValue" title="Kiểu đoạn" @mousedown.stop @change="onHeadingChange">
        <option value="p">Đoạn văn</option>
        <option value="1">Tiêu đề 1</option>
        <option value="2">Tiêu đề 2</option>
        <option value="3">Tiêu đề 3</option>
      </select>

      <span class="rich-sep"></span>

      <button type="button" class="rich-btn" :class="{ active: editor.isActive('bold') }" title="In đậm" @mousedown.prevent="run((e) => e.toggleBold())">
        <Bold :size="15" :stroke-width="2.4" />
      </button>
      <button type="button" class="rich-btn" :class="{ active: editor.isActive('italic') }" title="In nghiêng" @mousedown.prevent="run((e) => e.toggleItalic())">
        <Italic :size="15" :stroke-width="2.4" />
      </button>
      <button type="button" class="rich-btn" :class="{ active: editor.isActive('underline') }" title="Gạch dưới" @mousedown.prevent="run((e) => e.toggleUnderline())">
        <Underline :size="15" :stroke-width="2.4" />
      </button>
      <button type="button" class="rich-btn" :class="{ active: editor.isActive('strike') }" title="Gạch ngang" @mousedown.prevent="run((e) => e.toggleStrike())">
        <Strikethrough :size="15" :stroke-width="2.4" />
      </button>
      <button type="button" class="rich-btn" :class="{ active: editor.isActive('code') }" title="Mã inline" @mousedown.prevent="run((e) => e.toggleCode())">
        <Code :size="15" :stroke-width="2.4" />
      </button>

      <label class="rich-btn color-btn" title="Màu chữ">
        <Palette :size="15" :stroke-width="2" />
        <input type="color" :value="currentColor" @input="onColorInput" />
      </label>
      <button type="button" class="rich-btn" :class="{ active: editor.isActive('highlight') }" title="Tô màu nền" @mousedown.prevent="run((e) => e.toggleHighlight())">
        <Highlighter :size="15" :stroke-width="2" />
      </button>

      <span class="rich-sep"></span>

      <button type="button" class="rich-btn" :class="{ active: editor.isActive({ textAlign: 'left' }) }" title="Căn trái" @mousedown.prevent="run((e) => e.setTextAlign('left'))">
        <AlignLeft :size="15" :stroke-width="2" />
      </button>
      <button type="button" class="rich-btn" :class="{ active: editor.isActive({ textAlign: 'center' }) }" title="Căn giữa" @mousedown.prevent="run((e) => e.setTextAlign('center'))">
        <AlignCenter :size="15" :stroke-width="2" />
      </button>
      <button type="button" class="rich-btn" :class="{ active: editor.isActive({ textAlign: 'right' }) }" title="Căn phải" @mousedown.prevent="run((e) => e.setTextAlign('right'))">
        <AlignRight :size="15" :stroke-width="2" />
      </button>
      <button type="button" class="rich-btn" :class="{ active: editor.isActive({ textAlign: 'justify' }) }" title="Căn đều" @mousedown.prevent="run((e) => e.setTextAlign('justify'))">
        <AlignJustify :size="15" :stroke-width="2" />
      </button>

      <span class="rich-sep"></span>

      <button type="button" class="rich-btn" :class="{ active: editor.isActive('bulletList') }" title="Danh sách" @mousedown.prevent="run((e) => e.toggleBulletList())">
        <List :size="15" :stroke-width="2" />
      </button>
      <button type="button" class="rich-btn" :class="{ active: editor.isActive('orderedList') }" title="Đánh số" @mousedown.prevent="run((e) => e.toggleOrderedList())">
        <ListOrdered :size="15" :stroke-width="2" />
      </button>
      <button type="button" class="rich-btn" :class="{ active: editor.isActive('taskList') }" title="Checklist" @mousedown.prevent="run((e) => e.toggleTaskList())">
        <ListChecks :size="15" :stroke-width="2" />
      </button>
      <button type="button" class="rich-btn" :class="{ active: editor.isActive('blockquote') }" title="Trích dẫn" @mousedown.prevent="run((e) => e.toggleBlockquote())">
        <Quote :size="15" :stroke-width="2" />
      </button>
      <button type="button" class="rich-btn" title="Đường kẻ ngang" @mousedown.prevent="run((e) => e.setHorizontalRule())">
        <Minus :size="15" :stroke-width="2" />
      </button>

      <span class="rich-sep"></span>

      <button type="button" class="rich-btn" :class="{ active: editor.isActive('link') }" title="Chèn liên kết" @mousedown.prevent="setLink">
        <LinkIcon :size="15" :stroke-width="2" />
      </button>
      <button type="button" class="rich-btn" title="Chèn ảnh (URL)" @mousedown.prevent="setImage">
        <ImageIcon :size="15" :stroke-width="2" />
      </button>
      <button type="button" class="rich-btn" title="Chèn bảng" @mousedown.prevent="run((e) => e.insertTable({ rows: 3, cols: 3, withHeaderRow: true }))">
        <TableIcon :size="15" :stroke-width="2" />
      </button>
      <template v-if="editor.isActive('table')">
        <button type="button" class="rich-btn" title="Thêm cột" @mousedown.prevent="run((e) => e.addColumnAfter())">+C</button>
        <button type="button" class="rich-btn" title="Thêm hàng" @mousedown.prevent="run((e) => e.addRowAfter())">+H</button>
        <button type="button" class="rich-btn" title="Xóa bảng" @mousedown.prevent="run((e) => e.deleteTable())">
          <Trash2 :size="15" :stroke-width="2" />
        </button>
      </template>

      <span class="rich-sep"></span>

      <button type="button" class="rich-btn" title="Xóa định dạng" @mousedown.prevent="run((e) => e.unsetAllMarks().clearNodes())">
        <RemoveFormatting :size="15" :stroke-width="2" />
      </button>
    </div>

    <editor-content :editor="editor" class="rich-surface" />
  </div>
</template>

<script setup>
import { computed, watch } from 'vue';
import { EditorContent, useEditor } from '@tiptap/vue-3';
import StarterKit from '@tiptap/starter-kit';
import Placeholder from '@tiptap/extension-placeholder';
import TextAlign from '@tiptap/extension-text-align';
import { TextStyle, Color } from '@tiptap/extension-text-style';
import Highlight from '@tiptap/extension-highlight';
import Image from '@tiptap/extension-image';
import { TableKit } from '@tiptap/extension-table';
import TaskList from '@tiptap/extension-task-list';
import TaskItem from '@tiptap/extension-task-item';
import {
  AlignCenter,
  AlignJustify,
  AlignLeft,
  AlignRight,
  Bold,
  Code,
  Highlighter,
  Image as ImageIcon,
  Italic,
  Link as LinkIcon,
  List,
  ListChecks,
  ListOrdered,
  Minus,
  Palette,
  Quote,
  Redo,
  RemoveFormatting,
  Strikethrough,
  Table as TableIcon,
  Trash2,
  Underline,
  Undo,
} from 'lucide-vue-next';

const props = defineProps({
  modelValue: {
    type: String,
    default: '',
  },
  readonly: {
    type: Boolean,
    default: false,
  },
  placeholder: {
    type: String,
    default: 'Nhập nội dung chi tiết...',
  },
});

const emit = defineEmits(['update:modelValue']);

const editor = useEditor({
  content: props.modelValue || '',
  editable: !props.readonly,
  extensions: [
    StarterKit.configure({
      heading: { levels: [1, 2, 3] },
    }),
    Placeholder.configure({
      placeholder: props.placeholder,
    }),
    TextStyle,
    Color,
    Highlight.configure({ multicolor: true }),
    TextAlign.configure({
      types: ['heading', 'paragraph'],
    }),
    Image.configure({
      inline: false,
      allowBase64: true,
    }),
    TableKit.configure({
      table: { resizable: true },
    }),
    TaskList,
    TaskItem.configure({ nested: true }),
  ],
  onUpdate: ({ editor: instance }) => {
    emit('update:modelValue', instance.getHTML());
  },
});

const headingValue = computed(() => {
  if (!editor.value) return 'p';
  for (const level of [1, 2, 3]) {
    if (editor.value.isActive('heading', { level })) return String(level);
  }
  return 'p';
});

const currentColor = computed(() => {
  return editor.value?.getAttributes('textStyle').color || '#0f172a';
});

watch(
  () => props.modelValue,
  (val) => {
    if (!editor.value) return;
    const next = val || '';
    if (next !== editor.value.getHTML()) {
      editor.value.commands.setContent(next, { emitUpdate: false });
    }
  }
);

watch(
  () => props.readonly,
  (readonly) => {
    editor.value?.setEditable(!readonly);
  }
);

function run(fn) {
  if (!editor.value) return;
  fn(editor.value.chain().focus()).run();
}

function onHeadingChange(event) {
  const value = event.target.value;
  if (value === 'p') {
    run((e) => e.setParagraph());
  } else {
    run((e) => e.toggleHeading({ level: Number(value) }));
  }
}

function onColorInput(event) {
  const color = event.target.value;
  run((e) => e.setColor(color));
}

function setLink() {
  if (!editor.value) return;
  const previous = editor.value.getAttributes('link').href || '';
  const url = window.prompt('Nhập địa chỉ liên kết:', previous);
  if (url === null) return;
  if (url.trim() === '') {
    run((e) => e.unsetLink());
    return;
  }
  run((e) => e.extendMarkRange('link').setLink({ href: url.trim() }));
}

function setImage() {
  const url = window.prompt('Nhập URL hình ảnh:');
  if (!url || !url.trim()) return;
  run((e) => e.setImage({ src: url.trim() }));
}
</script>

<style scoped>
.rich-editor {
  display: flex;
  flex-direction: column;
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  background: var(--bg-surface);
  min-height: 0;
  flex: 1;
  overflow: hidden;
}

.rich-toolbar {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 2px;
  padding: 6px 8px;
  border-bottom: 1px solid var(--border);
  background: var(--bg-sub);
  flex-shrink: 0;
}

.rich-btn {
  min-width: 28px;
  height: 28px;
  padding: 0 6px;
  border-radius: var(--radius-sm);
  border: 1px solid transparent;
  color: var(--text-ink);
  font-size: 11px;
  font-weight: 700;
}
.rich-btn:hover {
  background: var(--bg-hover);
  border-color: var(--border);
}
.rich-btn.active {
  background: var(--accent-sub);
  border-color: var(--accent);
  color: var(--accent);
}

.rich-select {
  height: 28px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--bg-surface);
  color: var(--text-ink);
  font-size: 12px;
  padding: 0 6px;
}

.color-btn {
  position: relative;
  overflow: hidden;
}
.color-btn input[type='color'] {
  position: absolute;
  inset: 0;
  opacity: 0;
  cursor: pointer;
  width: 100%;
  height: 100%;
  border: none;
  padding: 0;
}

.rich-sep {
  width: 1px;
  height: 16px;
  background: var(--border);
  margin: 0 4px;
}

.rich-surface {
  flex: 1;
  min-height: 220px;
  overflow-y: auto;
  user-select: text;
}

.rich-surface :deep(.tiptap) {
  min-height: 220px;
  padding: 10px 12px;
  outline: none;
  color: var(--text-ink);
  font-size: 13.5px;
  line-height: 1.55;
}

.rich-surface :deep(.tiptap p.is-editor-empty:first-child::before) {
  content: attr(data-placeholder);
  float: left;
  color: var(--text-sub);
  pointer-events: none;
  height: 0;
}

.rich-surface :deep(h1) { font-size: 20px; margin: 8px 0 6px; }
.rich-surface :deep(h2) { font-size: 17px; margin: 8px 0 6px; }
.rich-surface :deep(h3) { font-size: 15px; margin: 8px 0 6px; }
.rich-surface :deep(ul),
.rich-surface :deep(ol) { padding-left: 22px; margin: 6px 0; }
.rich-surface :deep(blockquote) {
  border-left: 3px solid var(--accent);
  margin: 8px 0;
  padding: 4px 10px;
  color: var(--text-muted);
  background: var(--bg-sub);
}
.rich-surface :deep(hr) {
  border: none;
  border-top: 1px solid var(--border);
  margin: 12px 0;
}
.rich-surface :deep(a) { color: var(--accent); text-decoration: underline; }
.rich-surface :deep(img) { max-width: 100%; border-radius: 4px; margin: 8px 0; }
.rich-surface :deep(code) {
  background: var(--bg-sub);
  border: 1px solid var(--border);
  border-radius: 3px;
  padding: 1px 4px;
  font-size: 12px;
}
.rich-surface :deep(pre) {
  background: var(--bg-sub);
  border: 1px solid var(--border);
  border-radius: 6px;
  padding: 10px;
  overflow-x: auto;
}
.rich-surface :deep(table) {
  border-collapse: collapse;
  width: 100%;
  margin: 8px 0;
}
.rich-surface :deep(th),
.rich-surface :deep(td) {
  border: 1px solid var(--border);
  padding: 6px 8px;
  min-width: 48px;
  vertical-align: top;
}
.rich-surface :deep(th) {
  background: var(--bg-sub);
  font-weight: 700;
}
.rich-surface :deep(ul[data-type='taskList']) {
  list-style: none;
  padding-left: 4px;
}
.rich-surface :deep(ul[data-type='taskList'] li) {
  display: flex;
  align-items: flex-start;
  gap: 8px;
}
.rich-surface :deep(ul[data-type='taskList'] input) {
  margin-top: 3px;
}

.readonly .rich-surface {
  background: var(--bg-sub);
}
.readonly .rich-surface :deep(.tiptap) {
  min-height: 180px;
}
</style>
