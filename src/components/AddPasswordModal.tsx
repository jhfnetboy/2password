import { useState } from "react";
import { X, Globe, User, Key, FileText, RefreshCw, Tag, Plus } from "lucide-react";

interface AddPasswordModalProps {
  onAdd: (title: string, username: string, password: string, url?: string, notes?: string, tags?: string[]) => void;
  onClose: () => void;
}

export default function AddPasswordModal({ onAdd, onClose }: AddPasswordModalProps) {
  const [formData, setFormData] = useState({
    title: "",
    username: "",
    password: "",
    url: "",
    notes: "",
  });
  const [tags, setTags] = useState<string[]>([]);
  const [currentTag, setCurrentTag] = useState("");
  const [isGenerating, setIsGenerating] = useState(false);

  const addTag = () => {
    const tag = currentTag.trim();
    if (tag && !tags.includes(tag)) {
      setTags(prev => [...prev, tag]);
      setCurrentTag("");
    }
  };

  const removeTag = (tagToRemove: string) => {
    setTags(prev => prev.filter(tag => tag !== tagToRemove));
  };

  const handleTagKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === "Enter") {
      e.preventDefault();
      addTag();
    }
  };

  const generatePassword = () => {
    setIsGenerating(true);
    
    // Simple password generation (in a real app, you'd want more sophisticated options)
    const length = 16;
    const charset = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*";
    let password = "";
    
    for (let i = 0; i < length; i++) {
      password += charset.charAt(Math.floor(Math.random() * charset.length));
    }
    
    setFormData(prev => ({ ...prev, password }));
    setIsGenerating(false);
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    
    console.log("🚀 AddPasswordModal: Form submission started");
    console.log("📝 Form data:", {
      title: formData.title,
      username: formData.username,
      password: formData.password ? "[PROTECTED]" : "(empty)",
      url: formData.url,
      notes: formData.notes,
      tags: tags
    });
    
    if (!formData.title.trim() || !formData.username.trim() || !formData.password.trim()) {
      console.error("❌ Validation failed: Missing required fields");
      alert("Please fill in title, username, and password fields.");
      return;
    }

    console.log("✅ Validation passed, calling onAdd callback");
    try {
      onAdd(
        formData.title.trim(),
        formData.username.trim(),
        formData.password.trim(),
        formData.url.trim() || undefined,
        formData.notes.trim() || undefined,
        tags.length > 0 ? tags : undefined
      );
      console.log("✅ onAdd callback completed successfully");
    } catch (error) {
      console.error("❌ Error in onAdd callback:", error);
    }
  };

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4 z-50">
      <div className="bg-white rounded-lg shadow-xl max-w-md w-full max-h-screen overflow-y-auto">
        {/* Header */}
        <div className="flex items-center justify-between p-6 border-b border-gray-200">
          <h2 className="text-lg font-semibold text-gray-900">Add New Password</h2>
          <button
            onClick={onClose}
            className="p-1 hover:bg-gray-100 rounded-lg transition-colors"
          >
            <X className="h-5 w-5 text-gray-400" />
          </button>
        </div>

        {/* Form */}
        <form onSubmit={handleSubmit} className="p-6 space-y-4">
          {/* Website/Title */}
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-2">
              Website or Service *
            </label>
            <div className="relative">
              <Globe className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-gray-400" />
              <input
                type="text"
                required
                placeholder="e.g., GitHub, Gmail, Bank of America"
                className="input pl-10"
                value={formData.title}
                onChange={(e) => setFormData(prev => ({ ...prev, title: e.target.value }))}
              />
            </div>
          </div>

          {/* URL */}
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-2">
              URL (optional)
            </label>
            <input
              type="url"
              placeholder="https://example.com"
              className="input"
              value={formData.url}
              onChange={(e) => setFormData(prev => ({ ...prev, url: e.target.value }))}
            />
          </div>

          {/* Username */}
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-2">
              Username or Email *
            </label>
            <div className="relative">
              <User className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-gray-400" />
              <input
                type="text"
                required
                placeholder="username@example.com"
                className="input pl-10"
                value={formData.username}
                onChange={(e) => setFormData(prev => ({ ...prev, username: e.target.value }))}
              />
            </div>
          </div>

          {/* Password */}
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-2">
              Password *
            </label>
            <div className="relative">
              <Key className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-gray-400" />
              <input
                type="text"
                required
                placeholder="Enter or generate a password"
                className="input pl-10 pr-10"
                value={formData.password}
                onChange={(e) => setFormData(prev => ({ ...prev, password: e.target.value }))}
              />
              <button
                type="button"
                onClick={generatePassword}
                disabled={isGenerating}
                className="absolute right-3 top-1/2 transform -translate-y-1/2 p-1 hover:bg-gray-100 rounded transition-colors"
                title="Generate password"
              >
                <RefreshCw className={`h-4 w-4 text-gray-400 ${isGenerating ? 'animate-spin' : ''}`} />
              </button>
            </div>
            <p className="text-xs text-gray-500 mt-1">
              Click the refresh icon to generate a secure password
            </p>
          </div>

          {/* Tags */}
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-2">
              Tags (optional)
            </label>
            <div className="space-y-2">
              <div className="relative">
                <Tag className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-gray-400" />
                <input
                  type="text"
                  placeholder="Add a tag and press Enter"
                  className="input pl-10 pr-10"
                  value={currentTag}
                  onChange={(e) => setCurrentTag(e.target.value)}
                  onKeyPress={handleTagKeyPress}
                />
                <button
                  type="button"
                  onClick={addTag}
                  disabled={!currentTag.trim()}
                  className="absolute right-3 top-1/2 transform -translate-y-1/2 p-1 hover:bg-gray-100 rounded transition-colors disabled:opacity-50"
                  title="Add tag"
                >
                  <Plus className="h-4 w-4 text-gray-400" />
                </button>
              </div>
              {tags.length > 0 && (
                <div className="flex flex-wrap gap-2">
                  {tags.map((tag, index) => (
                    <span
                      key={index}
                      className="inline-flex items-center px-2 py-1 rounded-full text-xs bg-blue-100 text-blue-800"
                    >
                      {tag}
                      <button
                        type="button"
                        onClick={() => removeTag(tag)}
                        className="ml-1 hover:text-blue-600"
                        title="Remove tag"
                      >
                        <X className="h-3 w-3" />
                      </button>
                    </span>
                  ))}
                </div>
              )}
            </div>
          </div>

          {/* Notes */}
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-2">
              Notes (optional)
            </label>
            <div className="relative">
              <FileText className="absolute left-3 top-3 h-4 w-4 text-gray-400" />
              <textarea
                placeholder="Additional notes or comments"
                className="input pl-10 min-h-[80px] resize-none"
                value={formData.notes}
                onChange={(e) => setFormData(prev => ({ ...prev, notes: e.target.value }))}
              />
            </div>
          </div>

          {/* Actions */}
          <div className="flex space-x-3 pt-4">
            <button
              type="button"
              onClick={onClose}
              className="flex-1 btn btn-secondary"
            >
              Cancel
            </button>
            <button
              type="submit"
              className="flex-1 btn btn-primary"
            >
              Add Password
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}