# Rax CLI - Setup Guide

This guide covers setting up the complete Rax CLI infrastructure including the apt repository.

## GitHub Repository Setup

### 1. Enable GitHub Actions
- Go to repository Settings > Actions
- Enable "Allow all actions and reusable workflows"

### 2. Enable GitHub Pages
- Go to repository Settings > Pages
- **Source**: Deploy from a branch
- **Branch**: Select `gh-pages` (will be created by workflow)
- **Folder**: `/` (root)
- Click Save

### 3. Configure Environments
- Go to repository Settings > Environments
- Create new environment: `github-pages`
- Optionally add deployment protection rules

## First Release

### Create a Release Tag
```bash
# Tag the release
git tag -a v0.1.0 -m "Rax CLI v0.1.0"

# Push the tag
git push origin v0.1.0
```

### Automated Workflow
Once the tag is pushed:
1. **CI Workflow** runs tests and builds
2. **Release Workflow** creates GitHub release with artifacts
3. **Deploy Workflow** deploys apt repository to GitHub Pages

## Manual apt Repository Setup

If you prefer manual setup:

### 1. Build the Package
```bash
./build-deb.sh
```

### 2. Update Repository
```bash
./update-repo.sh
```

### 3. Deploy to gh-pages Branch
```bash
# Create gh-pages branch if it doesn't exist
git checkout --orphan gh-pages
git reset --hard

# Copy repo contents
cp -r repo/* .
rm -rf repo

# Commit and push
git add .
git commit -m "Deploy apt repository"
git push -u origin gh-pages
```

## User Installation

After setup, users can install with:

```bash
# Add repository key
curl -fsSL https://raxcore-dev.github.io/rax-offline-cli/repo.key | \
    sudo gpg --dearmor -o /usr/share/keyrings/rax-archive-keyring.gpg

# Add repository
echo "deb [signed-by=/usr/share/keyrings/rax-archive-keyring.gpg] https://raxcore-dev.github.io/rax-offline-cli/ /" | \
    sudo tee /etc/apt/sources.list.d/rax.list

# Install
sudo apt update
sudo apt install rax
```

## Repository Signing (Optional but Recommended)

### Generate GPG Key
```bash
gpg --full-generate-key
# Key type: RSA and RSA
# Key size: 4096
# Expiration: 2y
# Name: Rax CLI Repository
# Email: your-email@example.com
```

### Export Public Key
```bash
gpg --armor --export your-email@example.com > repo/repo.key
```

### Sign Release Files
```bash
cd repo
gpg --absolute-key-id <KEY_ID> --sign -o Release.gpg Release
gpg --absolute-key-id <KEY_ID> --clearsign -o InRelease Release
```

### Store Private Key Securely
```bash
# Export private key (store securely, never commit to git)
gpg --armor --export-secret-keys your-email@example.com > rax-repo-key.private

# Store in secure location
# DO NOT commit this file to git!
```

## CI/CD Variables

The workflows use these default variables:
- `GITHUB_TOKEN`: Automatically provided
- `secrets.GITHUB_TOKEN`: For release creation

No additional secrets need to be configured for basic setup.

## Troubleshooting

### Workflow Not Running
- Check Actions tab for any errors
- Ensure Actions are enabled in Settings
- Verify the tag format is `v*` (e.g., v0.1.0)

### Pages Not Deploying
- Check the gh-pages branch exists
- Verify Pages source is set correctly
- Check the Deploy workflow logs

### apt Repository Not Working
- Verify repo.key is accessible: `curl -I https://raxcore-dev.github.io/rax-offline-cli/repo.key`
- Check Packages file exists: `curl -I https://raxcore-dev.github.io/rax-offline-cli/dists/stable/main/binary-amd64/Packages.gz`
- Ensure the repository URL ends with `/`

## Maintenance

### New Release
```bash
# Update version in Cargo.toml
# Commit changes
git tag -a v0.2.0 -m "Rax CLI v0.2.0"
git push origin v0.2.0
```

### Update Repository Key
```bash
# When key expires, generate new one
# Update repo/repo.key
# Commit and push
git add repo/repo.key
git commit -m "Update repository signing key"
git push
```

## Security Best Practices

1. **Never commit GPG private keys** to the repository
2. **Rotate keys** before expiration
3. **Use protected branches** for main and gh-pages
4. **Enable branch protection** rules
5. **Review workflow permissions** regularly

## Support

For issues or questions:
- [GitHub Issues](https://github.com/Raxcore-dev/rax-offline-cli/issues)
- [GitHub Discussions](https://github.com/Raxcore-dev/rax-offline-cli/discussions)
