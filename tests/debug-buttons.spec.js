import { test, expect } from '@playwright/test';

test.describe('VaultSetup Button Debug', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to the application
    await page.goto('http://localhost:3000');
    
    // Wait for the page to load
    await page.waitForLoadState('networkidle');
  });

  test('Debug button visibility and clickability', async ({ page }) => {
    console.log('🔍 Starting button debug test...');
    
    // Check if we're on the VaultSetup page by looking for the debug text
    const debugText = await page.locator('text=🐛 Debug Mode').textContent();
    console.log('Debug mode text:', debugText);
    
    // Find all buttons
    const buttons = await page.locator('button').all();
    console.log(`Found ${buttons.length} buttons on the page`);
    
    // Check each button's properties
    for (let i = 0; i < buttons.length; i++) {
      const button = buttons[i];
      const text = await button.textContent();
      const isVisible = await button.isVisible();
      const isEnabled = await button.isEnabled();
      const boundingBox = await button.boundingBox();
      
      console.log(`Button ${i}:`);
      console.log(`  Text: ${text?.trim()}`);
      console.log(`  Visible: ${isVisible}`);
      console.log(`  Enabled: ${isEnabled}`);
      console.log(`  Position: ${boundingBox ? `x:${boundingBox.x}, y:${boundingBox.y}, width:${boundingBox.width}, height:${boundingBox.height}` : 'null'}`);
    }
    
    // Try to click each button and see what happens
    const demoVaultButton = page.locator('text=Open Demo Vault');
    const createNewButton = page.locator('text=Create New Vault');
    const openExistingButton = page.locator('text=Open Existing Vault');
    
    console.log('🎯 Testing Demo Vault button...');
    if (await demoVaultButton.isVisible()) {
      console.log('Demo Vault button is visible and clickable');
      // We won't actually click this one as it works
    }
    
    console.log('🎯 Testing Create New Vault button...');
    if (await createNewButton.isVisible()) {
      console.log('Create New Vault button is visible');
      // Set up console message listener before clicking
      page.on('console', msg => {
        if (msg.text().includes('🔴')) {
          console.log('✅ Create New Vault console log:', msg.text());
        }
      });
      
      // Set up dialog handler for alert
      page.on('dialog', async dialog => {
        console.log('✅ Create New Vault alert:', dialog.message());
        await dialog.accept();
      });
      
      await createNewButton.click();
      console.log('Clicked Create New Vault button');
      
      // Wait a moment to see if anything happens
      await page.waitForTimeout(1000);
    }
    
    console.log('🎯 Testing Open Existing Vault button...');
    if (await openExistingButton.isVisible()) {
      console.log('Open Existing Vault button is visible');
      // Set up console message listener before clicking
      page.on('console', msg => {
        if (msg.text().includes('🔵')) {
          console.log('✅ Open Existing Vault console log:', msg.text());
        }
      });
      
      // Set up dialog handler for alert
      page.on('dialog', async dialog => {
        console.log('✅ Open Existing Vault alert:', dialog.message());
        await dialog.accept();
      });
      
      await openExistingButton.click();
      console.log('Clicked Open Existing Vault button');
      
      // Wait a moment to see if anything happens
      await page.waitForTimeout(1000);
    }
  });

  test('Check for CSS issues that might block clicks', async ({ page }) => {
    console.log('🔍 Checking for CSS issues...');
    
    // Check z-index and pointer-events for buttons
    const createNewButton = page.locator('text=Create New Vault').first();
    const openExistingButton = page.locator('text=Open Existing Vault').first();
    
    if (await createNewButton.isVisible()) {
      const createNewStyles = await createNewButton.evaluate(el => {
        const computed = window.getComputedStyle(el);
        return {
          zIndex: computed.zIndex,
          pointerEvents: computed.pointerEvents,
          position: computed.position,
          opacity: computed.opacity,
          visibility: computed.visibility,
          display: computed.display
        };
      });
      console.log('Create New Vault button styles:', createNewStyles);
    }
    
    if (await openExistingButton.isVisible()) {
      const openExistingStyles = await openExistingButton.evaluate(el => {
        const computed = window.getComputedStyle(el);
        return {
          zIndex: computed.zIndex,
          pointerEvents: computed.pointerEvents,
          position: computed.position,
          opacity: computed.opacity,
          visibility: computed.visibility,
          display: computed.display
        };
      });
      console.log('Open Existing Vault button styles:', openExistingStyles);
    }
  });
});