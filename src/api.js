// API bridge for Tauri commands with multi-team & workspace support
import { invoke } from '@tauri-apps/api/core';

export const api = {
  // Connection
  async getConnectionStatus() {
    try {
      return await invoke('get_connection_status');
    } catch (e) {
      console.warn('getConnectionStatus error:', e);
      return false;
    }
  },

  async reconnectMongo(uri) {
    return await invoke('reconnect_mongo', { uri });
  },

  // Auth
  async login(email, password) {
    return await invoke('login', { email, password });
  },

  // Teams (Super Admin & Team Management)
  async getTeams() {
    return await invoke('get_teams');
  },

  async createTeam(name, description = '', enableReviewStep = true, enableResultColumn = true) {
    return await invoke('create_team', {
      name,
      description,
      enableReviewStep,
      enableResultColumn
    });
  },

  async updateTeam(id, name, description = '', enableReviewStep = null, enableResultColumn = null) {
    return await invoke('update_team', {
      id,
      name,
      description,
      enableReviewStep,
      enableResultColumn
    });
  },

  async deleteTeam(id) {
    return await invoke('delete_team', { id });
  },

  // Workspaces (Không gian làm việc trong mỗi Team)
  async getWorkspaces(teamId) {
    return await invoke('get_workspaces', { teamId: teamId || '' });
  },

  async createWorkspace(teamId, name, description = '', createdBy = 'Admin') {
    return await invoke('create_workspace', {
      teamId,
      name,
      description,
      createdBy
    });
  },

  async updateWorkspace(id, name, description = '') {
    return await invoke('update_workspace', { id, name, description });
  },

  async deleteWorkspace(id) {
    return await invoke('delete_workspace', { id });
  },

  // Users
  async getUsers(teamId = null) {
    return await invoke('get_users', { teamId: teamId || null });
  },

  async createUser(fullName, email, password, role, teamId = null, jobTitle = null) {
    return await invoke('create_user', {
      fullName,
      email,
      password,
      role,
      teamId: teamId || null,
      jobTitle: jobTitle || null
    });
  },

  async updateUser(email, fullName, role, isActive, password, teamId = null, jobTitle = null) {
    return await invoke('update_user', {
      email,
      fullName: fullName || null,
      role: role || null,
      isActive: isActive !== undefined ? isActive : null,
      password: password || null,
      teamId: teamId || null,
      jobTitle: jobTitle !== undefined ? jobTitle : null
    });
  },

  async deleteUser(email) {
    return await invoke('delete_user', { email });
  },

  // Tasks (Tickets)
  async getTasks(teamId = null, workspaceId = null, status = null, priority = null, userEmail = null) {
    return await invoke('get_tasks', {
      teamId: teamId || null,
      workspaceId: workspaceId || null,
      status: status || null,
      priority: priority || null,
      userEmail: userEmail || null
    });
  },

  async createTask(taskData) {
    return await invoke('create_task', {
      teamId: taskData.team_id || null,
      workspaceId: taskData.workspace_id || null,
      title: taskData.title,
      description: taskData.description || '',
      assignedTo: taskData.assigned_to || [],
      priority: taskData.priority || 'Medium',
      dueDate: taskData.due_date || null,
      createdBy: taskData.created_by || 'Admin',
      remindBeforeDays: taskData.remind_before_days || 2,
      attachments: taskData.attachments || []
    });
  },

  async updateTask(id, taskData) {
    return await invoke('update_task', {
      id,
      title: taskData.title,
      description: taskData.description || '',
      assignedTo: taskData.assigned_to || [],
      priority: taskData.priority || 'Medium',
      dueDate: taskData.due_date || null,
      status: taskData.status || 'TODO',
      remindBeforeDays: taskData.remind_before_days || 2,
      attachments: taskData.attachments !== undefined ? taskData.attachments : null,
      workspaceId: taskData.workspace_id || null,
      result: taskData.result !== undefined ? taskData.result : null,
      note: taskData.note !== undefined ? taskData.note : null
    });
  },

  async updateTaskStatus(id, status, result = null) {
    return await invoke('update_task_status', { id, status, result });
  },

  async deleteTask(id) {
    return await invoke('delete_task', { id });
  },

  // Export Excel File via Tauri native file writer
  async exportExcelFile(fileName, fileData) {
    let bytes;
    if (fileData instanceof Uint8Array) {
      bytes = Array.from(fileData);
    } else if (fileData instanceof ArrayBuffer) {
      bytes = Array.from(new Uint8Array(fileData));
    } else if (Array.isArray(fileData)) {
      bytes = fileData;
    } else {
      bytes = [];
    }
    return await invoke('export_excel_file', { fileName, fileData: bytes });
  },

  async saveExcelFileWithDialog(defaultFileName, fileData) {
    let bytes;
    if (fileData instanceof Uint8Array) {
      bytes = Array.from(fileData);
    } else if (fileData instanceof ArrayBuffer) {
      bytes = Array.from(new Uint8Array(fileData));
    } else if (Array.isArray(fileData)) {
      bytes = fileData;
    } else {
      bytes = [];
    }
    return await invoke('save_excel_file_with_dialog', { defaultFileName, fileData: bytes });
  },

  // Dashboard stats
  async getDashboardStats(teamId = null, workspaceId = null, forUser = null) {
    return await invoke('get_dashboard_stats', {
      teamId: teamId || null,
      workspaceId: workspaceId || null,
      forUser: forUser || null
    });
  },

  // Storage & Attachments (Cloudflare R2 dạng team -> workspace -> ticket)
  async isStorageConfigured() {
    try {
      return await invoke('is_storage_configured');
    } catch {
      return false;
    }
  },

  async uploadAttachment(teamId, workspaceId, taskId, fileName, fileData, contentType, uploadedBy, category = 'document') {
    let bytes;
    if (fileData instanceof Uint8Array) {
      bytes = Array.from(fileData);
    } else if (fileData instanceof ArrayBuffer) {
      bytes = Array.from(new Uint8Array(fileData));
    } else if (Array.isArray(fileData)) {
      bytes = fileData;
    } else {
      bytes = [];
    }

    return await invoke('upload_attachment', {
      teamId: teamId || null,
      workspaceId: workspaceId || null,
      taskId: taskId || null,
      fileName,
      fileData: bytes,
      contentType: contentType || 'application/octet-stream',
      uploadedBy: uploadedBy || 'Người dùng',
      category: category || 'document'
    });
  },

  async deleteAttachment(taskId, attachmentId, objectKey) {
    return await invoke('delete_attachment', {
      taskId: taskId || null,
      attachmentId,
      objectKey
    });
  },

  async getAttachmentUrl(objectKey) {
    return await invoke('get_attachment_url', { objectKey });
  },

  async openAttachment(objectKey, fileName) {
    return await invoke('open_attachment', { objectKey, fileName });
  }
};
