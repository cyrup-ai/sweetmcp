# SweetMCP Installation

## Quick Install

### macOS & Linux
```bash
curl -fsSL https://get.cyrup.ai/sweetmcp.sh | bash
```

### Windows (PowerShell as Administrator)
```powershell
iex (iwr -UseBasicParsing https://get.cyrup.ai/sweetmcp.ps1).Content
```

## What It Does

The installer performs a fully automated setup:

1. **Auto-installs Dependencies** - Git, curl, compilers, build tools, OpenSSL
2. **Rust Installation** - Auto-installs Rust if missing
3. **Repository Clone** - Downloads latest code
4. **Build** - Compiles the SweetMCP daemon
5. **Full Installation** - The daemon installer then:
   - Generates wildcard SSL certificates
   - Imports certificates to OS trust store
   - Adds host entries for all domains
   - Installs and starts services automatically

**No prerequisites!** We auto-install everything needed:
- **Linux**: Detects your distro and uses the right package manager
- **macOS**: Installs Xcode tools and Homebrew if needed
- **Windows**: Installs Git and Visual Studio Build Tools automatically

**Result**: Everything is running - just go have fun! 🍯

## What Gets Installed

| Component | Location |
|-----------|----------|
| Daemon Binary | `/usr/local/bin/cyrupd` |
| Pingora Gateway | Managed by daemon |
| Configuration | `~/.config/cyrupd/` |
| SSL Certificate | `~/.config/sweetmcp/wildcard.cyrup.pem` |
| Host Entries | `/etc/hosts` |
| Services | Running automatically |

## Available Endpoints

Once installed, these are immediately available:

- `https://sweetmcp.cyrup.dev:8443`
- `https://sweetmcp.cyrup.ai:8443` 
- `https://sweetmcp.cyrup.cloud:8443`
- `https://sweetmcp.cyrup.pro:8443`

## Platform Support

| Platform | Status | Service Manager |
|----------|--------|-----------------|
| macOS (Intel) | ✅ Full | launchd |
| macOS (Apple Silicon) | ✅ Full | launchd |
| Linux (x86_64) | ✅ Full | systemd |
| Linux (aarch64) | ✅ Full | systemd |
| Windows | ✅ Full | Windows Services |

## Service Management

Services start automatically during installation. If you need to manage them:

### Linux (systemd)
```bash
sudo systemctl status cyrupd    # Check status
sudo systemctl restart cyrupd   # Restart
journalctl -u cyrupd -f        # View logs
```

### macOS (launchd)
```bash
sudo launchctl list | grep cyrupd              # Check status
sudo launchctl kickstart -k system/com.cyrup.cyrupd  # Restart  
tail -f /var/log/cyrupd.log                    # View logs
```

### Windows
```powershell
sc query cyrupd          # Check status
sc stop cyrupd && sc start cyrupd  # Restart
Get-WinEvent -LogName Application | Where-Object {$_.ProviderName -eq 'cyrupd'}  # View logs
```

## Troubleshooting

### Installation Issues
- **"Failed to clone repository"**: Check your internet connection
- **"Build failed"**: Check you have enough disk space (2GB+) and memory (4GB+ recommended)
- **"Permission denied"**: The installer will request admin/sudo access when needed
- **Package manager issues**: The installer supports most distros, but very old or unusual systems may need manual dependency installation

### Service Issues
Services should start automatically. If not:
- Check logs for errors
- Verify port 8443 isn't already in use: `sudo lsof -i :8443` or `netstat -an | grep 8443`
- Ensure certificate was created in `~/.config/sweetmcp/`

### Certificate Issues
The installer generates and trusts certificates automatically. If you see SSL warnings:
- The certificate may not be imported correctly
- Try reinstalling: `sudo cyrupd uninstall && curl -fsSL https://get.cyrup.ai/sweetmcp.sh | bash`

### Dependency Installation Issues
The installer auto-installs all dependencies, but if it fails:
- **Linux**: Update your package manager first (`sudo apt update`, `sudo dnf update`, etc.)
- **macOS**: Ensure you have internet access for Xcode tools download
- **Windows**: Run as Administrator, ensure Windows Update is current

## Uninstallation

```bash
sudo cyrupd uninstall
```

This removes:
- The daemon and services
- Binary from `/usr/local/bin`
- Service configurations

Optional cleanup:
```bash
rm -rf ~/.config/cyrupd ~/.config/sweetmcp
sudo sed -i '/# SweetMCP/,+4d' /etc/hosts  # Remove host entries
```