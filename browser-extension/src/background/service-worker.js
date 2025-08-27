/**
 * TwoPassword Browser Extension - Service Worker
 * Handles background tasks, native messaging, and extension lifecycle
 */

import { NativeMessaging } from './native-messaging.js';
import { PasswordDetector } from './password-detector.js';
import { ContextMenuManager } from './context-menu.js';
import { StorageManager } from '../shared/storage.js';

class TwoPasswordServiceWorker {
  constructor() {
    this.nativeMessaging = new NativeMessaging();
    this.passwordDetector = new PasswordDetector();
    this.contextMenuManager = new ContextMenuManager();
    this.storageManager = new StorageManager();
    
    this.isConnectedToNative = false;
    this.extensionId = chrome.runtime.id;
    
    this.initializeServiceWorker();
  }

  async initializeServiceWorker() {
    try {
      console.log('[TwoPassword] Service Worker initializing...');
      
      // Set up event listeners
      this.setupEventListeners();
      
      // Initialize context menus
      await this.contextMenuManager.initialize();
      
      // Test native messaging connection
      await this.testNativeConnection();
      
      console.log('[TwoPassword] Service Worker initialized successfully');
    } catch (error) {
      console.error('[TwoPassword] Service Worker initialization failed:', error);
    }
  }

  setupEventListeners() {
    // Extension installation/startup
    chrome.runtime.onInstalled.addListener((details) => {
      this.handleInstallation(details);
    });

    chrome.runtime.onStartup.addListener(() => {
      console.log('[TwoPassword] Extension startup');
      this.testNativeConnection();
    });

    // Message handling from content scripts and popup
    chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
      this.handleMessage(message, sender, sendResponse);
      return true; // Keep message channel open for async responses
    });

    // Context menu clicks
    chrome.contextMenus.onClicked.addListener((info, tab) => {
      this.contextMenuManager.handleContextMenuClick(info, tab);
    });

    // Keyboard shortcuts
    chrome.commands.onCommand.addListener((command) => {
      this.handleCommand(command);
    });

    // Tab updates for password detection
    chrome.tabs.onUpdated.addListener((tabId, changeInfo, tab) => {
      if (changeInfo.status === 'complete') {
        this.passwordDetector.scanTab(tabId, tab);
      }
    });

    // Alarm for periodic tasks
    chrome.alarms.onAlarm.addListener((alarm) => {
      this.handleAlarm(alarm);
    });
  }

  async handleInstallation(details) {
    console.log('[TwoPassword] Installation details:', details);
    
    if (details.reason === 'install') {
      // First time installation
      await this.storageManager.initializeStorage();
      
      // Show welcome notification
      chrome.notifications.create('welcome', {
        type: 'basic',
        iconUrl: chrome.runtime.getURL('icons/icon-48.png'),
        title: 'TwoPassword Installed',
        message: 'Secure password management is now active. Click the icon to get started!'
      });
      
      // Set up periodic tasks
      chrome.alarms.create('healthCheck', { periodInMinutes: 60 });
      
    } else if (details.reason === 'update') {
      console.log(`[TwoPassword] Updated from version ${details.previousVersion}`);
      await this.handleVersionUpdate(details.previousVersion);
    }
  }

  async handleMessage(message, sender, sendResponse) {
    try {
      console.log('[TwoPassword] Received message:', message.type, sender);
      
      switch (message.type) {
        case 'GET_VAULT_STATUS':
          const status = await this.nativeMessaging.getVaultStatus();
          sendResponse({ success: true, data: status });
          break;

        case 'UNLOCK_VAULT':
          const unlockResult = await this.nativeMessaging.unlockVault(message.data);
          sendResponse({ success: true, data: unlockResult });
          break;

        case 'SEARCH_PASSWORDS':
          const searchResult = await this.nativeMessaging.searchPasswords(message.data);
          sendResponse({ success: true, data: searchResult });
          break;

        case 'SAVE_PASSWORD':
          const saveResult = await this.nativeMessaging.savePassword(message.data);
          sendResponse({ success: true, data: saveResult });
          break;

        case 'GENERATE_PASSWORD':
          const password = await this.nativeMessaging.generatePassword(message.data);
          sendResponse({ success: true, data: password });
          break;

        case 'GET_SECURITY_DASHBOARD':
          const dashboard = await this.nativeMessaging.getSecurityDashboard();
          sendResponse({ success: true, data: dashboard });
          break;

        case 'DETECT_LOGIN_FORM':
          await this.passwordDetector.detectLoginForm(sender.tab.id);
          sendResponse({ success: true });
          break;

        case 'CONTENT_SCRIPT_READY':
          // Content script is ready, can send initial data
          sendResponse({ success: true });
          break;

        default:
          console.warn('[TwoPassword] Unknown message type:', message.type);
          sendResponse({ success: false, error: 'Unknown message type' });
      }
    } catch (error) {
      console.error('[TwoPassword] Message handling error:', error);
      sendResponse({ success: false, error: error.message });
    }
  }

  async handleCommand(command) {
    console.log('[TwoPassword] Command received:', command);
    
    const activeTab = await this.getActiveTab();
    if (!activeTab) return;

    switch (command) {
      case 'toggle_password_generator':
        await this.sendToContentScript(activeTab.id, {
          type: 'TOGGLE_PASSWORD_GENERATOR'
        });
        break;

      case 'auto_fill_login':
        await this.sendToContentScript(activeTab.id, {
          type: 'AUTO_FILL_LOGIN'
        });
        break;

      case 'save_credentials':
        await this.sendToContentScript(activeTab.id, {
          type: 'SAVE_CREDENTIALS'
        });
        break;
    }
  }

  async handleAlarm(alarm) {
    console.log('[TwoPassword] Alarm triggered:', alarm.name);
    
    switch (alarm.name) {
      case 'healthCheck':
        await this.performHealthCheck();
        break;
    }
  }

  async testNativeConnection() {
    try {
      const status = await this.nativeMessaging.testConnection();
      this.isConnectedToNative = status.connected;
      
      if (this.isConnectedToNative) {
        console.log('[TwoPassword] Native messaging connected');
        chrome.action.setBadgeText({ text: '' });
        chrome.action.setBadgeBackgroundColor({ color: '#4CAF50' });
      } else {
        console.warn('[TwoPassword] Native messaging not available');
        chrome.action.setBadgeText({ text: '!' });
        chrome.action.setBadgeBackgroundColor({ color: '#FF9800' });
      }
    } catch (error) {
      console.error('[TwoPassword] Native connection test failed:', error);
      this.isConnectedToNative = false;
      chrome.action.setBadgeText({ text: 'X' });
      chrome.action.setBadgeBackgroundColor({ color: '#F44336' });
    }
  }

  async performHealthCheck() {
    try {
      // Test native connection
      await this.testNativeConnection();
      
      // Check storage health
      const storageHealth = await this.storageManager.checkHealth();
      
      // Update icon badge based on overall health
      if (this.isConnectedToNative && storageHealth.healthy) {
        chrome.action.setBadgeText({ text: '' });
      } else {
        chrome.action.setBadgeText({ text: '!' });
      }
      
    } catch (error) {
      console.error('[TwoPassword] Health check failed:', error);
    }
  }

  async getActiveTab() {
    try {
      const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
      return tab;
    } catch (error) {
      console.error('[TwoPassword] Failed to get active tab:', error);
      return null;
    }
  }

  async sendToContentScript(tabId, message) {
    try {
      await chrome.tabs.sendMessage(tabId, message);
    } catch (error) {
      console.error('[TwoPassword] Failed to send message to content script:', error);
    }
  }

  async handleVersionUpdate(previousVersion) {
    // Handle version-specific updates
    const currentVersion = chrome.runtime.getManifest().version;
    
    console.log(`[TwoPassword] Updating from ${previousVersion} to ${currentVersion}`);
    
    // Migration logic can be added here
    // For example, updating storage format, clearing old caches, etc.
  }
}

// Initialize the service worker
const serviceWorker = new TwoPasswordServiceWorker();

// Handle service worker lifecycle
self.addEventListener('activate', (event) => {
  console.log('[TwoPassword] Service Worker activated');
});

self.addEventListener('install', (event) => {
  console.log('[TwoPassword] Service Worker installed');
  self.skipWaiting();
});

// Export for testing
if (typeof module !== 'undefined' && module.exports) {
  module.exports = { TwoPasswordServiceWorker };
}