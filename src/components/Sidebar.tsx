import { Shield, Plus, Settings, Lock, Fingerprint, BarChart3 } from "lucide-react";

interface SidebarProps {
  touchIdAvailable: boolean;
  onNewEntry: () => void;
  onNavigate: (view: 'passwords' | 'health' | 'settings' | 'security') => void;
  currentView: 'passwords' | 'health' | 'settings' | 'security';
}

export default function Sidebar({ touchIdAvailable, onNewEntry, onNavigate, currentView }: SidebarProps) {
  return (
    <div className="sidebar">
      <div className="p-6">
        {/* Logo */}
        <div className="flex items-center mb-8">
          <Shield className="h-8 w-8 text-primary-600 mr-3" />
          <span className="text-xl font-bold text-gray-900">2Password</span>
        </div>

        {/* Quick Actions */}
        <div className="space-y-2 mb-8">
          <button
            onClick={onNewEntry}
            className="w-full flex items-center px-3 py-2 text-sm font-medium text-primary-600 bg-primary-50 rounded-lg hover:bg-primary-100 transition-colors"
          >
            <Plus className="h-4 w-4 mr-3" />
            Add Password
          </button>
        </div>

        {/* Navigation */}
        <nav className="space-y-1">
          <button 
            onClick={() => onNavigate('passwords')}
            className={`w-full flex items-center px-3 py-2 text-sm font-medium rounded-lg transition-colors ${
              currentView === 'passwords' 
                ? 'text-gray-900 bg-gray-100' 
                : 'text-gray-600 hover:text-gray-900 hover:bg-gray-50'
            }`}
          >
            <Shield className="h-4 w-4 mr-3" />
            All Passwords
          </button>

          <button 
            onClick={() => onNavigate('health')}
            className={`w-full flex items-center px-3 py-2 text-sm font-medium rounded-lg transition-colors ${
              currentView === 'health' 
                ? 'text-gray-900 bg-gray-100' 
                : 'text-gray-600 hover:text-gray-900 hover:bg-gray-50'
            }`}
          >
            <BarChart3 className="h-4 w-4 mr-3" />
            Password Health
          </button>
          
          {touchIdAvailable && (
            <button className="w-full flex items-center px-3 py-2 text-sm font-medium text-gray-600 hover:text-gray-900 hover:bg-gray-50 rounded-lg transition-colors">
              <Fingerprint className="h-4 w-4 mr-3" />
              Touch ID Settings
            </button>
          )}
          
          <button 
            onClick={() => onNavigate('settings')}
            className={`w-full flex items-center px-3 py-2 text-sm font-medium rounded-lg transition-colors ${
              currentView === 'settings' 
                ? 'text-gray-900 bg-gray-100' 
                : 'text-gray-600 hover:text-gray-900 hover:bg-gray-50'
            }`}
          >
            <Settings className="h-4 w-4 mr-3" />
            Settings
          </button>
          
          <button 
            onClick={() => onNavigate('security')}
            className={`w-full flex items-center px-3 py-2 text-sm font-medium rounded-lg transition-colors ${
              currentView === 'security' 
                ? 'text-gray-900 bg-gray-100' 
                : 'text-gray-600 hover:text-gray-900 hover:bg-gray-50'
            }`}
          >
            <Lock className="h-4 w-4 mr-3" />
            Security
          </button>
        </nav>
      </div>

      {/* Footer */}
      <div className="absolute bottom-0 left-0 right-0 p-6">
        <div className="text-xs text-gray-500 text-center">
          2Password v1.0.0
        </div>
      </div>
    </div>
  );
}