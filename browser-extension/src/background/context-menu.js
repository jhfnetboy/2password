/**
 * TwoPassword Browser Extension - Context Menu Manager
 * Manages right-click context menus for password operations
 */

export class ContextMenuManager {
  constructor() {
    this.menuItems = new Map();
    this.isInitialized = false;
  }

  /**
   * Initialize context menus
   */
  async initialize() {
    try {
      // Remove any existing menus first
      await this.removeAllMenus();
      
      // Create main menu items
      await this.createMenuItems();
      
      this.isInitialized = true;
      console.log('[ContextMenu] Context menus initialized');
    } catch (error) {
      console.error('[ContextMenu] Initialization failed:', error);
    }
  }

  /**
   * Create all context menu items
   */
  async createMenuItems() {
    // Main TwoPassword menu
    const mainMenuId = await this.createMenuItem({
      id: 'twopassword_main',
      title: 'TwoPassword',
      contexts: ['all']
    });

    // Auto-fill submenu (for input fields)
    const autofillMenuId = await this.createMenuItem({
      id: 'twopassword_autofill',
      parentId: mainMenuId,
      title: 'Auto-fill Login',
      contexts: ['editable'],
      enabled: true
    });

    // Generate password submenu
    const generateMenuId = await this.createMenuItem({
      id: 'twopassword_generate',
      parentId: mainMenuId,
      title: 'Generate Password',
      contexts: ['editable']
    });

    // Save credentials submenu (for pages with login forms)
    const saveMenuId = await this.createMenuItem({
      id: 'twopassword_save',
      parentId: mainMenuId,
      title: 'Save Login Credentials',
      contexts: ['page']
    });

    // Separator
    await this.createMenuItem({
      id: 'twopassword_separator1',
      parentId: mainMenuId,
      type: 'separator',
      contexts: ['all']
    });

    // Open vault
    const vaultMenuId = await this.createMenuItem({
      id: 'twopassword_vault',
      parentId: mainMenuId,
      title: 'Open Vault',
      contexts: ['all']
    });

    // Security dashboard
    const dashboardMenuId = await this.createMenuItem({
      id: 'twopassword_dashboard',
      parentId: mainMenuId,
      title: 'Security Dashboard',
      contexts: ['all']
    });

    // Separator
    await this.createMenuItem({
      id: 'twopassword_separator2',
      parentId: mainMenuId,
      type: 'separator',
      contexts: ['all']
    });

    // Settings
    const settingsMenuId = await this.createMenuItem({
      id: 'twopassword_settings',
      parentId: mainMenuId,
      title: 'Settings',
      contexts: ['all']
    });

    // Lock vault
    const lockMenuId = await this.createMenuItem({
      id: 'twopassword_lock',
      parentId: mainMenuId,
      title: 'Lock Vault',
      contexts: ['all'],
      enabled: false // Will be enabled when vault is unlocked
    });

    console.log('[ContextMenu] Created menu items');
  }

  /**
   * Create a single context menu item
   */
  async createMenuItem(options) {
    return new Promise((resolve, reject) => {
      const menuId = chrome.contextMenus.create(options, () => {
        if (chrome.runtime.lastError) {
          console.error('[ContextMenu] Failed to create menu:', chrome.runtime.lastError);
          reject(chrome.runtime.lastError);
        } else {
          this.menuItems.set(options.id, { ...options, menuId });
          resolve(menuId);
        }
      });
    });
  }

  /**
   * Remove all context menus
   */
  async removeAllMenus() {
    return new Promise((resolve) => {
      chrome.contextMenus.removeAll(() => {
        this.menuItems.clear();
        console.log('[ContextMenu] All menus removed');
        resolve();
      });
    });
  }

  /**
   * Handle context menu click
   */
  async handleContextMenuClick(info, tab) {
    console.log('[ContextMenu] Menu clicked:', info.menuItemId);

    try {
      switch (info.menuItemId) {
        case 'twopassword_autofill':
          await this.handleAutofill(info, tab);
          break;

        case 'twopassword_generate':
          await this.handleGeneratePassword(info, tab);
          break;

        case 'twopassword_save':
          await this.handleSaveCredentials(info, tab);
          break;

        case 'twopassword_vault':
          await this.handleOpenVault();
          break;

        case 'twopassword_dashboard':
          await this.handleOpenDashboard();
          break;

        case 'twopassword_settings':
          await this.handleOpenSettings();
          break;

        case 'twopassword_lock':
          await this.handleLockVault();
          break;

        default:
          console.warn('[ContextMenu] Unknown menu item:', info.menuItemId);
      }
    } catch (error) {
      console.error('[ContextMenu] Menu action failed:', error);
      this.showErrorNotification('Action failed: ' + error.message);
    }
  }

  /**
   * Handle auto-fill action
   */
  async handleAutofill(info, tab) {
    // Send message to content script to trigger autofill
    await chrome.tabs.sendMessage(tab.id, {
      type: 'CONTEXT_AUTOFILL',
      data: {
        frameId: info.frameId,
        selectionText: info.selectionText
      }
    });
  }

  /**
   * Handle generate password action
   */
  async handleGeneratePassword(info, tab) {
    try {
      // Get password generation preferences
      const result = await chrome.storage.sync.get(['passwordGeneration']);
      const settings = result.passwordGeneration || {
        length: 16,
        includeUppercase: true,
        includeLowercase: true,
        includeNumbers: true,
        includeSymbols: true,
        excludeSimilar: true
      };

      // Send message to content script to show password generator
      await chrome.tabs.sendMessage(tab.id, {
        type: 'SHOW_PASSWORD_GENERATOR',
        data: {
          frameId: info.frameId,
          settings
        }
      });
    } catch (error) {
      console.error('[ContextMenu] Generate password failed:', error);
    }
  }

  /**
   * Handle save credentials action
   */
  async handleSaveCredentials(info, tab) {
    // Send message to content script to detect and save credentials
    await chrome.tabs.sendMessage(tab.id, {
      type: 'SAVE_CREDENTIALS_CONTEXT',
      data: {
        url: tab.url,
        title: tab.title
      }
    });
  }

  /**
   * Handle open vault action
   */
  async handleOpenVault() {
    // Open popup in new tab or window
    chrome.tabs.create({
      url: chrome.runtime.getURL('src/popup/popup.html?mode=fullscreen')
    });
  }

  /**
   * Handle open dashboard action
   */
  async handleOpenDashboard() {
    // Open security dashboard
    chrome.tabs.create({
      url: chrome.runtime.getURL('src/options/options.html?tab=security')
    });
  }

  /**
   * Handle open settings action
   */
  async handleOpenSettings() {
    // Open options page
    chrome.runtime.openOptionsPage();
  }

  /**
   * Handle lock vault action
   */
  async handleLockVault() {
    try {
      // Send message to background script to lock vault
      const response = await chrome.runtime.sendMessage({
        type: 'LOCK_VAULT'
      });

      if (response.success) {
        this.showSuccessNotification('Vault locked successfully');
        this.updateMenuStates(false); // Update menu states for locked vault
      } else {
        this.showErrorNotification('Failed to lock vault');
      }
    } catch (error) {
      console.error('[ContextMenu] Lock vault failed:', error);
    }
  }

  /**
   * Update menu states based on vault status
   */
  updateMenuStates(isUnlocked) {
    try {
      // Enable/disable menu items based on vault state
      chrome.contextMenus.update('twopassword_autofill', {
        enabled: isUnlocked
      });

      chrome.contextMenus.update('twopassword_save', {
        enabled: isUnlocked
      });

      chrome.contextMenus.update('twopassword_lock', {
        enabled: isUnlocked
      });

      console.log(`[ContextMenu] Menu states updated (unlocked: ${isUnlocked})`);
    } catch (error) {
      console.error('[ContextMenu] Failed to update menu states:', error);
    }
  }

  /**
   * Update menus based on page context
   */
  updateContextualMenus(tabId, hasLoginForms, hasPasswordFields) {
    try {
      // Show/hide save credentials based on login form detection
      chrome.contextMenus.update('twopassword_save', {
        visible: hasLoginForms
      });

      // Update autofill availability
      chrome.contextMenus.update('twopassword_autofill', {
        visible: hasLoginForms || hasPasswordFields
      });

      console.log(`[ContextMenu] Contextual menus updated (forms: ${hasLoginForms}, fields: ${hasPasswordFields})`);
    } catch (error) {
      console.error('[ContextMenu] Failed to update contextual menus:', error);
    }
  }

  /**
   * Show success notification
   */
  showSuccessNotification(message) {
    chrome.notifications.create('success_' + Date.now(), {
      type: 'basic',
      iconUrl: chrome.runtime.getURL('icons/icon-48.png'),
      title: 'TwoPassword',
      message: message
    });
  }

  /**
   * Show error notification
   */
  showErrorNotification(message) {
    chrome.notifications.create('error_' + Date.now(), {
      type: 'basic',
      iconUrl: chrome.runtime.getURL('icons/icon-48.png'),
      title: 'TwoPassword Error',
      message: message
    });
  }

  /**
   * Clean up resources
   */
  cleanup() {
    this.removeAllMenus();
    this.isInitialized = false;
  }
}