# Page snapshot

```yaml
- generic [ref=e4]:
  - generic [ref=e5]:
    - img [ref=e6]
    - heading "Welcome to 2Password" [level=1] [ref=e8]
    - paragraph [ref=e9]: Secure password management with Touch ID integration
    - paragraph [ref=e10]: "🐛 Debug Mode: isLoading = false"
  - generic [ref=e11]:
    - 'button "Open Demo Vault ⚠️ For testing - Creates if not exists Password: demo123" [ref=e12] [cursor=pointer]':
      - generic [ref=e13] [cursor=pointer]:
        - img [ref=e15] [cursor=pointer]
        - generic [ref=e17] [cursor=pointer]:
          - generic [ref=e18] [cursor=pointer]: Open Demo Vault
          - generic [ref=e19] [cursor=pointer]: ⚠️ For testing - Creates if not exists
          - generic [ref=e20] [cursor=pointer]: "Password: demo123"
    - button "Create New Vault Set up a new password vault with master password" [ref=e21] [cursor=pointer]:
      - generic [ref=e22] [cursor=pointer]:
        - img [ref=e24] [cursor=pointer]
        - generic [ref=e26] [cursor=pointer]:
          - generic [ref=e27] [cursor=pointer]: Create New Vault
          - generic [ref=e28] [cursor=pointer]: Set up a new password vault with master password
    - button "Open Existing Vault Load your existing password vault" [active] [ref=e29] [cursor=pointer]:
      - generic [ref=e30] [cursor=pointer]:
        - img [ref=e32] [cursor=pointer]
        - generic [ref=e34] [cursor=pointer]:
          - generic [ref=e35] [cursor=pointer]: Open Existing Vault
          - generic [ref=e36] [cursor=pointer]: Load your existing password vault
  - generic [ref=e37]: "Failed to load vault: TypeError: Cannot read properties of undefined (reading 'invoke')"
```