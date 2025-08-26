import { useState } from "react";
import { Globe, User, Copy, Trash2, Eye, EyeOff } from "lucide-react";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import { PasswordEntry } from "../types";

interface PasswordListProps {
  entries: PasswordEntry[];
  onDeleteEntry: (entryId: string) => void;
  searchQuery: string;
}

export default function PasswordList({ entries, onDeleteEntry, searchQuery }: PasswordListProps) {
  const [visiblePasswords, setVisiblePasswords] = useState<Set<string>>(new Set());
  const [copiedEntry, setCopiedEntry] = useState<string | null>(null);

  const togglePasswordVisibility = (entryId: string) => {
    const newVisible = new Set(visiblePasswords);
    if (newVisible.has(entryId)) {
      newVisible.delete(entryId);
    } else {
      newVisible.add(entryId);
    }
    setVisiblePasswords(newVisible);
  };

  const copyToClipboard = async (text: string, type: string, entryId: string) => {
    try {
      await writeText(text);
      setCopiedEntry(`${entryId}-${type}`);
      setTimeout(() => setCopiedEntry(null), 2000);
    } catch (error) {
      console.error("Failed to copy to clipboard:", error);
    }
  };

  const handleDelete = (entryId: string, title: string) => {
    if (confirm(`Are you sure you want to delete "${title}"?`)) {
      onDeleteEntry(entryId);
    }
  };

  if (entries.length === 0) {
    return (
      <div className="text-center py-12">
        <Globe className="h-12 w-12 text-gray-400 mx-auto mb-4" />
        <h3 className="text-lg font-medium text-gray-900 mb-2">
          {searchQuery ? "No passwords found" : "No passwords yet"}
        </h3>
        <p className="text-gray-500 mb-6">
          {searchQuery 
            ? `No passwords match "${searchQuery}"` 
            : "Add your first password to get started"}
        </p>
      </div>
    );
  }

  return (
    <div className="space-y-4">
      {entries.map((entry) => (
        <div key={entry.id} className="card p-6">
          <div className="flex items-start justify-between">
            <div className="flex-1 min-w-0">
              {/* Title and URL */}
              <div className="flex items-center mb-2">
                <Globe className="h-5 w-5 text-gray-400 mr-3 flex-shrink-0" />
                <div className="min-w-0">
                  <h3 className="text-lg font-medium text-gray-900 truncate">
                    {entry.title}
                  </h3>
                  {entry.url && (
                    <p className="text-sm text-gray-500 truncate">{entry.url}</p>
                  )}
                </div>
              </div>

              {/* Username */}
              <div className="flex items-center justify-between mb-3">
                <div className="flex items-center min-w-0 flex-1">
                  <User className="h-4 w-4 text-gray-400 mr-2 flex-shrink-0" />
                  <span className="text-sm text-gray-700 truncate">{entry.username}</span>
                </div>
                <button
                  onClick={() => copyToClipboard(entry.username, "username", entry.id)}
                  className="ml-2 p-1 text-gray-400 hover:text-gray-600 transition-colors"
                  title="Copy username"
                >
                  <Copy className="h-4 w-4" />
                </button>
              </div>

              {/* Password */}
              <div className="flex items-center justify-between">
                <div className="flex items-center min-w-0 flex-1">
                  <div className="w-4 mr-2" /> {/* Spacer for alignment */}
                  <span className="text-sm text-gray-700 font-mono">
                    {visiblePasswords.has(entry.id) 
                      ? entry.password 
                      : "•".repeat(Math.min(entry.password.length, 12))}
                  </span>
                </div>
                <div className="flex items-center space-x-1 ml-2">
                  <button
                    onClick={() => togglePasswordVisibility(entry.id)}
                    className="p-1 text-gray-400 hover:text-gray-600 transition-colors"
                    title={visiblePasswords.has(entry.id) ? "Hide password" : "Show password"}
                  >
                    {visiblePasswords.has(entry.id) ? (
                      <EyeOff className="h-4 w-4" />
                    ) : (
                      <Eye className="h-4 w-4" />
                    )}
                  </button>
                  <button
                    onClick={() => copyToClipboard(entry.password, "password", entry.id)}
                    className="p-1 text-gray-400 hover:text-gray-600 transition-colors"
                    title="Copy password"
                  >
                    <Copy className="h-4 w-4" />
                  </button>
                </div>
              </div>

              {/* Notes */}
              {entry.notes && (
                <div className="mt-3 text-sm text-gray-600">
                  {entry.notes}
                </div>
              )}

              {/* Tags */}
              {entry.tags && entry.tags.length > 0 && (
                <div className="mt-3 flex flex-wrap gap-1">
                  {entry.tags.map((tag, index) => (
                    <span
                      key={index}
                      className="inline-flex items-center px-2 py-1 rounded-full text-xs bg-gray-100 text-gray-700"
                    >
                      {tag}
                    </span>
                  ))}
                </div>
              )}

              {/* Metadata */}
              <div className="mt-4 text-xs text-gray-400">
                Created {new Date(entry.created_at).toLocaleDateString()}
                {entry.updated_at !== entry.created_at && (
                  <span> • Updated {new Date(entry.updated_at).toLocaleDateString()}</span>
                )}
              </div>
            </div>

            {/* Actions */}
            <div className="ml-4 flex-shrink-0">
              <button
                onClick={() => handleDelete(entry.id, entry.title)}
                className="p-2 text-gray-400 hover:text-red-600 transition-colors"
                title="Delete password"
              >
                <Trash2 className="h-4 w-4" />
              </button>
            </div>
          </div>

          {/* Copy notification */}
          {(copiedEntry === `${entry.id}-username` || copiedEntry === `${entry.id}-password`) && (
            <div className="mt-2 text-xs text-green-600 font-medium">
              {copiedEntry.includes("username") ? "Username" : "Password"} copied to clipboard!
            </div>
          )}
        </div>
      ))}
    </div>
  );
}