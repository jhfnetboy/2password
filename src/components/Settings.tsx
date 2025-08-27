import { useState } from "react";
import { Shield, ArrowLeft, Save, Trash2 } from "lucide-react";

interface SettingsProps {
  onBack: () => void;
  touchIdAvailable: boolean;
}

export default function Settings({ onBack, touchIdAvailable }: SettingsProps) {
  const [settings, setSettings] = useState({
    autoLock: 15,
    showPasswordHints: true,
    enableTouchID: touchIdAvailable,
    clearClipboard: 60,
    theme: 'system' as 'light' | 'dark' | 'system',
  });

  const [showConfirmClear, setShowConfirmClear] = useState(false);

  const handleSave = () => {
    // In a real app, this would save to backend
    console.log("Saving settings:", settings);
    alert("Settings saved successfully!");
  };

  const handleClearVault = () => {
    if (showConfirmClear) {
      // In a real app, this would clear the vault
      console.log("Clearing vault...");
      alert("Vault cleared! (This is a demo)");
      setShowConfirmClear(false);
    } else {
      setShowConfirmClear(true);
    }
  };

  return (
    <div className="flex flex-col h-full bg-white">
      {/* Header */}
      <div className="flex items-center p-6 border-b border-gray-200">
        <button
          onClick={onBack}
          className="p-2 hover:bg-gray-100 rounded-lg transition-colors mr-3"
        >
          <ArrowLeft className="h-5 w-5 text-gray-600" />
        </button>
        <div className="flex items-center">
          <Shield className="h-6 w-6 text-primary-600 mr-3" />
          <h1 className="text-xl font-semibold text-gray-900">Settings</h1>
        </div>
      </div>

      {/* Content */}
      <div className="flex-1 overflow-auto p-6 space-y-8">
        {/* Security Settings */}
        <section>
          <h2 className="text-lg font-medium text-gray-900 mb-4">Security</h2>
          <div className="space-y-4">
            {/* Auto Lock */}
            <div className="flex items-center justify-between">
              <div>
                <label className="text-sm font-medium text-gray-700">Auto Lock</label>
                <p className="text-xs text-gray-500">Lock vault after inactivity</p>
              </div>
              <select
                value={settings.autoLock}
                onChange={(e) => setSettings(prev => ({ ...prev, autoLock: Number(e.target.value) }))}
                className="select text-sm"
              >
                <option value={5}>5 minutes</option>
                <option value={15}>15 minutes</option>
                <option value={30}>30 minutes</option>
                <option value={60}>1 hour</option>
                <option value={0}>Never</option>
              </select>
            </div>

            {/* Touch ID */}
            {touchIdAvailable && (
              <div className="flex items-center justify-between">
                <div>
                  <label className="text-sm font-medium text-gray-700">Touch ID</label>
                  <p className="text-xs text-gray-500">Use Touch ID to unlock vault</p>
                </div>
                <label className="relative inline-flex items-center cursor-pointer">
                  <input
                    type="checkbox"
                    checked={settings.enableTouchID}
                    onChange={(e) => setSettings(prev => ({ ...prev, enableTouchID: e.target.checked }))}
                    className="sr-only peer"
                  />
                  <div className="w-11 h-6 bg-gray-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-primary-300 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-primary-600"></div>
                </label>
              </div>
            )}

            {/* Clear Clipboard */}
            <div className="flex items-center justify-between">
              <div>
                <label className="text-sm font-medium text-gray-700">Clear Clipboard</label>
                <p className="text-xs text-gray-500">Auto-clear copied passwords</p>
              </div>
              <select
                value={settings.clearClipboard}
                onChange={(e) => setSettings(prev => ({ ...prev, clearClipboard: Number(e.target.value) }))}
                className="select text-sm"
              >
                <option value={30}>30 seconds</option>
                <option value={60}>1 minute</option>
                <option value={300}>5 minutes</option>
                <option value={0}>Never</option>
              </select>
            </div>
          </div>
        </section>

        {/* Display Settings */}
        <section>
          <h2 className="text-lg font-medium text-gray-900 mb-4">Display</h2>
          <div className="space-y-4">
            {/* Show Password Hints */}
            <div className="flex items-center justify-between">
              <div>
                <label className="text-sm font-medium text-gray-700">Show Password Hints</label>
                <p className="text-xs text-gray-500">Display password strength indicators</p>
              </div>
              <label className="relative inline-flex items-center cursor-pointer">
                <input
                  type="checkbox"
                  checked={settings.showPasswordHints}
                  onChange={(e) => setSettings(prev => ({ ...prev, showPasswordHints: e.target.checked }))}
                  className="sr-only peer"
                />
                <div className="w-11 h-6 bg-gray-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-primary-300 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-primary-600"></div>
              </label>
            </div>

            {/* Theme */}
            <div className="flex items-center justify-between">
              <div>
                <label className="text-sm font-medium text-gray-700">Theme</label>
                <p className="text-xs text-gray-500">Choose app appearance</p>
              </div>
              <select
                value={settings.theme}
                onChange={(e) => setSettings(prev => ({ ...prev, theme: e.target.value as 'light' | 'dark' | 'system' }))}
                className="select text-sm"
              >
                <option value="light">Light</option>
                <option value="dark">Dark</option>
                <option value="system">System</option>
              </select>
            </div>
          </div>
        </section>

        {/* Data Management */}
        <section>
          <h2 className="text-lg font-medium text-gray-900 mb-4">Data Management</h2>
          <div className="space-y-4">
            {/* Clear Vault */}
            <div className="p-4 bg-red-50 border border-red-200 rounded-lg">
              <h3 className="text-sm font-medium text-red-900 mb-2">Danger Zone</h3>
              <p className="text-xs text-red-700 mb-3">
                This action will permanently delete all passwords in your vault. This cannot be undone.
              </p>
              <button
                onClick={handleClearVault}
                className={`flex items-center space-x-2 px-3 py-2 text-sm rounded-lg transition-colors ${
                  showConfirmClear
                    ? 'bg-red-600 text-white hover:bg-red-700'
                    : 'bg-red-100 text-red-700 hover:bg-red-200'
                }`}
              >
                <Trash2 className="h-4 w-4" />
                <span>{showConfirmClear ? 'Confirm Clear Vault' : 'Clear Vault'}</span>
              </button>
              {showConfirmClear && (
                <button
                  onClick={() => setShowConfirmClear(false)}
                  className="ml-2 px-3 py-2 text-sm bg-gray-100 text-gray-700 hover:bg-gray-200 rounded-lg transition-colors"
                >
                  Cancel
                </button>
              )}
            </div>
          </div>
        </section>
      </div>

      {/* Footer */}
      <div className="p-6 border-t border-gray-200">
        <button
          onClick={handleSave}
          className="w-full btn btn-primary flex items-center justify-center space-x-2"
        >
          <Save className="h-4 w-4" />
          <span>Save Settings</span>
        </button>
      </div>
    </div>
  );
}