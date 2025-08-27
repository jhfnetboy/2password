/**
 * TwoPassword Browser Extension - Storage Manager
 * Manages extension storage for settings, cache, and temporary data
 */

export class StorageManager {
  constructor() {
    this.storageKeys = {
      // Settings
      EXTENSION_SETTINGS: 'twopassword_settings',
      PASSWORD_GENERATION: 'password_generation',
      AUTO_FILL_SETTINGS: 'autofill_settings',
      SECURITY_SETTINGS: 'security_settings',
      
      // Cache
      DOMAIN_CREDENTIALS: 'domain_credentials_cache',
      FORM_DETECTION_CACHE: 'form_detection_cache',
      SECURITY_DASHBOARD_CACHE: 'security_dashboard_cache',
      
      // Temporary data
      SESSION_DATA: 'session_data',
      PENDING_SAVES: 'pending_saves',
      LAST_ACTIVITY: 'last_activity'
    };

    this.defaultSettings = {
      extensionSettings: {
        autoFillEnabled: true,
        autoSaveEnabled: true,
        showNotifications: true,
        lockTimeout: 15, // minutes
        theme: 'auto',
        language: 'en'
      },
      
      passwordGeneration: {
        length: 16,
        includeUppercase: true,
        includeLowercase: true,
        includeNumbers: true,
        includeSymbols: true,
        excludeSimilar: true,
        customSymbols: '!@#$%^&*()_+-=[]{}|;:,.<>?'
      },
      
      autoFillSettings: {
        autoSubmit: false,
        fillOnFocus: true,
        showSuggestions: true,
        maxSuggestions: 5,
        matchingStrategy: 'domain' // 'domain', 'subdomain', 'exact'
      },
      
      securitySettings: {
        breachCheckEnabled: true,
        weakPasswordAlerts: true,
        duplicatePasswordAlerts: true,
        oldPasswordAlerts: true,
        oldPasswordThreshold: 90 // days
      }
    };
  }

  /**
   * Initialize storage with default settings
   */
  async initializeStorage() {
    try {
      console.log('[Storage] Initializing storage...');
      
      // Check if settings exist
      const existingSettings = await this.getAllSettings();
      
      // Merge with defaults for any missing settings
      const mergedSettings = this.mergeWithDefaults(existingSettings);
      
      // Save merged settings
      await this.saveAllSettings(mergedSettings);
      
      // Initialize cache areas
      await this.initializeCache();
      
      console.log('[Storage] Storage initialized successfully');
    } catch (error) {
      console.error('[Storage] Initialization failed:', error);
      throw error;
    }
  }

  /**
   * Merge existing settings with defaults
   */
  mergeWithDefaults(existingSettings) {
    const merged = {};
    
    for (const [key, defaultValue] of Object.entries(this.defaultSettings)) {
      if (existingSettings[key]) {
        // Merge existing with defaults (in case new settings were added)
        merged[key] = { ...defaultValue, ...existingSettings[key] };
      } else {
        merged[key] = { ...defaultValue };
      }
    }
    
    return merged;
  }

  /**
   * Initialize cache areas
   */
  async initializeCache() {
    const cacheData = {
      [this.storageKeys.DOMAIN_CREDENTIALS]: {},
      [this.storageKeys.FORM_DETECTION_CACHE]: {},
      [this.storageKeys.SECURITY_DASHBOARD_CACHE]: null,
      [this.storageKeys.SESSION_DATA]: {},
      [this.storageKeys.PENDING_SAVES]: [],
      [this.storageKeys.LAST_ACTIVITY]: Date.now()
    };
    
    await chrome.storage.local.set(cacheData);
  }

  // Settings Management
  async getAllSettings() {
    try {
      const data = await chrome.storage.sync.get(Object.keys(this.defaultSettings));
      return data;
    } catch (error) {
      console.error('[Storage] Failed to get settings:', error);
      return {};
    }
  }

  async saveAllSettings(settings) {
    try {
      await chrome.storage.sync.set(settings);
      console.log('[Storage] Settings saved successfully');
    } catch (error) {
      console.error('[Storage] Failed to save settings:', error);
      throw error;
    }
  }

  async getExtensionSettings() {
    const result = await chrome.storage.sync.get(this.storageKeys.EXTENSION_SETTINGS);
    return result[this.storageKeys.EXTENSION_SETTINGS] || this.defaultSettings.extensionSettings;
  }

  async saveExtensionSettings(settings) {
    await chrome.storage.sync.set({
      [this.storageKeys.EXTENSION_SETTINGS]: settings
    });
  }

  async getPasswordGenerationSettings() {
    const result = await chrome.storage.sync.get(this.storageKeys.PASSWORD_GENERATION);
    return result[this.storageKeys.PASSWORD_GENERATION] || this.defaultSettings.passwordGeneration;
  }

  async savePasswordGenerationSettings(settings) {
    await chrome.storage.sync.set({
      [this.storageKeys.PASSWORD_GENERATION]: settings
    });
  }

  async getAutoFillSettings() {
    const result = await chrome.storage.sync.get(this.storageKeys.AUTO_FILL_SETTINGS);
    return result[this.storageKeys.AUTO_FILL_SETTINGS] || this.defaultSettings.autoFillSettings;
  }

  async saveAutoFillSettings(settings) {
    await chrome.storage.sync.set({
      [this.storageKeys.AUTO_FILL_SETTINGS]: settings
    });
  }

  async getSecuritySettings() {
    const result = await chrome.storage.sync.get(this.storageKeys.SECURITY_SETTINGS);
    return result[this.storageKeys.SECURITY_SETTINGS] || this.defaultSettings.securitySettings;
  }

  async saveSecuritySettings(settings) {
    await chrome.storage.sync.set({
      [this.storageKeys.SECURITY_SETTINGS]: settings
    });
  }

  // Cache Management
  async getDomainCredentialsCache() {
    const result = await chrome.storage.local.get(this.storageKeys.DOMAIN_CREDENTIALS);
    return result[this.storageKeys.DOMAIN_CREDENTIALS] || {};
  }

  async saveDomainCredentialsCache(domain, credentials, ttl = 300000) { // 5 minutes default
    const cache = await this.getDomainCredentialsCache();
    cache[domain] = {
      credentials,
      timestamp: Date.now(),
      ttl
    };
    
    await chrome.storage.local.set({
      [this.storageKeys.DOMAIN_CREDENTIALS]: cache
    });
  }

  async clearDomainCredentialsCache(domain = null) {
    if (domain) {
      const cache = await this.getDomainCredentialsCache();
      delete cache[domain];
      await chrome.storage.local.set({
        [this.storageKeys.DOMAIN_CREDENTIALS]: cache
      });
    } else {
      await chrome.storage.local.set({
        [this.storageKeys.DOMAIN_CREDENTIALS]: {}
      });
    }
  }

  async getFormDetectionCache() {
    const result = await chrome.storage.local.get(this.storageKeys.FORM_DETECTION_CACHE);
    return result[this.storageKeys.FORM_DETECTION_CACHE] || {};
  }

  async saveFormDetectionCache(url, detectionData, ttl = 600000) { // 10 minutes default
    const cache = await this.getFormDetectionCache();
    const urlKey = this.createUrlKey(url);
    
    cache[urlKey] = {
      data: detectionData,
      timestamp: Date.now(),
      ttl
    };
    
    await chrome.storage.local.set({
      [this.storageKeys.FORM_DETECTION_CACHE]: cache
    });
  }

  async getSecurityDashboardCache() {
    const result = await chrome.storage.local.get(this.storageKeys.SECURITY_DASHBOARD_CACHE);
    const cache = result[this.storageKeys.SECURITY_DASHBOARD_CACHE];
    
    // Check if cache is valid (not older than 1 hour)
    if (cache && (Date.now() - cache.timestamp) < 3600000) {
      return cache.data;
    }
    
    return null;
  }

  async saveSecurityDashboardCache(dashboardData) {
    await chrome.storage.local.set({
      [this.storageKeys.SECURITY_DASHBOARD_CACHE]: {
        data: dashboardData,
        timestamp: Date.now()
      }
    });
  }

  // Session Management
  async getSessionData() {
    const result = await chrome.storage.local.get(this.storageKeys.SESSION_DATA);
    return result[this.storageKeys.SESSION_DATA] || {};
  }

  async saveSessionData(key, value) {
    const sessionData = await this.getSessionData();
    sessionData[key] = value;
    
    await chrome.storage.local.set({
      [this.storageKeys.SESSION_DATA]: sessionData
    });
  }

  async clearSessionData(key = null) {
    if (key) {
      const sessionData = await this.getSessionData();
      delete sessionData[key];
      await chrome.storage.local.set({
        [this.storageKeys.SESSION_DATA]: sessionData
      });
    } else {
      await chrome.storage.local.set({
        [this.storageKeys.SESSION_DATA]: {}
      });
    }
  }

  // Pending Operations
  async getPendingSaves() {
    const result = await chrome.storage.local.get(this.storageKeys.PENDING_SAVES);
    return result[this.storageKeys.PENDING_SAVES] || [];
  }

  async addPendingSave(saveData) {
    const pendingSaves = await this.getPendingSaves();
    pendingSaves.push({
      ...saveData,
      timestamp: Date.now(),
      id: this.generateId()
    });
    
    await chrome.storage.local.set({
      [this.storageKeys.PENDING_SAVES]: pendingSaves
    });
  }

  async removePendingSave(id) {
    const pendingSaves = await this.getPendingSaves();
    const filtered = pendingSaves.filter(save => save.id !== id);
    
    await chrome.storage.local.set({
      [this.storageKeys.PENDING_SAVES]: filtered
    });
  }

  async clearPendingSaves() {
    await chrome.storage.local.set({
      [this.storageKeys.PENDING_SAVES]: []
    });
  }

  // Activity Tracking
  async updateLastActivity() {
    await chrome.storage.local.set({
      [this.storageKeys.LAST_ACTIVITY]: Date.now()
    });
  }

  async getLastActivity() {
    const result = await chrome.storage.local.get(this.storageKeys.LAST_ACTIVITY);
    return result[this.storageKeys.LAST_ACTIVITY] || 0;
  }

  // Cache Cleanup
  async cleanupExpiredCache() {
    try {
      console.log('[Storage] Starting cache cleanup...');
      
      const now = Date.now();
      let cleaned = 0;

      // Clean domain credentials cache
      const domainCache = await this.getDomainCredentialsCache();
      const cleanedDomainCache = {};
      
      for (const [domain, data] of Object.entries(domainCache)) {
        if (now - data.timestamp < data.ttl) {
          cleanedDomainCache[domain] = data;
        } else {
          cleaned++;
        }
      }
      
      await chrome.storage.local.set({
        [this.storageKeys.DOMAIN_CREDENTIALS]: cleanedDomainCache
      });

      // Clean form detection cache
      const formCache = await this.getFormDetectionCache();
      const cleanedFormCache = {};
      
      for (const [url, data] of Object.entries(formCache)) {
        if (now - data.timestamp < data.ttl) {
          cleanedFormCache[url] = data;
        } else {
          cleaned++;
        }
      }
      
      await chrome.storage.local.set({
        [this.storageKeys.FORM_DETECTION_CACHE]: cleanedFormCache
      });

      console.log(`[Storage] Cache cleanup complete, removed ${cleaned} expired entries`);
      return cleaned;
    } catch (error) {
      console.error('[Storage] Cache cleanup failed:', error);
      return 0;
    }
  }

  // Storage Health
  async checkHealth() {
    try {
      // Check storage quota usage
      const usage = await chrome.storage.local.getBytesInUse();
      const quota = chrome.storage.local.QUOTA_BYTES || 10485760; // 10MB default
      const usagePercentage = (usage / quota) * 100;

      // Check if settings are intact
      const settings = await this.getAllSettings();
      const hasValidSettings = Object.keys(this.defaultSettings).every(key => 
        settings[key] && typeof settings[key] === 'object'
      );

      // Check cache validity
      const cacheKeys = [
        this.storageKeys.DOMAIN_CREDENTIALS,
        this.storageKeys.FORM_DETECTION_CACHE,
        this.storageKeys.SESSION_DATA
      ];
      
      const cacheData = await chrome.storage.local.get(cacheKeys);
      const hasValidCache = cacheKeys.every(key => cacheData[key] !== undefined);

      const health = {
        healthy: usagePercentage < 80 && hasValidSettings && hasValidCache,
        usagePercentage,
        usageBytes: usage,
        quotaBytes: quota,
        hasValidSettings,
        hasValidCache,
        timestamp: Date.now()
      };

      console.log('[Storage] Health check:', health);
      return health;
    } catch (error) {
      console.error('[Storage] Health check failed:', error);
      return { healthy: false, error: error.message };
    }
  }

  // Utility methods
  createUrlKey(url) {
    try {
      const urlObj = new URL(url);
      return `${urlObj.protocol}//${urlObj.host}${urlObj.pathname}`;
    } catch (error) {
      return url;
    }
  }

  generateId() {
    return 'id_' + Date.now() + '_' + Math.random().toString(36).substr(2, 9);
  }

  // Reset all data (for debugging/reset purposes)
  async resetAllData() {
    try {
      await chrome.storage.sync.clear();
      await chrome.storage.local.clear();
      await this.initializeStorage();
      console.log('[Storage] All data reset successfully');
    } catch (error) {
      console.error('[Storage] Reset failed:', error);
      throw error;
    }
  }
}