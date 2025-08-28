import { useState } from "react";
import { open, save } from "@tauri-apps/plugin-dialog";
import { Shield, FolderOpen } from "lucide-react";
import { load_vault, create_vault, add_entry } from "../utils/api";
import PasswordModal from "./PasswordModal";

interface VaultSetupProps {
  onVaultLoaded: () => void;
}

type PasswordModalType = 'none' | 'create' | 'load';

export default function VaultSetup({ onVaultLoaded }: VaultSetupProps) {
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [passwordModal, setPasswordModal] = useState<PasswordModalType>('none');
  const [pendingVaultPath, setPendingVaultPath] = useState<string>("");


  // Add sample entries to demo vault
  const addSampleEntries = async () => {
    const sampleEntries = [
      {
        title: "Gmail Account",
        username: "demo@gmail.com",
        password: "SecurePass123!",
        url: "https://gmail.com",
        notes: "Personal email account",
        tags: ["email", "personal"]
      },
      {
        title: "GitHub",
        username: "demouser",
        password: "GitHubSecure456#",
        url: "https://github.com",
        notes: "Development repository",
        tags: ["development", "work"]
      },
      {
        title: "Banking App",
        username: "demo_user_123",
        password: "BankSecure789$",
        url: "https://bank.example.com",
        notes: "Online banking access",
        tags: ["banking", "finance"]
      },
      {
        title: "Netflix",
        username: "demo@email.com",
        password: "StreamPass999@",
        url: "https://netflix.com",
        notes: "Streaming service account",
        tags: ["entertainment", "streaming"]
      },
      {
        title: "Amazon AWS",
        username: "demo-admin",
        password: "CloudSecure2024!",
        url: "https://aws.amazon.com",
        notes: "Cloud services management",
        tags: ["cloud", "work", "aws"]
      }
    ];

    for (const entry of sampleEntries) {
      try {
        await add_entry(
          entry.title,
          entry.username,
          entry.password,
          entry.url,
          entry.notes,
          entry.tags
        );
        console.log(`✅ Added sample entry: ${entry.title}`);
      } catch (error) {
        console.error(`❌ Failed to add sample entry ${entry.title}:`, error);
      }
    }
  };

  const handleLoadExistingVault = async () => {
    console.log("🔵 LOAD EXISTING VAULT BUTTON CLICKED!");
    try {
      const result = await open({
        filters: [{
          name: "2Password Vault",
          extensions: ["enc"]
        }],
        title: "Select your password vault file",
      });
      
      if (result) {
        setPendingVaultPath(result);
        setPasswordModal('load');
      }
    } catch (error) {
      setError("Failed to open file dialog: " + error);
    }
  };

  const handleCreateNewVault = async () => {
    console.log("🔴 CREATE NEW VAULT BUTTON CLICKED!");
    try {
      // Get save location first
      const path = await save({
        filters: [{
          name: "2Password Vault",
          extensions: ["enc"]
        }],
        title: "Save new vault file",
        defaultPath: "MyVault.enc"
      });
      
      if (path) {
        setPendingVaultPath(path);
        setPasswordModal('create');
      }
    } catch (error) {
      setError("Failed to open save dialog: " + error);
    }
  };

  const handlePasswordSubmit = async (password: string) => {
    setIsLoading(true);
    setError(null);

    try {
      if (passwordModal === 'create') {
        // Create new vault
        console.log("🔴 Creating new vault with modal password");
        await create_vault(pendingVaultPath, password);
        await load_vault(pendingVaultPath, password);
        onVaultLoaded();
      } else if (passwordModal === 'load') {
        // Load existing vault
        console.log("🔵 Loading existing vault with modal password");
        await load_vault(pendingVaultPath, password);
        onVaultLoaded();
      }
    } catch (error) {
      const errorMessage = passwordModal === 'create' 
        ? "Failed to create vault. Please check your inputs."
        : "Failed to load vault. Please check your password.";
      setError(errorMessage + " Error: " + error);
    } finally {
      setIsLoading(false);
      setPasswordModal('none');
      setPendingVaultPath("");
    }
  };

  const handlePasswordModalClose = () => {
    if (!isLoading) {
      setPasswordModal('none');
      setPendingVaultPath("");
    }
  };

  const handleCreateDemoVault = async () => {
    console.log("🟢 DEMO VAULT BUTTON CLICKED!");
    try {
      setIsLoading(true);
      setError(null);
      
      console.log("🚀 Opening demo vault...");
      
      // Get user's home directory for demo vault
      const { homeDir } = await import("@tauri-apps/api/path");
      const homePath = await homeDir();
      const demoPath = `${homePath}/Documents/demo-vault.enc`;
      const defaultPassword = "demo123";
      
      console.log("   📂 Path:", demoPath);
      console.log("   🔑 Password: demo123");
      
      try {
        // First try to load existing demo vault
        console.log("📂 Trying to load existing demo vault...");
        const loadResult = await load_vault(demoPath, defaultPassword);
        console.log("✅ Existing demo vault loaded successfully! Result:", loadResult);
      } catch (loadError) {
        console.log("⚠️ Demo vault doesn't exist or failed to load, creating new one...");
        console.log("📡 Calling create_vault function...");
        
        // Create the vault if it doesn't exist
        const createResult = await create_vault(demoPath, defaultPassword);
        console.log("✅ Demo vault created successfully! Result:", createResult);
        
        // Load the newly created vault
        console.log("📂 Loading newly created demo vault...");
        const loadResult = await load_vault(demoPath, defaultPassword);
        console.log("✅ Demo vault loaded successfully! Result:", loadResult);
        
        // Add sample entries to the new demo vault
        console.log("📝 Adding sample entries to demo vault...");
        await addSampleEntries();
        console.log("✅ Sample entries added to demo vault!");
      }
      
      console.log("🔔 Notifying parent component...");
      // Notify parent component
      onVaultLoaded();
      console.log("✅ handleCreateDemoVault completed successfully!");
    } catch (error) {
      console.error("❌ Failed to create demo vault:", error);
      console.error("❌ Error details:", {
        name: error instanceof Error ? error.name : typeof error,
        message: error instanceof Error ? error.message : String(error),
        stack: error instanceof Error ? error.stack : undefined
      });
      setError("Failed to create demo vault: " + error);
    } finally {
      console.log("🏁 Setting isLoading to false");
      setIsLoading(false);
    }
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
            onClick={handleCreateDemoVault}
            disabled={isLoading}
            className="w-full p-4 bg-gradient-to-r from-green-500 to-green-600 text-white rounded-lg shadow-sm hover:shadow-md transition-all group disabled:opacity-50"
          >
            <div className="flex items-center">
              <div className="w-12 h-12 bg-white bg-opacity-20 rounded-lg flex items-center justify-center mr-4 group-hover:bg-opacity-30 transition-colors">
                <Shield className="h-6 w-6 text-white" />
              </div>
              <div className="text-left">
                <div className="font-medium text-white">Open Demo Vault</div>
                <div className="text-sm text-green-100">⚠️ For testing - Creates if not exists</div>
                <div className="text-xs text-green-200">Password: demo123</div>
              </div>
            </div>
          </button>

          <button
            onClick={handleCreateNewVault}
            disabled={isLoading}
            className="w-full p-4 bg-white rounded-lg shadow-sm border border-gray-200 hover:border-blue-300 hover:shadow-md transition-all group disabled:opacity-50"
          >
            <div className="flex items-center">
              <div className="w-12 h-12 bg-blue-100 rounded-lg flex items-center justify-center mr-4 group-hover:bg-blue-200 transition-colors">
                <Shield className="h-6 w-6 text-blue-600" />
              </div>
              <div className="text-left">
                <div className="font-medium text-gray-900">Create New Vault</div>
                <div className="text-sm text-gray-500">Set up a new password vault with master password</div>
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

        {/* Password Modal */}
        <PasswordModal
          isOpen={passwordModal !== 'none'}
          title={passwordModal === 'create' ? 'Create New Vault' : 'Open Existing Vault'}
          description={
            passwordModal === 'create' 
              ? 'Enter a master password for your new vault. This password will be used to encrypt all your data.'
              : 'Enter the master password to unlock your vault.'
          }
          onSubmit={handlePasswordSubmit}
          onClose={handlePasswordModalClose}
          isLoading={isLoading}
          minLength={passwordModal === 'create' ? 8 : 1}
        />
      </div>
    </div>
  );
}