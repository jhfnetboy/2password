import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Plus, Search, Shield } from "lucide-react";
import VaultSetup from "./components/VaultSetup";
import PasswordList from "./components/PasswordList";
import AddPasswordModal from "./components/AddPasswordModal";
import Sidebar from "./components/Sidebar";
import Settings from "./components/Settings";
import { PasswordEntry } from "./types";

function App() {
  const [isVaultLoaded, setIsVaultLoaded] = useState(false);
  const [isLoading, setIsLoading] = useState(true);
  const [entries, setEntries] = useState<PasswordEntry[]>([]);
  const [searchQuery, setSearchQuery] = useState("");
  const [showAddModal, setShowAddModal] = useState(false);
  const [touchIdAvailable, setTouchIdAvailable] = useState(false);
  const [currentView, setCurrentView] = useState<'passwords' | 'settings'>('passwords');

  useEffect(() => {
    checkVaultStatus();
    checkTouchIdAvailability();
  }, []);

  const checkVaultStatus = async () => {
    try {
      const loaded = await invoke<boolean>("is_vault_loaded");
      setIsVaultLoaded(loaded);
      if (loaded) {
        await loadEntries();
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

  const handleVaultLoaded = () => {
    setIsVaultLoaded(true);
    loadEntries();
  };

  const handleAddEntry = async (
    title: string,
    username: string,
    password: string,
    url?: string,
    notes?: string,
    tags?: string[]
  ) => {
    try {
      await invoke("add_entry", { title, username, password, url, notes, tags });
      await loadEntries();
      setShowAddModal(false);
    } catch (error) {
      console.error("Failed to add entry:", error);
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
                  <h1 className="text-xl font-semibold text-gray-900">Password Vault</h1>
                  <div className="flex items-center text-sm text-gray-500">
                    <Shield className="h-4 w-4 mr-1" />
                    {entries.length} entries
                  </div>
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
        ) : (
          /* Settings View */
          <Settings 
            onBack={() => setCurrentView('passwords')} 
            touchIdAvailable={touchIdAvailable}
          />
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