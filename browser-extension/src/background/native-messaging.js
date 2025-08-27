/**
 * TwoPassword Browser Extension - Native Messaging
 * Handles communication with the native TwoPassword application
 */

export class NativeMessaging {
  constructor() {
    this.hostName = 'com.twopassword.native';
    this.port = null;
    this.messageQueue = [];
    this.isConnected = false;
    this.connectionTimeout = 5000; // 5 seconds
    this.reconnectAttempts = 0;
    this.maxReconnectAttempts = 3;
  }

  /**
   * Test connection to native host
   */
  async testConnection() {
    try {
      const response = await this.sendMessage({ type: 'ping' });
      return { connected: true, response };
    } catch (error) {
      console.error('[Native] Connection test failed:', error);
      return { connected: false, error: error.message };
    }
  }

  /**
   * Connect to native host
   */
  async connect() {
    if (this.isConnected) {
      return true;
    }

    return new Promise((resolve, reject) => {
      try {
        console.log('[Native] Connecting to native host:', this.hostName);
        
        this.port = chrome.runtime.connectNative(this.hostName);
        
        if (!this.port) {
          throw new Error('Failed to connect to native host');
        }

        // Set up event handlers
        this.port.onMessage.addListener((message) => {
          this.handleNativeMessage(message);
        });

        this.port.onDisconnect.addListener(() => {
          this.handleDisconnection();
        });

        // Test connection with ping
        setTimeout(async () => {
          try {
            await this.sendPing();
            this.isConnected = true;
            this.reconnectAttempts = 0;
            console.log('[Native] Connected successfully');
            resolve(true);
          } catch (error) {
            console.error('[Native] Connection verification failed:', error);
            this.isConnected = false;
            reject(error);
          }
        }, 100);

      } catch (error) {
        console.error('[Native] Connection failed:', error);
        this.isConnected = false;
        reject(error);
      }
    });
  }

  /**
   * Disconnect from native host
   */
  disconnect() {
    if (this.port) {
      this.port.disconnect();
      this.port = null;
    }
    this.isConnected = false;
    console.log('[Native] Disconnected');
  }

  /**
   * Send message to native host
   */
  async sendMessage(message) {
    return new Promise(async (resolve, reject) => {
      try {
        // Ensure connection
        if (!this.isConnected) {
          await this.connect();
        }

        if (!this.port) {
          throw new Error('No active connection to native host');
        }

        // Generate unique message ID
        const messageId = this.generateMessageId();
        const messageWithId = { ...message, id: messageId };

        // Set up response handler
        const timeout = setTimeout(() => {
          reject(new Error('Message timeout'));
        }, this.connectionTimeout);

        // Store response handler
        this.messageQueue.push({
          id: messageId,
          resolve,
          reject,
          timeout
        });

        // Send message
        console.log('[Native] Sending message:', messageWithId.type);
        this.port.postMessage(messageWithId);

      } catch (error) {
        console.error('[Native] Send message failed:', error);
        reject(error);
      }
    });
  }

  /**
   * Handle incoming message from native host
   */
  handleNativeMessage(message) {
    console.log('[Native] Received message:', message.type);

    // Find and resolve pending message
    const queueIndex = this.messageQueue.findIndex(item => item.id === message.id);
    if (queueIndex !== -1) {
      const queueItem = this.messageQueue[queueIndex];
      clearTimeout(queueItem.timeout);
      this.messageQueue.splice(queueIndex, 1);

      if (message.success) {
        queueItem.resolve(message.data || message);
      } else {
        queueItem.reject(new Error(message.error || 'Native operation failed'));
      }
    } else {
      // Handle unsolicited messages (notifications, events)
      this.handleUnsolicitedMessage(message);
    }
  }

  /**
   * Handle unsolicited messages from native host
   */
  handleUnsolicitedMessage(message) {
    console.log('[Native] Unsolicited message:', message.type);
    
    switch (message.type) {
      case 'vault_locked':
        this.broadcastToExtension({ type: 'VAULT_LOCKED' });
        break;
      
      case 'vault_unlocked':
        this.broadcastToExtension({ type: 'VAULT_UNLOCKED' });
        break;
      
      case 'password_saved':
        this.broadcastToExtension({ type: 'PASSWORD_SAVED', data: message.data });
        break;
      
      case 'security_alert':
        this.showSecurityAlert(message.data);
        break;
    }
  }

  /**
   * Handle connection disconnection
   */
  handleDisconnection() {
    console.warn('[Native] Connection disconnected');
    
    const error = chrome.runtime.lastError;
    if (error) {
      console.error('[Native] Disconnection error:', error.message);
    }

    this.isConnected = false;
    this.port = null;

    // Reject all pending messages
    this.messageQueue.forEach(item => {
      clearTimeout(item.timeout);
      item.reject(new Error('Connection lost'));
    });
    this.messageQueue = [];

    // Attempt reconnection
    if (this.reconnectAttempts < this.maxReconnectAttempts) {
      this.reconnectAttempts++;
      console.log(`[Native] Attempting reconnection ${this.reconnectAttempts}/${this.maxReconnectAttempts}`);
      
      setTimeout(() => {
        this.connect().catch(error => {
          console.error('[Native] Reconnection failed:', error);
        });
      }, 2000 * this.reconnectAttempts); // Exponential backoff
    }
  }

  /**
   * Send ping to test connection
   */
  async sendPing() {
    return this.sendMessage({ type: 'ping' });
  }

  // Vault operations
  async getVaultStatus() {
    return this.sendMessage({ type: 'get_vault_status' });
  }

  async unlockVault(credentials) {
    return this.sendMessage({ 
      type: 'unlock_vault',
      data: credentials
    });
  }

  async lockVault() {
    return this.sendMessage({ type: 'lock_vault' });
  }

  // Password operations
  async searchPasswords(query) {
    return this.sendMessage({
      type: 'search_passwords',
      data: { query }
    });
  }

  async getPasswordsByDomain(domain) {
    return this.sendMessage({
      type: 'get_passwords_by_domain',
      data: { domain }
    });
  }

  async savePassword(passwordData) {
    return this.sendMessage({
      type: 'save_password',
      data: passwordData
    });
  }

  async updatePassword(id, passwordData) {
    return this.sendMessage({
      type: 'update_password',
      data: { id, ...passwordData }
    });
  }

  async deletePassword(id) {
    return this.sendMessage({
      type: 'delete_password',
      data: { id }
    });
  }

  // Password generation
  async generatePassword(options = {}) {
    return this.sendMessage({
      type: 'generate_password',
      data: options
    });
  }

  // Security features
  async getSecurityDashboard() {
    return this.sendMessage({ type: 'get_security_dashboard' });
  }

  async checkPasswordHealth(passwords) {
    return this.sendMessage({
      type: 'check_password_health',
      data: { passwords }
    });
  }

  async scanForBreaches(passwords) {
    return this.sendMessage({
      type: 'scan_breaches',
      data: { passwords }
    });
  }

  // Utility methods
  generateMessageId() {
    return 'msg_' + Date.now() + '_' + Math.random().toString(36).substr(2, 9);
  }

  broadcastToExtension(message) {
    // Send message to all extension contexts (popup, content scripts)
    chrome.runtime.sendMessage(message).catch(error => {
      // Ignore errors if no listeners
      console.log('[Native] Broadcast message sent:', message.type);
    });
  }

  showSecurityAlert(alertData) {
    chrome.notifications.create('security_alert_' + Date.now(), {
      type: 'basic',
      iconUrl: chrome.runtime.getURL('icons/icon-48.png'),
      title: 'TwoPassword Security Alert',
      message: alertData.message,
      priority: 2
    });
  }
}