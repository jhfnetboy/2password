import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { Shield, FolderOpen } from "lucide-react";

interface VaultSetupProps {
  onVaultLoaded: () => void;
}

export default function VaultSetup({ onVaultLoaded }: VaultSetupProps) {
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleLoadExistingVault = async () => {
    try {
      const result = await open({
        filters: [{
          name: "2Password Vault",
          extensions: ["enc"]
        }],
        title: "Select your password vault file",
      });
      
      if (result) {
        const password = prompt("Enter your master password:");
        if (password) {
          setIsLoading(true);
          setError(null);
          
          try {
            await invoke("load_vault", { 
              path: result, 
              password 
            });
            onVaultLoaded();
          } catch (error) {
            setError("Failed to load vault. Please check your password.");
          } finally {
            setIsLoading(false);
          }
        }
      }
    } catch (error) {
      setError("Failed to load vault: " + error);
      setIsLoading(false);
    }
  };

  const handleCreateNewVault = () => {
    // For demo purposes, just call greet and load vault
    invoke("greet").then(() => {
      onVaultLoaded();
    });
  };

  return (
    <div className="h-screen flex items-center justify-center bg-gradient-to-br from-blue-50 to-indigo-50">
      <div className="max-w-md w-full mx-4">
        <div className="text-center mb-8">
          <Shield className="h-16 w-16 text-blue-600 mx-auto mb-4" />
          <h1 className="text-3xl font-bold text-gray-900 mb-2">Welcome to 2Password</h1>
          <p className="text-gray-600">Secure password management with Touch ID integration</p>
        </div>

        <div className="space-y-4">
          <button
            onClick={handleCreateNewVault}
            className="w-full p-4 bg-white rounded-lg shadow-sm border border-gray-200 hover:border-blue-300 hover:shadow-md transition-all group"
          >
            <div className="flex items-center">
              <div className="w-12 h-12 bg-blue-100 rounded-lg flex items-center justify-center mr-4 group-hover:bg-blue-200 transition-colors">
                <Shield className="h-6 w-6 text-blue-600" />
              </div>
              <div className="text-left">
                <div className="font-medium text-gray-900">Create New Vault</div>
                <div className="text-sm text-gray-500">Set up a new password vault</div>
              </div>
            </div>
          </button>

          <button
            onClick={handleLoadExistingVault}
            disabled={isLoading}
            className="w-full p-4 bg-white rounded-lg shadow-sm border border-gray-200 hover:border-blue-300 hover:shadow-md transition-all group disabled:opacity-50"
          >
            <div className="flex items-center">
              <div className="w-12 h-12 bg-gray-100 rounded-lg flex items-center justify-center mr-4 group-hover:bg-gray-200 transition-colors">
                <FolderOpen className="h-6 w-6 text-gray-600" />
              </div>
              <div className="text-left">
                <div className="font-medium text-gray-900">Open Existing Vault</div>
                <div className="text-sm text-gray-500">Load your existing password vault</div>
              </div>
            </div>
          </button>
        </div>

        {error && (
          <div className="mt-4 p-3 bg-red-50 border border-red-200 rounded-lg text-red-700 text-sm">
            {error}
          </div>
        )}

        {isLoading && (
          <div className="mt-4 text-center">
            <div className="text-sm text-gray-600">Loading...</div>
          </div>
        )}
      </div>
    </div>
  );
}