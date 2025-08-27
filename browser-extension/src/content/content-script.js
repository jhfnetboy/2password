/**
 * TwoPassword Browser Extension - Content Script
 * Handles password form detection, auto-fill, and user interaction on web pages
 */

class TwoPasswordContentScript {
  constructor() {
    this.isInitialized = false;
    this.detectedForms = [];
    this.activeField = null;
    this.autoFillOverlay = null;
    this.passwordGenerator = null;
    
    // Settings cache
    this.settings = {
      autoFillEnabled: true,
      autoSaveEnabled: true,
      showSuggestions: true
    };
    
    // Initialize content script
    this.initialize();
  }

  async initialize() {
    try {
      console.log('[TwoPassword Content] Initializing on', window.location.href);
      
      // Load settings
      await this.loadSettings();
      
      // Set up event listeners
      this.setupEventListeners();
      
      // Set up form monitoring
      this.setupFormMonitoring();
      
      // Set up password field monitoring
      this.setupPasswordFieldMonitoring();
      
      // Notify background script that content script is ready
      this.sendMessage({ type: 'CONTENT_SCRIPT_READY' });
      
      this.isInitialized = true;
      console.log('[TwoPassword Content] Initialization complete');
    } catch (error) {
      console.error('[TwoPassword Content] Initialization failed:', error);
    }
  }

  async loadSettings() {
    try {
      const response = await this.sendMessage({ type: 'GET_EXTENSION_SETTINGS' });
      if (response.success) {
        this.settings = { ...this.settings, ...response.data };
      }
    } catch (error) {
      console.error('[TwoPassword Content] Failed to load settings:', error);
    }
  }

  setupEventListeners() {
    // Listen for messages from background script
    chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
      this.handleMessage(message, sender, sendResponse);
      return true; // Keep message channel open
    });

    // Listen for form submissions
    document.addEventListener('submit', (event) => {
      this.handleFormSubmit(event);
    }, true);

    // Listen for input changes
    document.addEventListener('input', (event) => {
      this.handleInputChange(event);
    }, true);

    // Listen for focus events
    document.addEventListener('focus', (event) => {
      this.handleInputFocus(event);
    }, true);

    // Listen for keyboard shortcuts
    document.addEventListener('keydown', (event) => {
      this.handleKeyboardShortcut(event);
    }, true);

    // Listen for page unload
    window.addEventListener('beforeunload', () => {
      this.cleanup();
    });
  }

  setupFormMonitoring() {
    // Initial scan
    this.scanForForms();
    
    // Watch for dynamically added forms
    const observer = new MutationObserver((mutations) => {
      let shouldRescan = false;
      
      mutations.forEach((mutation) => {
        if (mutation.type === 'childList') {
          mutation.addedNodes.forEach((node) => {
            if (node.nodeType === Node.ELEMENT_NODE) {
              if (node.tagName === 'FORM' || node.querySelector('form')) {
                shouldRescan = true;
              }
            }
          });
        }
      });
      
      if (shouldRescan) {
        setTimeout(() => this.scanForForms(), 100);
      }
    });
    
    observer.observe(document.body, {
      childList: true,
      subtree: true
    });
    
    this.formObserver = observer;
  }

  setupPasswordFieldMonitoring() {
    // Monitor password field visibility changes
    const observer = new MutationObserver((mutations) => {
      mutations.forEach((mutation) => {
        if (mutation.type === 'attributes' && mutation.attributeName === 'type') {
          const element = mutation.target;
          if (element.type === 'password') {
            this.handlePasswordFieldReveal(element);
          }
        }
      });
    });
    
    observer.observe(document.body, {
      attributes: true,
      attributeFilter: ['type'],
      subtree: true
    });
    
    this.passwordFieldObserver = observer;
  }

  async handleMessage(message, sender, sendResponse) {
    try {
      console.log('[TwoPassword Content] Received message:', message.type);
      
      switch (message.type) {
        case 'FORMS_DETECTED':
          this.handleFormsDetected(message.data);
          sendResponse({ success: true });
          break;

        case 'TOGGLE_PASSWORD_GENERATOR':
          await this.togglePasswordGenerator();
          sendResponse({ success: true });
          break;

        case 'AUTO_FILL_LOGIN':
          await this.autoFillLogin();
          sendResponse({ success: true });
          break;

        case 'SAVE_CREDENTIALS':
          await this.saveCredentials();
          sendResponse({ success: true });
          break;

        case 'CONTEXT_AUTOFILL':
          await this.handleContextAutofill(message.data);
          sendResponse({ success: true });
          break;

        case 'SHOW_PASSWORD_GENERATOR':
          await this.showPasswordGenerator(message.data);
          sendResponse({ success: true });
          break;

        case 'SAVE_CREDENTIALS_CONTEXT':
          await this.handleContextSaveCredentials(message.data);
          sendResponse({ success: true });
          break;

        default:
          console.warn('[TwoPassword Content] Unknown message type:', message.type);
          sendResponse({ success: false, error: 'Unknown message type' });
      }
    } catch (error) {
      console.error('[TwoPassword Content] Message handling error:', error);
      sendResponse({ success: false, error: error.message });
    }
  }

  scanForForms() {
    const forms = document.querySelectorAll('form');
    const passwordFields = document.querySelectorAll('input[type="password"]');
    
    this.detectedForms = [];
    
    // Analyze forms
    forms.forEach((form, index) => {
      const formData = this.analyzeForm(form, index);
      if (formData.hasPasswordField) {
        this.detectedForms.push(formData);
      }
    });
    
    // Handle standalone password fields
    passwordFields.forEach((field) => {
      if (!field.closest('form')) {
        this.detectedForms.push(this.analyzeStandaloneField(field));
      }
    });
    
    console.log(`[TwoPassword Content] Detected ${this.detectedForms.length} login forms`);
    
    // Add autofill buttons if enabled
    if (this.settings.autoFillEnabled) {
      this.addAutoFillButtons();
    }
  }

  analyzeForm(form, index) {
    const passwordField = form.querySelector('input[type="password"]');
    const usernameField = this.findUsernameField(form);
    const submitButton = form.querySelector('input[type="submit"], button[type="submit"]');
    
    return {
      type: 'form',
      element: form,
      index,
      hasPasswordField: !!passwordField,
      passwordField,
      usernameField,
      submitButton,
      domain: window.location.hostname,
      url: window.location.href
    };
  }

  analyzeStandaloneField(field) {
    return {
      type: 'standalone',
      element: field,
      hasPasswordField: true,
      passwordField: field,
      usernameField: this.findNearbyUsernameField(field),
      domain: window.location.hostname,
      url: window.location.href
    };
  }

  findUsernameField(form) {
    const selectors = [
      'input[type="email"]',
      'input[name*="username" i]',
      'input[name*="email" i]',
      'input[name*="user" i]',
      'input[id*="username" i]',
      'input[id*="email" i]',
      'input[placeholder*="username" i]',
      'input[placeholder*="email" i]'
    ];
    
    for (const selector of selectors) {
      const field = form.querySelector(selector);
      if (field) return field;
    }
    
    return null;
  }

  findNearbyUsernameField(passwordField) {
    // Look for username field before the password field
    let sibling = passwordField.previousElementSibling;
    while (sibling) {
      if (sibling.tagName === 'INPUT' && 
          (sibling.type === 'text' || sibling.type === 'email')) {
        return sibling;
      }
      sibling = sibling.previousElementSibling;
    }
    
    return null;
  }

  addAutoFillButtons() {
    this.detectedForms.forEach((formData) => {
      if (formData.passwordField && !formData.passwordField.dataset.twopasswordButton) {
        this.addAutoFillButton(formData.passwordField);
        formData.passwordField.dataset.twopasswordButton = 'added';
      }
    });
  }

  addAutoFillButton(field) {
    // Create autofill button
    const button = document.createElement('button');
    button.type = 'button';
    button.className = 'twopassword-autofill-btn';
    button.innerHTML = '🔐';
    button.title = 'TwoPassword Auto-fill';
    button.style.cssText = `
      position: absolute;
      right: 5px;
      top: 50%;
      transform: translateY(-50%);
      background: #4CAF50;
      color: white;
      border: none;
      border-radius: 3px;
      width: 24px;
      height: 24px;
      font-size: 12px;
      cursor: pointer;
      z-index: 10000;
      box-shadow: 0 2px 4px rgba(0,0,0,0.2);
    `;
    
    // Position relative to field
    const fieldRect = field.getBoundingClientRect();
    const wrapper = document.createElement('div');
    wrapper.style.cssText = `
      position: relative;
      display: inline-block;
    `;
    
    // Wrap field if not already wrapped
    if (!field.parentElement.classList.contains('twopassword-field-wrapper')) {
      field.parentNode.insertBefore(wrapper, field);
      wrapper.appendChild(field);
      wrapper.classList.add('twopassword-field-wrapper');
    } else {
      wrapper = field.parentElement;
    }
    
    wrapper.appendChild(button);
    
    // Button click handler
    button.addEventListener('click', async (event) => {
      event.preventDefault();
      event.stopPropagation();
      await this.showAutoFillMenu(field, button);
    });
    
    // Hide button when field loses focus
    field.addEventListener('blur', () => {
      setTimeout(() => {
        if (button.parentElement && !button.matches(':hover')) {
          button.style.display = 'none';
        }
      }, 200);
    });
    
    // Show button when field gains focus
    field.addEventListener('focus', () => {
      button.style.display = 'block';
    });
    
    // Initially hide button
    button.style.display = 'none';
  }

  async showAutoFillMenu(field, button) {
    try {
      // Get credentials for current domain
      const response = await this.sendMessage({
        type: 'SEARCH_PASSWORDS',
        data: { domain: window.location.hostname }
      });
      
      if (response.success && response.data.length > 0) {
        this.showCredentialsList(field, response.data);
      } else {
        this.showNoCredentialsMessage(field);
      }
    } catch (error) {
      console.error('[TwoPassword Content] Auto-fill menu error:', error);
      this.showErrorMessage(field, 'Failed to load credentials');
    }
  }

  showCredentialsList(field, credentials) {
    // Remove existing menu
    this.removeAutoFillMenu();
    
    const menu = document.createElement('div');
    menu.className = 'twopassword-autofill-menu';
    menu.style.cssText = `
      position: absolute;
      top: 100%;
      left: 0;
      right: 0;
      background: white;
      border: 1px solid #ccc;
      border-radius: 4px;
      box-shadow: 0 4px 8px rgba(0,0,0,0.1);
      z-index: 10001;
      max-height: 200px;
      overflow-y: auto;
    `;
    
    credentials.forEach((credential, index) => {
      const item = document.createElement('div');
      item.className = 'twopassword-credential-item';
      item.style.cssText = `
        padding: 8px 12px;
        cursor: pointer;
        border-bottom: 1px solid #eee;
        display: flex;
        justify-content: space-between;
        align-items: center;
      `;
      
      item.innerHTML = `
        <div>
          <div style="font-weight: 500;">${this.escapeHtml(credential.username || credential.email)}</div>
          <div style="font-size: 12px; color: #666;">${this.escapeHtml(credential.title)}</div>
        </div>
        <div style="font-size: 12px; color: #999;">Fill</div>
      `;
      
      item.addEventListener('click', () => {
        this.fillCredentials(field, credential);
        this.removeAutoFillMenu();
      });
      
      item.addEventListener('mouseenter', () => {
        item.style.background = '#f5f5f5';
      });
      
      item.addEventListener('mouseleave', () => {
        item.style.background = 'white';
      });
      
      menu.appendChild(item);
    });
    
    // Position menu
    const fieldWrapper = field.closest('.twopassword-field-wrapper') || field.parentElement;
    fieldWrapper.appendChild(menu);
    
    // Close menu on outside click
    setTimeout(() => {
      document.addEventListener('click', this.handleAutoFillMenuClose.bind(this), { once: true });
    }, 10);
    
    this.currentAutoFillMenu = menu;
  }

  showNoCredentialsMessage(field) {
    this.removeAutoFillMenu();
    
    const menu = document.createElement('div');
    menu.className = 'twopassword-autofill-menu';
    menu.style.cssText = `
      position: absolute;
      top: 100%;
      left: 0;
      right: 0;
      background: white;
      border: 1px solid #ccc;
      border-radius: 4px;
      box-shadow: 0 4px 8px rgba(0,0,0,0.1);
      z-index: 10001;
      padding: 12px;
      text-align: center;
      color: #666;
    `;
    
    menu.textContent = 'No saved passwords for this site';
    
    const fieldWrapper = field.closest('.twopassword-field-wrapper') || field.parentElement;
    fieldWrapper.appendChild(menu);
    
    setTimeout(() => {
      this.removeAutoFillMenu();
    }, 2000);
    
    this.currentAutoFillMenu = menu;
  }

  showErrorMessage(field, message) {
    // Similar to showNoCredentialsMessage but with error styling
    this.removeAutoFillMenu();
    
    const menu = document.createElement('div');
    menu.className = 'twopassword-autofill-menu';
    menu.style.cssText = `
      position: absolute;
      top: 100%;
      left: 0;
      right: 0;
      background: #ffe6e6;
      border: 1px solid #ffcccc;
      border-radius: 4px;
      box-shadow: 0 4px 8px rgba(0,0,0,0.1);
      z-index: 10001;
      padding: 12px;
      text-align: center;
      color: #d00;
    `;
    
    menu.textContent = message;
    
    const fieldWrapper = field.closest('.twopassword-field-wrapper') || field.parentElement;
    fieldWrapper.appendChild(menu);
    
    setTimeout(() => {
      this.removeAutoFillMenu();
    }, 3000);
    
    this.currentAutoFillMenu = menu;
  }

  handleAutoFillMenuClose(event) {
    if (this.currentAutoFillMenu && !this.currentAutoFillMenu.contains(event.target)) {
      this.removeAutoFillMenu();
    }
  }

  removeAutoFillMenu() {
    if (this.currentAutoFillMenu) {
      this.currentAutoFillMenu.remove();
      this.currentAutoFillMenu = null;
    }
  }

  fillCredentials(field, credential) {
    try {
      // Find the form containing this field
      const formData = this.detectedForms.find(form => 
        form.passwordField === field || 
        (form.element && form.element.contains(field))
      );
      
      if (formData) {
        // Fill username field
        if (formData.usernameField && credential.username) {
          this.fillField(formData.usernameField, credential.username);
        }
        
        // Fill password field
        if (formData.passwordField && credential.password) {
          this.fillField(formData.passwordField, credential.password);
        }
        
        console.log('[TwoPassword Content] Credentials filled successfully');
      } else {
        // Just fill the current field if it's a password field
        if (field.type === 'password' && credential.password) {
          this.fillField(field, credential.password);
        }
      }
    } catch (error) {
      console.error('[TwoPassword Content] Fill credentials error:', error);
    }
  }

  fillField(field, value) {
    // Set field value
    field.value = value;
    
    // Trigger input events to notify the page
    const inputEvent = new Event('input', { bubbles: true, cancelable: true });
    const changeEvent = new Event('change', { bubbles: true, cancelable: true });
    
    field.dispatchEvent(inputEvent);
    field.dispatchEvent(changeEvent);
    
    // Focus the field briefly to trigger any focus handlers
    field.focus();
  }

  async handleFormSubmit(event) {
    if (!this.settings.autoSaveEnabled) return;
    
    const form = event.target;
    const passwordField = form.querySelector('input[type="password"]');
    
    if (passwordField && passwordField.value) {
      const usernameField = this.findUsernameField(form);
      
      if (usernameField && usernameField.value) {
        // Show save credentials prompt
        await this.showSaveCredentialsPrompt({
          domain: window.location.hostname,
          url: window.location.href,
          title: document.title,
          username: usernameField.value,
          password: passwordField.value
        });
      }
    }
  }

  async showSaveCredentialsPrompt(credentialData) {
    // Check if we already have credentials for this domain/username
    const response = await this.sendMessage({
      type: 'SEARCH_PASSWORDS',
      data: { 
        domain: credentialData.domain,
        username: credentialData.username 
      }
    });
    
    if (response.success && response.data.length > 0) {
      // Ask if user wants to update existing credentials
      this.showUpdateCredentialsPrompt(credentialData, response.data[0]);
    } else {
      // Show save new credentials prompt
      this.showSaveNewCredentialsPrompt(credentialData);
    }
  }

  showSaveNewCredentialsPrompt(credentialData) {
    const prompt = this.createCredentialsPrompt(
      'Save Password?',
      `Save login for ${credentialData.domain}?`,
      [
        { text: 'Save', action: () => this.saveCredentials(credentialData) },
        { text: 'Never for this site', action: () => this.neverSaveForSite(credentialData.domain) },
        { text: 'Not now', action: () => this.dismissPrompt() }
      ]
    );
    
    this.showPrompt(prompt);
  }

  showUpdateCredentialsPrompt(credentialData, existingCredential) {
    const prompt = this.createCredentialsPrompt(
      'Update Password?',
      `Update saved password for ${credentialData.username}@${credentialData.domain}?`,
      [
        { text: 'Update', action: () => this.updateCredentials(existingCredential.id, credentialData) },
        { text: 'Save as new', action: () => this.saveCredentials(credentialData) },
        { text: 'Not now', action: () => this.dismissPrompt() }
      ]
    );
    
    this.showPrompt(prompt);
  }

  createCredentialsPrompt(title, message, buttons) {
    const prompt = document.createElement('div');
    prompt.className = 'twopassword-save-prompt';
    prompt.style.cssText = `
      position: fixed;
      top: 20px;
      right: 20px;
      background: white;
      border: 1px solid #ccc;
      border-radius: 8px;
      box-shadow: 0 4px 12px rgba(0,0,0,0.15);
      z-index: 10002;
      padding: 16px;
      min-width: 300px;
      font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    `;
    
    prompt.innerHTML = `
      <div style="display: flex; align-items: center; margin-bottom: 12px;">
        <img src="${chrome.runtime.getURL('icons/icon-32.png')}" width="24" height="24" style="margin-right: 8px;">
        <h3 style="margin: 0; font-size: 16px; font-weight: 600;">${title}</h3>
      </div>
      <p style="margin: 0 0 16px 0; color: #666; font-size: 14px;">${message}</p>
      <div class="twopassword-prompt-buttons" style="display: flex; gap: 8px; justify-content: flex-end;">
      </div>
    `;
    
    const buttonsContainer = prompt.querySelector('.twopassword-prompt-buttons');
    
    buttons.forEach((buttonConfig) => {
      const button = document.createElement('button');
      button.textContent = buttonConfig.text;
      button.style.cssText = `
        padding: 6px 12px;
        border: 1px solid #ddd;
        border-radius: 4px;
        background: white;
        color: #333;
        font-size: 14px;
        cursor: pointer;
      `;
      
      if (buttonConfig.text === 'Save' || buttonConfig.text === 'Update') {
        button.style.background = '#4CAF50';
        button.style.color = 'white';
        button.style.border = '1px solid #45a049';
      }
      
      button.addEventListener('click', () => {
        this.removePrompt();
        buttonConfig.action();
      });
      
      buttonsContainer.appendChild(button);
    });
    
    return prompt;
  }

  showPrompt(prompt) {
    this.removePrompt(); // Remove any existing prompt
    document.body.appendChild(prompt);
    this.currentPrompt = prompt;
    
    // Auto-dismiss after 10 seconds
    setTimeout(() => {
      this.removePrompt();
    }, 10000);
  }

  removePrompt() {
    if (this.currentPrompt) {
      this.currentPrompt.remove();
      this.currentPrompt = null;
    }
  }

  dismissPrompt() {
    this.removePrompt();
  }

  async saveCredentials(credentialData) {
    try {
      const response = await this.sendMessage({
        type: 'SAVE_PASSWORD',
        data: credentialData
      });
      
      if (response.success) {
        this.showSuccessNotification('Password saved successfully');
      } else {
        this.showErrorNotification('Failed to save password');
      }
    } catch (error) {
      console.error('[TwoPassword Content] Save credentials error:', error);
      this.showErrorNotification('Failed to save password');
    }
  }

  async updateCredentials(id, credentialData) {
    try {
      const response = await this.sendMessage({
        type: 'UPDATE_PASSWORD',
        data: { id, ...credentialData }
      });
      
      if (response.success) {
        this.showSuccessNotification('Password updated successfully');
      } else {
        this.showErrorNotification('Failed to update password');
      }
    } catch (error) {
      console.error('[TwoPassword Content] Update credentials error:', error);
      this.showErrorNotification('Failed to update password');
    }
  }

  async neverSaveForSite(domain) {
    // Add domain to never-save list
    try {
      const response = await this.sendMessage({
        type: 'ADD_NEVER_SAVE_DOMAIN',
        data: { domain }
      });
      
      if (response.success) {
        this.showSuccessNotification(`Won't save passwords for ${domain}`);
      }
    } catch (error) {
      console.error('[TwoPassword Content] Never save error:', error);
    }
  }

  showSuccessNotification(message) {
    this.showNotification(message, 'success');
  }

  showErrorNotification(message) {
    this.showNotification(message, 'error');
  }

  showNotification(message, type) {
    const notification = document.createElement('div');
    notification.style.cssText = `
      position: fixed;
      top: 20px;
      right: 20px;
      padding: 12px 16px;
      border-radius: 4px;
      color: white;
      font-size: 14px;
      z-index: 10003;
      max-width: 300px;
      box-shadow: 0 4px 8px rgba(0,0,0,0.2);
      background: ${type === 'success' ? '#4CAF50' : '#f44336'};
    `;
    
    notification.textContent = message;
    document.body.appendChild(notification);
    
    // Auto-remove after 3 seconds
    setTimeout(() => {
      notification.remove();
    }, 3000);
  }

  // Utility methods
  async sendMessage(message) {
    return new Promise((resolve, reject) => {
      chrome.runtime.sendMessage(message, (response) => {
        if (chrome.runtime.lastError) {
          reject(new Error(chrome.runtime.lastError.message));
        } else {
          resolve(response);
        }
      });
    });
  }

  escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
  }

  cleanup() {
    // Clean up observers and event listeners
    if (this.formObserver) {
      this.formObserver.disconnect();
    }
    
    if (this.passwordFieldObserver) {
      this.passwordFieldObserver.disconnect();
    }
    
    this.removeAutoFillMenu();
    this.removePrompt();
  }
}

// Initialize content script
const twoPasswordContentScript = new TwoPasswordContentScript();