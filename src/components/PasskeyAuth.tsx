import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Fingerprint, Check, X, AlertCircle } from 'lucide-react';

interface PasskeyStatus {
  available: boolean;
  registered: boolean;
  username?: string;
}

interface PasskeyAuthResult {
  success: boolean;
  auth_token?: string;
  error?: string;
}

interface PasskeyAuthProps {
  username?: string;
  onAuthSuccess: (authToken: string) => void;
  onAuthError: (error: string) => void;
  onRegisterSuccess?: () => void;
  showRegisterOption?: boolean;
  className?: string;
}

export const PasskeyAuth: React.FC<PasskeyAuthProps> = ({
  username = '',
  onAuthSuccess,
  onAuthError,
  onRegisterSuccess,
  showRegisterOption = true,
  className = ''
}) => {
  const [status, setStatus] = useState<PasskeyStatus>({ available: false, registered: false });
  const [isAuthenticating, setIsAuthenticating] = useState(false);
  const [isRegistering, setIsRegistering] = useState(false);
  const [error, setError] = useState<string>('');

  useEffect(() => {
    checkPasskeyStatus();
  }, []);

  const checkPasskeyStatus = async () => {
    try {
      const [available, statusResult] = await Promise.all([
        invoke<boolean>('check_passkey_available'),
        invoke<PasskeyStatus>('get_passkey_status')
      ]);

      setStatus({
        available,
        registered: statusResult.registered,
        username: statusResult.username
      });
    } catch (error) {
      console.error('Failed to check Passkey status:', error);
      setError('Unable to check Passkey status');
    }
  };

  const handleAuthenticate = async () => {
    if (!status.available) {
      onAuthError('Touch ID/Face ID is not available on this device');
      return;
    }

    setIsAuthenticating(true);
    setError('');

    try {
      const result = await invoke<PasskeyAuthResult>('authenticate_passkey', {
        username: username || status.username || null
      });

      if (result.success && result.auth_token) {
        onAuthSuccess(result.auth_token);
      } else {
        const errorMsg = result.error || 'Biometric authentication failed';
        setError(errorMsg);
        onAuthError(errorMsg);
      }
    } catch (error) {
      const errorMsg = `Authentication failed: ${error}`;
      setError(errorMsg);
      onAuthError(errorMsg);
    } finally {
      setIsAuthenticating(false);
    }
  };

  const handleRegister = async () => {
    if (!status.available) {
      setError('Touch ID/Face ID is not available on this device');
      return;
    }

    if (!username) {
      setError('Username is required to register Passkey');
      return;
    }

    setIsRegistering(true);
    setError('');

    try {
      const success = await invoke<boolean>('register_passkey', { username });

      if (success) {
        await checkPasskeyStatus(); // Refresh status
        onRegisterSuccess?.();
      } else {
        setError('Passkey registration failed');
      }
    } catch (error) {
      const errorMsg = `Registration failed: ${error}`;
      setError(errorMsg);
    } finally {
      setIsRegistering(false);
    }
  };

  if (!status.available) {
    return (
      <div className={`flex items-center space-x-3 p-4 bg-yellow-50 border border-yellow-200 rounded-lg ${className}`}>
        <AlertCircle className="h-5 w-5 text-yellow-600 flex-shrink-0" />
        <div className="text-sm text-yellow-800">
          <p className="font-medium">Biometric Authentication Unavailable</p>
          <p>This device does not support Touch ID or Face ID</p>
        </div>
      </div>
    );
  }

  return (
    <div className={`space-y-4 ${className}`}>
      {/* Passkey Status */}
      <div className="flex items-center space-x-3">
        <Fingerprint className="h-6 w-6 text-blue-600" />
        <div>
          <h3 className="text-sm font-medium text-gray-900">Biometric Authentication</h3>
          <p className="text-xs text-gray-500">
            {status.registered 
              ? `Passkey registered for ${status.username || 'user'}`
              : 'Passkey not registered'
            }
          </p>
        </div>
        {status.registered && (
          <Check className="h-4 w-4 text-green-500" />
        )}
      </div>

      {/* Authentication Button */}
      {status.registered && (
        <button
          onClick={handleAuthenticate}
          disabled={isAuthenticating}
          className="w-full flex items-center justify-center space-x-2 px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
        >
          <Fingerprint className="h-4 w-4" />
          <span>
            {isAuthenticating ? 'Authenticating...' : 'Authenticate with Touch ID'}
          </span>
        </button>
      )}

      {/* Registration Button */}
      {!status.registered && showRegisterOption && username && (
        <button
          onClick={handleRegister}
          disabled={isRegistering}
          className="w-full flex items-center justify-center space-x-2 px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
        >
          <Fingerprint className="h-4 w-4" />
          <span>
            {isRegistering ? 'Registering...' : 'Register Touch ID'}
          </span>
        </button>
      )}

      {/* Error Display */}
      {error && (
        <div className="flex items-center space-x-2 p-3 bg-red-50 border border-red-200 rounded-lg">
          <X className="h-4 w-4 text-red-600 flex-shrink-0" />
          <p className="text-sm text-red-800">{error}</p>
        </div>
      )}

      {/* Info Text */}
      <div className="text-xs text-gray-500">
        <p>
          Use Touch ID or Face ID for secure authentication.
          This combines with your simple password to generate a strong master key.
        </p>
      </div>
    </div>
  );
};

export default PasskeyAuth;