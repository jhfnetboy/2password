/**
 * TwoPassword Browser Extension - Password Detector
 * Detects login forms and password fields on web pages
 */

export class PasswordDetector {
  constructor() {
    this.detectionRules = {
      // Login form selectors
      loginFormSelectors: [
        'form[id*="login"]',
        'form[class*="login"]', 
        'form[id*="signin"]',
        'form[class*="signin"]',
        'form[id*="auth"]',
        'form[class*="auth"]'
      ],
      
      // Username field selectors
      usernameSelectors: [
        'input[name="username"]',
        'input[name="email"]',
        'input[name="user"]',
        'input[id*="username"]',
        'input[id*="email"]',
        'input[placeholder*="username" i]',
        'input[placeholder*="email" i]',
        'input[type="email"]'
      ],
      
      // Password field selectors
      passwordSelectors: [
        'input[type="password"]',
        'input[name="password"]',
        'input[id*="password"]',
        'input[placeholder*="password" i]'
      ],
      
      // Submit button selectors
      submitSelectors: [
        'input[type="submit"]',
        'button[type="submit"]',
        'button[id*="login"]',
        'button[class*="login"]',
        'button[id*="signin"]',
        'button[class*="signin"]'
      ]
    };
    
    this.scanResults = new Map();
  }

  /**
   * Scan tab for login forms and password fields
   */
  async scanTab(tabId, tab) {
    try {
      if (!this.shouldScanTab(tab)) {
        return;
      }

      console.log(`[PasswordDetector] Scanning tab: ${tab.url}`);
      
      // Inject content script to scan the page
      const results = await chrome.scripting.executeScript({
        target: { tabId },
        function: this.scanPageForForms,
        args: [this.detectionRules]
      });

      if (results && results[0]) {
        const scanResult = results[0].result;
        this.scanResults.set(tabId, {
          ...scanResult,
          timestamp: Date.now(),
          url: tab.url,
          domain: this.extractDomain(tab.url)
        });

        // Notify content script about detected forms
        if (scanResult.hasLoginForms) {
          await this.notifyContentScript(tabId, scanResult);
        }
      }
    } catch (error) {
      console.error('[PasswordDetector] Tab scan failed:', error);
    }
  }

  /**
   * Check if tab should be scanned
   */
  shouldScanTab(tab) {
    if (!tab || !tab.url) return false;
    
    // Skip special pages
    const skipProtocols = ['chrome:', 'chrome-extension:', 'moz-extension:', 'about:', 'file:'];
    if (skipProtocols.some(protocol => tab.url.startsWith(protocol))) {
      return false;
    }
    
    // Only scan HTTP/HTTPS pages
    if (!tab.url.startsWith('http://') && !tab.url.startsWith('https://')) {
      return false;
    }
    
    return true;
  }

  /**
   * Scan page for forms (runs in page context)
   */
  scanPageForForms(detectionRules) {
    const result = {
      hasLoginForms: false,
      forms: [],
      passwordFields: [],
      domain: window.location.hostname,
      url: window.location.href
    };

    try {
      // Find all forms
      const forms = document.querySelectorAll('form');
      
      forms.forEach((form, index) => {
        const formData = this.analyzeForm(form, index, detectionRules);
        if (formData.isLoginForm) {
          result.hasLoginForms = true;
          result.forms.push(formData);
        }
      });

      // Find standalone password fields (not in forms)
      const standalonePasswordFields = document.querySelectorAll('input[type="password"]');
      standalonePasswordFields.forEach(field => {
        if (!field.closest('form')) {
          result.passwordFields.push({
            id: field.id || null,
            name: field.name || null,
            placeholder: field.placeholder || null,
            position: this.getElementPosition(field)
          });
        }
      });

      console.log('[PasswordDetector] Page scan complete:', result);
      return result;
      
    } catch (error) {
      console.error('[PasswordDetector] Page scan error:', error);
      return result;
    }
  }

  /**
   * Analyze individual form
   */
  analyzeForm(form, index, detectionRules) {
    const formData = {
      index,
      id: form.id || null,
      action: form.action || null,
      method: form.method || 'GET',
      isLoginForm: false,
      usernameField: null,
      passwordField: null,
      submitButton: null,
      position: this.getElementPosition(form)
    };

    // Look for password fields
    const passwordFields = form.querySelectorAll(detectionRules.passwordSelectors.join(','));
    if (passwordFields.length === 0) {
      return formData; // Not a login form without password field
    }

    formData.passwordField = this.analyzeField(passwordFields[0]);
    formData.isLoginForm = true;

    // Look for username fields
    const usernameFields = form.querySelectorAll(detectionRules.usernameSelectors.join(','));
    if (usernameFields.length > 0) {
      formData.usernameField = this.analyzeField(usernameFields[0]);
    }

    // Look for submit button
    const submitButtons = form.querySelectorAll(detectionRules.submitSelectors.join(','));
    if (submitButtons.length > 0) {
      formData.submitButton = this.analyzeField(submitButtons[0]);
    }

    // Check if it's likely a login form based on additional hints
    formData.loginConfidence = this.calculateLoginConfidence(form, formData);

    return formData;
  }

  /**
   * Analyze form field
   */
  analyzeField(field) {
    return {
      id: field.id || null,
      name: field.name || null,
      type: field.type || null,
      placeholder: field.placeholder || null,
      autocomplete: field.autocomplete || null,
      position: this.getElementPosition(field)
    };
  }

  /**
   * Calculate login form confidence score
   */
  calculateLoginConfidence(form, formData) {
    let confidence = 50; // Base confidence

    // Form attributes
    const formText = (form.id + ' ' + form.className + ' ' + form.action).toLowerCase();
    const loginKeywords = ['login', 'signin', 'sign-in', 'auth', 'authenticate'];
    if (loginKeywords.some(keyword => formText.includes(keyword))) {
      confidence += 20;
    }

    // Has username field
    if (formData.usernameField) {
      confidence += 15;
    }

    // Password field characteristics
    if (formData.passwordField) {
      const passwordText = (
        formData.passwordField.name + ' ' +
        formData.passwordField.id + ' ' +
        formData.passwordField.placeholder
      ).toLowerCase();
      
      if (passwordText.includes('password')) {
        confidence += 10;
      }
    }

    // Form size (login forms are typically small)
    const inputCount = form.querySelectorAll('input').length;
    if (inputCount >= 2 && inputCount <= 6) {
      confidence += 10;
    } else if (inputCount > 10) {
      confidence -= 10; // Probably registration form
    }

    return Math.min(100, Math.max(0, confidence));
  }

  /**
   * Get element position
   */
  getElementPosition(element) {
    const rect = element.getBoundingClientRect();
    return {
      top: rect.top,
      left: rect.left,
      width: rect.width,
      height: rect.height
    };
  }

  /**
   * Notify content script about detected forms
   */
  async notifyContentScript(tabId, scanResult) {
    try {
      await chrome.tabs.sendMessage(tabId, {
        type: 'FORMS_DETECTED',
        data: scanResult
      });
    } catch (error) {
      // Content script might not be ready, that's ok
      console.log('[PasswordDetector] Content script notification failed (expected if script not ready)');
    }
  }

  /**
   * Detect login form on specific tab (called from content script)
   */
  async detectLoginForm(tabId) {
    try {
      const result = await chrome.scripting.executeScript({
        target: { tabId },
        function: this.findActiveLoginForm
      });

      if (result && result[0]) {
        const loginForm = result[0].result;
        
        // Store detection result
        this.scanResults.set(tabId, {
          ...loginForm,
          timestamp: Date.now()
        });

        return loginForm;
      }
    } catch (error) {
      console.error('[PasswordDetector] Login form detection failed:', error);
    }
    
    return null;
  }

  /**
   * Find currently active/visible login form
   */
  findActiveLoginForm() {
    const forms = document.querySelectorAll('form');
    let bestForm = null;
    let highestScore = 0;

    forms.forEach(form => {
      const passwordField = form.querySelector('input[type="password"]');
      if (!passwordField) return;

      // Check if form is visible
      const rect = form.getBoundingClientRect();
      if (rect.width === 0 || rect.height === 0) return;

      let score = 0;
      
      // Visibility score
      if (this.isElementVisible(form)) score += 40;
      
      // Position score (forms higher on page are more likely to be login forms)
      if (rect.top < window.innerHeight / 2) score += 20;
      
      // Size score (reasonable size forms)
      if (rect.width > 200 && rect.width < 600 && rect.height > 100 && rect.height < 400) {
        score += 15;
      }
      
      // Username field presence
      if (form.querySelector('input[type="email"], input[name*="username"], input[name*="email"]')) {
        score += 15;
      }
      
      // Recent interaction (focus/click)
      if (form.querySelector('input:focus')) score += 10;

      if (score > highestScore) {
        highestScore = score;
        bestForm = form;
      }
    });

    if (bestForm) {
      return this.analyzeForm(bestForm, 0, {
        usernameSelectors: ['input[type="email"]', 'input[name*="username"]', 'input[name*="email"]'],
        passwordSelectors: ['input[type="password"]'],
        submitSelectors: ['input[type="submit"]', 'button[type="submit"]']
      });
    }

    return null;
  }

  /**
   * Check if element is visible
   */
  isElementVisible(element) {
    const style = window.getComputedStyle(element);
    return style.display !== 'none' && 
           style.visibility !== 'hidden' && 
           style.opacity !== '0';
  }

  /**
   * Extract domain from URL
   */
  extractDomain(url) {
    try {
      return new URL(url).hostname;
    } catch (error) {
      return null;
    }
  }

  /**
   * Get scan results for tab
   */
  getScanResults(tabId) {
    return this.scanResults.get(tabId);
  }

  /**
   * Clear scan results for tab
   */
  clearScanResults(tabId) {
    this.scanResults.delete(tabId);
  }
}