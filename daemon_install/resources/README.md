# Resources

This directory contains platform-specific resources for the installer.

## macOS Signed Helper

The `sudo-prompt-applet.zip` file should contain a signed macOS application bundle that can request administrator privileges. This is used to provide a native authorization dialog on macOS.

In production, this would be:
1. A minimal Objective-C/Swift application that uses `AuthorizationServices`
2. Code-signed with a valid Developer ID
3. Includes an embedded `Info.plist` and authorization rights
4. Compressed as a zip file for embedding

For development, the library will fall back to using `osascript` directly.