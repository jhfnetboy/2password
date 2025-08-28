import { useState, useRef, useEffect } from "react";
import { X, Eye, EyeOff, Fingerprint, User } from "lucide-react";
import { invoke } from '@tauri-apps/api/core';
import PasskeyAuth from "./PasskeyAuth";

interface EnhancedPasswordModalProps {
  isOpen: boolean;
  title: string;
  description?: string;
  onSubmit: (data: {
    password: string;
    username?: string;
    authToken?: string;
    usePasskey: boolean;
  }) => void;
  onClose: () => void;
  isLoading?: boolean;
  minLength?: number;
  mode: 'create' | 'load';
  showPasskeyOption?: boolean;
  enforcePasskey?: boolean;
}

export default function EnhancedPasswordModal({
  isOpen,
  title,
  description,
  onSubmit,
  onClose,
  isLoading = false,
  minLength = 8,
  mode,
  showPasskeyOption = true,
  enforcePasskey = false,
}: EnhancedPasswordModalProps) {
  const [password, setPassword] = useState("");
  const [username, setUsername] = useState("");
  const [showPassword, setShowPassword] = useState(false);
  const [error, setError] = useState("");
  const [usePasskey, setUsePasskey] = useState(enforcePasskey);
  const [authToken, setAuthToken] = useState<string>("");
  const [passkeyAvailable, setPasskeyAvailable] = useState(false);
  
  const dialogRef = useRef<HTMLDialogElement>(null);
  const passwordInputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (isOpen && showPasskeyOption) {
      checkPasskeyAvailability();
    }
  }, [isOpen, showPasskeyOption]);

  useEffect(() => {
    const dialog = dialogRef.current;
    if (!dialog) return;

    if (isOpen) {
      dialog.showModal();
      setTimeout(() => {
        passwordInputRef.current?.focus();
      }, 100);
    } else {
      dialog.close();
      resetForm();
    }
  }, [isOpen]);

  const resetForm = () => {
    setPassword("");
    setUsername("");
    setError("");
    setShowPassword(false);
    setUsePasskey(false);
    setAuthToken("");
  };

  const checkPasskeyAvailability = async () => {
    try {
      const available = await invoke<boolean>('check_passkey_available');
      setPasskeyAvailable(available);
    } catch (error) {
      console.error('Failed to check Passkey availability:', error);
      setPasskeyAvailable(false);
    }
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    
    if (password.length < minLength) {
      setError(`Password must be at least ${minLength} characters long`);
      return;
    }

    if ((usePasskey || enforcePasskey) && mode === 'create' && !username.trim()) {
      setError("Username is required when using Passkey");
      return;
    }

    if ((usePasskey || enforcePasskey) && !authToken) {
      setError("Please complete Touch ID authentication");
      return;
    }

    setError("");
    onSubmit({
      password,
      username: username.trim() || undefined,
      authToken: authToken || undefined,
      usePasskey,
    });
  };

  const handleClose = () => {
    if (!isLoading) {
      onClose();
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Escape" && !isLoading) {
      handleClose();
    }
  };

  const handlePasskeyAuthSuccess = (token: string) => {
    setAuthToken(token);
    setError("");
  };

  const handlePasskeyAuthError = (errorMsg: string) => {
    setError(errorMsg);
    setAuthToken("");
  };

  const handlePasskeyRegisterSuccess = () => {
    setError("");
  };

  return (
    <dialog
      ref={dialogRef}
      className="backdrop:bg-black backdrop:bg-opacity-50 bg-transparent p-0 max-w-lg w-full rounded-lg shadow-xl"
      onKeyDown={handleKeyDown}
    >
      <div className="bg-white rounded-lg p-6 w-full max-h-[90vh] overflow-y-auto">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-xl font-semibold text-gray-900">{title}</h2>
          <button
            onClick={handleClose}
            disabled={isLoading}
            className="text-gray-400 hover:text-gray-600 p-1 rounded-full hover:bg-gray-100 transition-colors disabled:opacity-50"
            aria-label="Close dialog"
          >
            <X className="h-5 w-5" />
          </button>
        </div>

        {description && (
          <p className="text-gray-600 mb-6">{description}</p>
        )}

        <form onSubmit={handleSubmit} className="space-y-6">
          {/* Passkey Required Notice or Toggle */}
          {showPasskeyOption && passkeyAvailable && (
            <div className={`p-3 rounded-lg border ${enforcePasskey ? 'bg-green-50 border-green-200' : 'bg-blue-50 border-blue-200'}`}>
              <div className="flex items-center justify-between">
                <div className="flex items-center space-x-3">
                  <Fingerprint className={`h-5 w-5 ${enforcePasskey ? 'text-green-600' : 'text-blue-600'}`} />
                  <div>
                    <p className={`text-sm font-medium ${enforcePasskey ? 'text-green-900' : 'text-blue-900'}`}>
                      {enforcePasskey ? 'Touch ID Required for Security' : 'Use Touch ID for Enhanced Security'}
                    </p>
                    <p className={`text-xs ${enforcePasskey ? 'text-green-700' : 'text-blue-700'}`}>
                      {enforcePasskey ? 'Mandatory biometric authentication with simple password' : 'Combine biometric authentication with simple password for ultimate security'}
                    </p>
                  </div>
                </div>
                {!enforcePasskey && (
                  <label className="relative inline-flex items-center cursor-pointer">
                    <input
                      type="checkbox"
                      checked={usePasskey}
                      onChange={(e) => setUsePasskey(e.target.checked)}
                      disabled={isLoading}
                      className="sr-only peer"
                    />
                    <div className="w-11 h-6 bg-gray-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-blue-300 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-blue-600"></div>
                  </label>
                )}
                {enforcePasskey && (
                  <div className="px-3 py-1 bg-green-100 text-green-800 text-xs font-medium rounded-full">
                    Required
                  </div>
                )}
              </div>
            </div>
          )}

          {/* Username (only shown when using Passkey for creation) */}
          {(usePasskey || enforcePasskey) && mode === 'create' && (
            <div>
              <label htmlFor="username" className="block text-sm font-medium text-gray-700 mb-2">
                Username
              </label>
              <div className="relative">
                <input
                  id="username"
                  type="text"
                  value={username}
                  onChange={(e) => setUsername(e.target.value)}
                  disabled={isLoading}
                  className="w-full pl-10 pr-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 disabled:opacity-50 disabled:cursor-not-allowed"
                  placeholder="Enter your username"
                  autoComplete="username"
                />
                <User className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-gray-400" />
              </div>
            </div>
          )}

          {/* Simple Password */}
          <div>
            <label htmlFor="password" className="block text-sm font-medium text-gray-700 mb-2">
              {(usePasskey || enforcePasskey) ? 'Simple Password' : 'Master Password'}
            </label>
            <div className="relative">
              <input
                ref={passwordInputRef}
                id="password"
                type={showPassword ? "text" : "password"}
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                disabled={isLoading}
                className="w-full px-3 py-2 pr-10 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 disabled:opacity-50 disabled:cursor-not-allowed"
                placeholder={(usePasskey || enforcePasskey) ? "Enter a simple, memorable password" : "Enter your master password"}
                autoComplete={(usePasskey || enforcePasskey) ? "off" : "current-password"}
              />
              <button
                type="button"
                onClick={() => setShowPassword(!showPassword)}
                disabled={isLoading}
                className="absolute right-3 top-1/2 transform -translate-y-1/2 text-gray-400 hover:text-gray-600 disabled:opacity-50"
                aria-label={showPassword ? "Hide password" : "Show password"}
              >
                {showPassword ? (
                  <EyeOff className="h-4 w-4" />
                ) : (
                  <Eye className="h-4 w-4" />
                )}
              </button>
            </div>
            {(usePasskey || enforcePasskey) && (
              <p className="mt-1 text-xs text-gray-500">
                With Touch ID, this password only needs to be simple and memorable. The system will automatically enhance security.
              </p>
            )}
          </div>

          {/* Passkey Authentication */}
          {(usePasskey || enforcePasskey) && password.length >= minLength && (
            <div className="border border-gray-200 rounded-lg p-4">
              <PasskeyAuth
                username={username || undefined}
                onAuthSuccess={handlePasskeyAuthSuccess}
                onAuthError={handlePasskeyAuthError}
                onRegisterSuccess={handlePasskeyRegisterSuccess}
                showRegisterOption={mode === 'create'}
                className=""
              />
            </div>
          )}

          {/* Error Display */}
          {error && (
            <div className="p-3 bg-red-50 border border-red-200 rounded-lg">
              <p className="text-sm text-red-600">{error}</p>
            </div>
          )}

          {/* Buttons */}
          <div className="flex space-x-3">
            <button
              type="button"
              onClick={handleClose}
              disabled={isLoading}
              className="flex-1 px-4 py-2 text-gray-700 bg-gray-100 rounded-lg hover:bg-gray-200 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            >
              Cancel
            </button>
            <button
              type="submit"
              disabled={
                isLoading || 
                password.length === 0 || 
                ((usePasskey || enforcePasskey) && mode === 'create' && !username.trim()) ||
                ((usePasskey || enforcePasskey) && !authToken)
              }
              className="flex-1 px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center"
            >
              {isLoading ? (
                <div className="animate-spin rounded-full h-4 w-4 border-2 border-white border-t-transparent"></div>
              ) : (
                mode === 'create' ? 'Create' : 'Open'
              )}
            </button>
          </div>
        </form>

        {/* Info Section */}
        {(usePasskey || enforcePasskey) && (
          <div className="mt-6 p-4 bg-gray-50 rounded-lg">
            <h4 className="text-sm font-medium text-gray-900 mb-2">Multi-Factor Security Architecture</h4>
            <p className="text-xs text-gray-600 leading-relaxed">
              Your master key is generated from these factors:<br/>
              • Simple Password (you set)<br/>
              • Touch ID Authentication Token (biometric)<br/>
              • iCloud ID Hash (device binding)<br/>
              • Random Salt (system generated)<br/><br/>
              Even if someone knows your simple password, they cannot decrypt your data without your device and biometric authentication.
            </p>
          </div>
        )}
      </div>
    </dialog>
  );
}