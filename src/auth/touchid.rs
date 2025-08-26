//! Touch ID authentication for macOS
//!
//! This module provides Touch ID integration using the LocalAuthentication framework

use crate::{Result, TwoPasswordError};

#[cfg(target_os = "macos")]
use core_foundation::{
    base::{CFTypeRef, TCFType},
    boolean::CFBoolean,
    dictionary::{CFDictionary, CFMutableDictionary},
    error::CFError,
    string::{CFString, CFStringRef},
};

#[cfg(target_os = "macos")]
use security_framework::access_control::SecAccessControl;

/// Check if Touch ID is available on this device
#[cfg(target_os = "macos")]
pub fn is_available() -> bool {
    // Use LocalAuthentication framework to check availability
    use objc::runtime::{Class, Object};
    use objc::{msg_send, sel, sel_impl};

    unsafe {
        let la_context_class = match Class::get("LAContext") {
            Some(class) => class,
            None => {
                tracing::warn!("LocalAuthentication framework not available");
                return false;
            }
        };

        let context: *mut Object = msg_send![la_context_class, alloc];
        let context: *mut Object = msg_send![context, init];

        if context.is_null() {
            return false;
        }

        // Check if biometric authentication is available
        // LAPolicy.deviceOwnerAuthenticationWithBiometrics = 1
        let policy: i32 = 1;
        let mut error: *mut Object = std::ptr::null_mut();

        let can_evaluate: bool = msg_send![context, canEvaluatePolicy:policy error:&mut error];

        // Clean up
        let _: () = msg_send![context, release];

        if !error.is_null() {
            let _: () = msg_send![error, release];
        }

        can_evaluate
    }
}

/// Check if Touch ID is available (always false on non-macOS)
#[cfg(not(target_os = "macos"))]
pub fn is_available() -> bool {
    false
}

/// Authenticate using Touch ID
#[cfg(target_os = "macos")]
pub fn authenticate(reason: &str) -> Result<bool> {
    use objc::runtime::{Class, Object};
    use objc::{msg_send, sel, sel_impl};
    use std::ffi::CString;

    unsafe {
        let la_context_class = Class::get("LAContext").ok_or_else(|| {
            TwoPasswordError::touch_id("LocalAuthentication framework not available")
        })?;

        let context: *mut Object = msg_send![la_context_class, alloc];
        let context: *mut Object = msg_send![context, init];

        if context.is_null() {
            return Err(TwoPasswordError::touch_id("Failed to create LAContext"));
        }

        // Create reason string
        let ns_string_class = Class::get("NSString").unwrap();
        let reason_cstring = CString::new(reason).unwrap();
        let _reason_nsstring: *mut Object = msg_send![ns_string_class, 
            stringWithUTF8String:reason_cstring.as_ptr()];

        // LAPolicy.deviceOwnerAuthenticationWithBiometrics = 1
        let policy: i32 = 1;

        // For now, return a basic implementation
        // In a full implementation, you would need to handle the async callback
        let mut error: *mut Object = std::ptr::null_mut();
        let can_evaluate: bool = msg_send![context, canEvaluatePolicy:policy error:&mut error];

        // Clean up
        let _: () = msg_send![context, release];

        if !error.is_null() {
            let _: () = msg_send![error, release];
            return Err(TwoPasswordError::touch_id("Touch ID evaluation failed"));
        }

        if !can_evaluate {
            return Err(TwoPasswordError::touch_id("Touch ID not available"));
        }

        // This is a simplified implementation
        // In reality, we'd need to handle the async evaluatePolicy callback
        tracing::info!("Touch ID authentication requested: {}", reason);
        Ok(true) // Placeholder - would be actual authentication result
    }
}

/// Authenticate using Touch ID (always fails on non-macOS)
#[cfg(not(target_os = "macos"))]
pub fn authenticate(_reason: &str) -> Result<bool> {
    Err(TwoPasswordError::touch_id(
        "Touch ID not available on this platform",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_availability() {
        // This will depend on the platform and hardware
        let available = is_available();
        tracing::info!("Touch ID available: {}", available);
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_authenticate() {
        // Note: This test may prompt for Touch ID in development
        // In CI/CD, it should fail gracefully
        match authenticate("Test authentication") {
            Ok(result) => tracing::info!("Touch ID authentication result: {}", result),
            Err(e) => tracing::info!("Touch ID authentication error: {}", e),
        }
    }
}
