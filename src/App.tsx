import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Plus, Search, Shield, LogOut } from "lucide-react";
import VaultSetup from "./components/VaultSetup";
import PasswordList from "./components/PasswordList";
import AddPasswordModal from "./components/AddPasswordModal";
import Sidebar from "./components/Sidebar";
import Settings from "./components/Settings";
import PasswordHealthDashboard from "./components/PasswordHealthDashboard";
import { PasswordEntry } from "./types";
import { close_vault } from "./utils/api";

function App() {
  const [isVaultLoaded, setIsVaultLoaded] = useState(false);
  const [isLoading, setIsLoading] = useState(true);
  const [entries, setEntries] = useState<PasswordEntry[]>([]);
  const [searchQuery, setSearchQuery] = useState("");
  const [showAddModal, setShowAddModal] = useState(false);
  const [touchIdAvailable, setTouchIdAvailable] = useState(false);
  const [currentView, setCurrentView] = useState<'passwords' | 'health' | 'settings' | 'security'>('passwords');
  const [vaultPath, setVaultPath] = useState<string>("");

  useEffect(() => {
    checkVaultStatus();
    checkTouchIdAvailability();
  }, []);

  const checkVaultStatus = async () => {
    console.log("🔍 App.checkVaultStatus called on startup");
    try {
      const loaded = await invoke<boolean>("is_vault_loaded");
      console.log("📊 Vault loaded status:", loaded);
      setIsVaultLoaded(loaded);
      if (loaded) {
        console.log("✅ Vault is loaded, getting details...");
        // Get vault status to retrieve the path
        try {
          const vaultStatus = await invoke<{loaded: boolean, path?: string}>("get_vault_status");
          console.log("📂 Vault status on startup:", vaultStatus);
          if (vaultStatus.path) {
            setVaultPath(vaultStatus.path);
          }
        } catch (statusError) {
          console.error("Failed to get vault status:", statusError);
        }
        await loadEntries();
      } else {
        console.log("❌ No vault loaded on startup");
      }
    } catch (error) {
      console.error("Failed to check vault status:", error);
    } finally {
      setIsLoading(false);
    }
  };

  const checkTouchIdAvailability = async () => {
    try {
      const available = await invoke<boolean>("check_touchid_available");
      setTouchIdAvailable(available);
    } catch (error) {
      console.error("Failed to check Touch ID availability:", error);
    }
  };

  const loadEntries = async () => {
    try {
      const entriesData = await invoke<PasswordEntry[]>("get_all_entries");
      setEntries(entriesData);
    } catch (error) {
      console.error("Failed to load entries:", error);
    }
  };

  const handleVaultLoaded = async () => {
    console.log("🎯 App.handleVaultLoaded called!");
    setIsVaultLoaded(true);
    console.log("🔄 Set isVaultLoaded to true");
    
    // Get vault status to retrieve the path
    try {
      console.log("📡 Getting vault status...");
      const vaultStatus = await invoke<{loaded: boolean, path?: string}>("get_vault_status");
      console.log("📊 Vault status:", vaultStatus);
      if (vaultStatus.path) {
        setVaultPath(vaultStatus.path);
        console.log("📁 Set vault path to:", vaultStatus.path);
      }
    } catch (statusError) {
      console.error("Failed to get vault status:", statusError);
    }
    
    console.log("📂 Loading entries...");
    await loadEntries();
    console.log("✅ handleVaultLoaded completed!");
  };

  // Extract vault name from path
  const getVaultName = (path: string): string => {
    if (!path) return "Unknown Vault";
    const fileName = path.split('/').pop() || path.split('\\').pop() || path;
    return fileName.replace(/\.[^/.]+$/, ""); // Remove file extension
  };

  const handleSwitchVault = async () => {
    console.log("🔄 Switching vault...");
    
    try {
      console.log("🔓 Closing current vault...");
      await close_vault();
      console.log("✅ Vault closed successfully");
      
      // Reset application state
      console.log("🔄 Resetting application state...");
      setIsVaultLoaded(false);
      setEntries([]);
      setSearchQuery("");
      setShowAddModal(false);
      setCurrentView('passwords');
      setVaultPath("");
      
      console.log("✅ Vault switch completed - showing vault selection");
    } catch (error) {
      console.error("❌ Failed to switch vault:", error);
      alert("Failed to switch vault: " + error);
    }
  };

  const handleAddEntry = async (
    title: string,
    username: string,
    password: string,
    url?: string,
    notes?: string,
    tags?: string[]
  ) => {
    console.log("🎯 App.handleAddEntry: Called with parameters:");
    console.log("   📝 title:", title);
    console.log("   👤 username:", username); 
    console.log("   🔑 password:", password ? "[PROTECTED]" : "(empty)");
    console.log("   🌐 url:", url);
    console.log("   📄 notes:", notes);
    console.log("   🏷️ tags:", tags);
    
    try {
      console.log("📡 Invoking 'add_entry' Tauri command...");
      await invoke("add_entry", { title, username, password, url, notes, tags });
      console.log("✅ add_entry command completed successfully");
      
      console.log("🔄 Reloading entries from vault...");
      await loadEntries();
      console.log("✅ Entries reloaded successfully");
      
      console.log("🚪 Closing add modal");
      setShowAddModal(false);
      console.log("✅ handleAddEntry completed successfully");
    } catch (error) {
      console.error("❌ Failed to add entry:", error);
      console.error("❌ Error details:", {
        name: error instanceof Error ? error.name : typeof error,
        message: error instanceof Error ? error.message : String(error),
        stack: error instanceof Error ? error.stack : undefined
      });
      alert("Failed to add entry: " + error);
    }
  };

  const handleDeleteEntry = async (entryId: string) => {
    try {
      await invoke("remove_entry", { entry_id: entryId });
      await loadEntries();
    } catch (error) {
      console.error("Failed to delete entry:", error);
      alert("Failed to delete entry: " + error);
    }
  };

  const handleSearch = async (query: string) => {
    setSearchQuery(query);
    if (query.trim() === "") {
      await loadEntries();
    } else {
      try {
        const searchResults = await invoke<PasswordEntry[]>("search_entries", {
          query,
        });
        setEntries(searchResults);
      } catch (error) {
        console.error("Failed to search entries:", error);
      }
    }
  };

  if (isLoading) {
    return (
      <div className="h-screen flex items-center justify-center bg-gray-50">
        <div className="text-center">
          <Shield className="h-12 w-12 text-primary-600 mx-auto mb-4" />
          <div className="text-lg font-medium text-gray-900">Loading 2Password...</div>
        </div>
      </div>
    );
  }

  if (!isVaultLoaded) {
    return <VaultSetup onVaultLoaded={handleVaultLoaded} />;
  }

  return (
    <div className="h-screen flex bg-gray-50">
      {/* Sidebar */}
      <Sidebar 
        touchIdAvailable={touchIdAvailable}
        onNewEntry={() => setShowAddModal(true)}
        onNavigate={(view) => setCurrentView(view)}
        currentView={currentView}
      />

      {/* Main Content */}
      <div className="flex-1 flex flex-col">
        {currentView === 'passwords' ? (
          <>
            {/* Header */}
            <header className="bg-white border-b border-gray-200 px-6 py-4">
              <div className="flex items-center justify-between">
                <div className="flex items-center space-x-4">
                  <div>
                    <h1 className="text-xl font-semibold text-gray-900">Password Vault</h1>
                    <p className="text-sm text-gray-600">{getVaultName(vaultPath)}</p>
                  </div>
                  <div className="flex items-center text-sm text-gray-500">
                    <Shield className="h-4 w-4 mr-1" />
                    {entries.length} entries
                  </div>
                  
                  {/* Switch Vault Button */}
                  <button
                    onClick={handleSwitchVault}
                    className="flex items-center space-x-2 px-3 py-1.5 text-sm text-gray-600 hover:text-red-600 hover:bg-red-50 rounded-lg transition-colors"
                    title="Switch to another vault"
                  >
                    <LogOut className="h-4 w-4 text-red-500" />
                    <span>Switch Vault</span>
                  </button>
                </div>
                
                <div className="flex items-center space-x-4">
                  {/* Search */}
                  <div className="relative">
                    <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-gray-400" />
                    <input
                      type="text"
                      placeholder="Search passwords..."
                      className="pl-10 pr-4 py-2 w-64 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
                      value={searchQuery}
                      onChange={(e) => handleSearch(e.target.value)}
                    />
                  </div>
                  
                  {/* Add Button */}
                  <button
                    onClick={() => setShowAddModal(true)}
                    className="btn btn-primary flex items-center space-x-2"
                  >
                    <Plus className="h-4 w-4" />
                    <span>Add Password</span>
                  </button>
                </div>
              </div>
            </header>

            {/* Password List */}
            <main className="flex-1 overflow-auto p-6">
              <PasswordList
                entries={entries}
                onDeleteEntry={handleDeleteEntry}
                searchQuery={searchQuery}
              />
            </main>
          </>
        ) : currentView === 'health' ? (
          /* Password Health Dashboard */
          <PasswordHealthDashboard 
            onBack={() => setCurrentView('passwords')} 
          />
        ) : currentView === 'settings' ? (
          /* Settings View */
          <Settings 
            onBack={() => setCurrentView('passwords')} 
            touchIdAvailable={touchIdAvailable}
          />
        ) : (
          /* Security View */
          <div className="flex-1 flex flex-col">
            <header className="bg-white border-b border-gray-200 px-6 py-4">
              <div className="flex items-center justify-between">
                <div className="flex items-center space-x-4">
                  <button
                    onClick={() => setCurrentView('passwords')}
                    className="text-gray-500 hover:text-gray-700"
                  >
                    ← Back
                  </button>
                  <h1 className="text-xl font-semibold text-gray-900">Security Center</h1>
                </div>
              </div>
            </header>
            <main className="flex-1 overflow-auto p-6">
              <div className="max-w-4xl mx-auto">
                <div className="bg-white rounded-lg shadow-sm border p-6">
                  <h2 className="text-lg font-medium text-gray-900 mb-4">Security Settings</h2>
                  <p className="text-gray-600">Security features are under development.</p>
                </div>
              </div>
            </main>
          </div>
        )}
      </div>

      {/* Add Password Modal */}
      {showAddModal && (
        <AddPasswordModal
          onAdd={handleAddEntry}
          onClose={() => setShowAddModal(false)}
        />
      )}
    </div>
  );
}

export default App;