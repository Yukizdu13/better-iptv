# Better IPTV - AUR Package

This directory contains the PKGBUILD for publishing Better IPTV on the Arch User Repository (AUR).

## Package Information

- **Package name**: `better-iptv-bin`
- **Type**: Binary package (downloads pre-built AppImage from GitHub releases)
- **Dependencies**: `mpv`, `webkit2gtk`, `gtk3`

## Publishing to AUR

### First-time Setup

1. **Create AUR account** at https://aur.archlinux.org/register

2. **Add SSH key** to your AUR account:
   ```bash
   ssh-keygen -t ed25519 -C "your_email@example.com"
   cat ~/.ssh/id_ed25519.pub
   # Add this to https://aur.archlinux.org/account/
   ```

3. **Clone AUR repository**:
   ```bash
   git clone ssh://aur@aur.archlinux.org/better-iptv-bin.git aur-repo
   ```

4. **Copy PKGBUILD files**:
   ```bash
   cp aur/PKGBUILD aur-repo/
   cp aur/.SRCINFO aur-repo/
   ```

5. **Update maintainer info** in `PKGBUILD`:
   ```bash
   # Change this line:
   # Maintainer: Your Name <your.email@example.com>
   ```

6. **Update checksum**:
   ```bash
   cd aur-repo
   # Download the AppImage to calculate checksum
   wget https://github.com/mewset/better-iptv/releases/download/v2.0.0/better-iptv_2.0.0_amd64.AppImage
   sha256sum better-iptv_2.0.0_amd64.AppImage

   # Update sha256sums in PKGBUILD with the actual checksum
   # Replace: sha256sums=('SKIP')
   # With:    sha256sums=('actual_checksum_here')
   ```

7. **Generate .SRCINFO**:
   ```bash
   makepkg --printsrcinfo > .SRCINFO
   ```

8. **Test build locally**:
   ```bash
   makepkg -si
   # This builds and installs the package
   ```

9. **Commit and push to AUR**:
   ```bash
   git add PKGBUILD .SRCINFO
   git commit -m "Initial release: v2.0.0"
   git push
   ```

### Updating the Package

When you release a new version:

1. **Update version in PKGBUILD**:
   ```bash
   pkgver=2.1.0  # New version
   pkgrel=1      # Reset to 1 for new version
   ```

2. **Update source URL** to point to new release

3. **Update checksum**:
   ```bash
   wget https://github.com/mewset/better-iptv/releases/download/v2.1.0/better-iptv_2.1.0_amd64.AppImage
   sha256sum better-iptv_2.1.0_amd64.AppImage
   # Update sha256sums in PKGBUILD
   ```

4. **Regenerate .SRCINFO**:
   ```bash
   makepkg --printsrcinfo > .SRCINFO
   ```

5. **Test build**:
   ```bash
   makepkg -si
   ```

6. **Commit and push**:
   ```bash
   git add PKGBUILD .SRCINFO
   git commit -m "Update to v2.1.0"
   git push
   ```

## Automated Updates

You can automate AUR updates in your release workflow by adding this to `.github/workflows/release.yml`:

```yaml
- name: Update AUR package
  if: matrix.platform == 'ubuntu-22.04'
  run: |
    # This would require AUR SSH key as GitHub secret
    # and a script to update PKGBUILD automatically
    echo "Manual AUR update required"
```

## Installation for Users

Once published on AUR, users can install with:

```bash
# Using yay
yay -S better-iptv-bin

# Using paru
paru -S better-iptv-bin

# Manual installation
git clone https://aur.archlinux.org/better-iptv-bin.git
cd better-iptv-bin
makepkg -si
```

## Package Variants

You can create additional package variants:

- **`better-iptv`**: Builds from source (requires Rust, Node.js)
- **`better-iptv-git`**: Builds latest git commit

See AUR packaging guidelines: https://wiki.archlinux.org/title/AUR_submission_guidelines

## Testing

Before publishing, test the package:

```bash
cd aur
makepkg -si  # Build and install

# Test the application
better-iptv

# Check files
pacman -Ql better-iptv-bin

# Uninstall
sudo pacman -R better-iptv-bin
```

## Troubleshooting

### AppImage extraction fails
If the AppImage extraction fails, you may need to install `fuse2`:
```bash
sudo pacman -S fuse2
```

### Wrong checksum
Always regenerate checksums after updating the version:
```bash
updpkgsums  # Updates checksums in PKGBUILD
makepkg --printsrcinfo > .SRCINFO
```

### Build fails
Check that all dependencies are correctly listed:
- Runtime deps: `depends=()`
- Build deps: `makedepends=()`

## Resources

- [AUR Submission Guidelines](https://wiki.archlinux.org/title/AUR_submission_guidelines)
- [PKGBUILD Manual](https://wiki.archlinux.org/title/PKGBUILD)
- [Arch Package Guidelines](https://wiki.archlinux.org/title/Arch_package_guidelines)
