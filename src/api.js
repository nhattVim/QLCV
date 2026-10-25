// API bridge for Tauri commands with local fallback session support
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

  // Users
  async getUsers() {
    return await invoke('get_users');
  },

  async createUser(fullName, email, password, role) {
    return await invoke('create_user', {
      fullName,
      email,
      password,
      role
    });
  },

  async updateUser(email, fullName, role, isActive, password) {
    return await invoke('update_user', {
      email,
      fullName: fullName || null,
      role: role || null,
      isActive: isActive !== undefined ? isActive : null,
      password: password || null
    });
  },

  async deleteUser(email) {
    return await invoke('delete_user', { email });
  },

  // Tasks
  async getTasks(status = null, priority = null, userEmail = null) {
    return await invoke('get_tasks', {
      status: status || null,
      priority: priority || null,
      userEmail: userEmail || null
    });
  },

  async createTask(taskData) {
    return await invoke('create_task', {
      title: taskData.title,
      description: taskData.description || '',
      assignedTo: taskData.assigned_to || [],
      priority: taskData.priority || 'Medium',
      dueDate: taskData.due_date || null,
      createdBy: taskData.created_by || 'Admin',
      remindBeforeDays: taskData.remind_before_days || 2
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
      remindBeforeDays: taskData.remind_before_days || 2
    });
  },

  async updateTaskStatus(id, status) {
    return await invoke('update_task_status', { id, status });
  },

  async deleteTask(id) {
    return await invoke('delete_task', { id });
  },

  // Dashboard stats
  async getDashboardStats(forUser = null) {
    return await invoke('get_dashboard_stats', { forUser: forUser || null });
  }
};
