# TwoPassword Browser Extension

A secure, privacy-focused browser extension for the TwoPassword password manager with Touch ID integration and advanced security features.

## Features

### 🔐 Secure Password Management
- **Auto-fill login credentials** on any website
- **Smart form detection** for login forms
- **One-click password generation** with customizable options
- **Touch ID/biometric authentication** on supported platforms

### 🛡️ Privacy & Security
- **Native messaging** - communicates only with local TwoPassword app
- **k-anonymity breach checking** via HaveIBeenPwned API
- **No cloud storage** - all data stays local
- **End-to-end encryption** with AES-256-GCM

### 🎯 Smart Features
- **Context-aware suggestions** based on current website
- **Duplicate password detection** and warnings
- **Password strength analysis** with actionable feedback
- **Security dashboard** with comprehensive health metrics

### 🌐 Cross-Browser Support
- **Manifest V3 compliant** for Chrome, Edge, and other Chromium browsers
- **Firefox compatibility** (with manifest adaptation)
- **Safari support** (through WebExtension polyfills)

## Installation

### Prerequisites
1. **TwoPassword native application** must be installed and running
2. **Native messaging host** must be configured
3. **Modern browser** supporting WebExtensions API

### Chrome/Edge Installation
1. Download the extension from the Chrome Web Store (coming soon)
2. Or load as unpacked extension in developer mode:
   - Open Chrome/Edge and navigate to `chrome://extensions/`
   - Enable "Developer mode"
   - Click "Load unpacked" and select the `browser-extension` folder

### Firefox Installation
1. Download from Firefox Add-ons (coming soon)
2. Or load temporarily:
   - Navigate to `about:debugging#/runtime/this-firefox`
   - Click "Load Temporary Add-on"
   - Select the `manifest.json` file

### Safari Installation
1. Convert extension using Xcode WebExtension tools
2. Install through macOS Safari Extension preferences

## Usage

### Initial Setup
1. **Install TwoPassword app** on your computer
2. **Install browser extension** from your browser's extension store
3. **Click extension icon** in browser toolbar
4. **Unlock vault** using Touch ID or master password

### Daily Usage

#### Auto-fill Passwords
- Navigate to any login page
- Extension automatically detects login forms
- Click the 🔐 icon next to password fields
- Select credentials from the dropdown menu
- Login form fills automatically

#### Save New Passwords
- Fill out a login form manually
- Submit the form
- Extension prompts to save credentials
- Choose "Save" to store in TwoPassword vault

#### Generate Strong Passwords
- Right-click on any password field
- Select "TwoPassword → Generate Password"
- Customize password options (length, characters)
- Generated password fills automatically

#### Security Dashboard
- Click extension icon in toolbar
- Select "Security Dashboard"
- View password strength analysis
- See breach alerts and recommendations
- Track security improvements over time

## Architecture

### Extension Components

```
browser-extension/
├── manifest.json              # Extension manifest (Manifest V3)
├── src/
│   ├── background/           # Service Worker background scripts
│   │   ├── service-worker.js # Main background script
│   │   ├── native-messaging.js # Native app communication
│   │   ├── password-detector.js # Form detection
│   │   └── context-menu.js   # Right-click context menus
│   ├── content/             # Content scripts (injected into pages)
│   │   ├── content-script.js # Main content script
│   │   └── content-script.css # Content script styles
│   ├── popup/               # Extension popup (toolbar icon)
│   │   ├── popup.html       # Popup interface
│   │   ├── popup.css        # Popup styles
│   │   └── popup.js         # Popup logic
│   ├── options/             # Extension options/settings page
│   │   ├── options.html     # Settings interface
│   │   ├── options.css      # Settings styles
│   │   └── options.js       # Settings logic
│   └── shared/              # Shared utilities
│       └── storage.js       # Extension storage management
├── icons/                   # Extension icons (16px, 32px, 48px, 128px)
└── _locales/               # Internationalization
    └── en/
        └── messages.json   # English strings
```

### Security Model

#### Native Messaging
- **Local communication only** - extension talks to local TwoPassword app
- **Encrypted message channel** using Chrome's native messaging API
- **No network requests** for sensitive password data
- **Process isolation** between browser and password vault

#### Content Script Isolation
- **Minimal DOM access** - only for form detection and auto-fill
- **No password exposure** - credentials never stored in browser context
- **Event-driven architecture** - responds to user actions only
- **CSP compliance** - follows strict Content Security Policy

#### Permission Model
```json
{
  "permissions": [
    "storage",          // Extension settings only
    "activeTab",        // Current tab access when needed
    "contextMenus",     // Right-click menu integration  
    "notifications",    // User alerts and confirmations
    "nativeMessaging",  // Communication with TwoPassword app
    "alarms"           // Background tasks and cleanup
  ]
}
```

## Development

### Prerequisites
- Node.js 16+ for build tools
- TwoPassword native app for testing
- Chrome/Firefox for testing

### Build Process
```bash
# Install dependencies (if any build tools added)
npm install

# Test extension loading
# Chrome: Load unpacked from browser-extension/
# Firefox: Load temporary from browser-extension/manifest.json

# Package for distribution
zip -r twopassword-extension.zip browser-extension/
```

### Testing
1. **Load extension** in development mode
2. **Test with TwoPassword app** running locally  
3. **Verify auto-fill** on various login forms
4. **Test security features** like password generation
5. **Check cross-browser compatibility**

### Native Messaging Setup
The extension requires a native messaging host to communicate with the TwoPassword application:

```json
{
  "name": "com.twopassword.native",
  "description": "TwoPassword Native Messaging Host",
  "path": "/path/to/twopassword-native-host",
  "type": "stdio",
  "allowed_origins": [
    "chrome-extension://[extension-id]/"
  ]
}
```

## Privacy & Security

### Data Handling
- **No telemetry or analytics** - extension doesn't track usage
- **Local storage only** - settings stored in browser's local storage  
- **No network access** - passwords never leave your device
- **Minimal permissions** - only necessary browser APIs used

### Security Features
- **Form detection heuristics** - smart login form identification
- **Auto-save prompts** - optional credential saving with user consent
- **Breach monitoring** - optional HaveIBeenPwned integration via k-anonymity
- **Context isolation** - extension components run in separate security contexts

### Compliance
- **GDPR compliant** - no personal data collection
- **SOC 2 Type II** principles followed
- **Zero-knowledge architecture** - extension cannot access decrypted passwords
- **Open source** - code available for security audit

## Browser Compatibility

| Browser | Manifest | Status | Notes |
|---------|----------|--------|-------|
| Chrome 88+ | V3 | ✅ Full Support | Native implementation |
| Edge 88+ | V3 | ✅ Full Support | Chromium-based |
| Firefox 109+ | V3 | ⚠️ Limited | Some V3 features pending |
| Safari 15+ | V3 | 🔄 Planned | WebExtension conversion needed |
| Opera 74+ | V3 | ✅ Full Support | Chromium-based |

## Contributing

1. **Fork the repository**
2. **Create feature branch** (`git checkout -b feature/amazing-feature`)
3. **Test thoroughly** with TwoPassword app
4. **Submit pull request** with detailed description

### Code Style
- **ESLint configuration** for JavaScript consistency
- **Manifest V3 compliance** for future compatibility
- **Accessibility standards** (WCAG 2.1 AA)
- **Performance optimization** for minimal resource usage

## Support

### Troubleshooting
- **Extension not loading**: Check if TwoPassword app is running
- **Auto-fill not working**: Verify native messaging host is installed
- **Touch ID unavailable**: Ensure biometrics are set up in TwoPassword app
- **Sync issues**: Extension doesn't sync - all data managed by native app

### Getting Help
- **GitHub Issues**: Report bugs and feature requests
- **Documentation**: Comprehensive guides at [docs.twopassword.com]
- **Community**: Join discussions on GitHub Discussions
- **Security Issues**: Report privately to security@twopassword.com

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- **zxcvbn** for password strength analysis
- **HaveIBeenPwned** for breach detection API
- **WebExtensions API** for cross-browser compatibility
- **Manifest V3** specification for modern extension architecture