# Setting up the Rax CLI apt Repository

This document explains how to set up and maintain the official apt repository for Rax CLI.

## Repository Structure

```
repo/
├── dists/
│   └── stable/
│       ├── main/
│       │   └── binary-amd64/
│       │       ├── Packages
│       │       └── Packages.gz
│       └── Release
├── pool/
│   └── main/
│       └── r/
│           └── rax/
│               └── rax_*.deb
└── repo.key
```

## Initial Setup

### 1. Generate GPG Key for Signing

```bash
gpg --full-generate-key
# Key type: RSA and RSA
# Key size: 4096
# Expiration: 2y
# Name: Rax CLI Repository
# Email: rax@localhost
# Comment: apt repository signing key
```

### 2. Export the Key

```bash
gpg --armor --export rax@localhost > repo/repo.key
```

### 3. Sign the Repository

```bash
cd repo
gpg --absolute-key-id <KEY_ID> --sign -o Release.gpg Release
```

## Automated Deployment

The repository is automatically deployed to GitHub Pages when:
1. A new release is published on GitHub
2. The `deploy-repo.yml` workflow runs

## Manual Update

```bash
./update-repo.sh
```

## User Installation

Users can install Rax CLI with:

```bash
# Add the repository key
curl -fsSL https://rax-cli.github.io/repo.key | \
    sudo gpg --dearmor -o /usr/share/keyrings/rax-archive-keyring.gpg

# Add the repository
echo "deb [signed-by=/usr/share/keyrings/rax-archive-keyring.gpg] https://rax-cli.github.io/ /" | \
    sudo tee /etc/apt/sources.list.d/rax.list

# Install
sudo apt update
sudo apt install rax
```

## GitHub Pages Setup

1. Go to repository Settings > Pages
2. Source: Deploy from a branch
3. Branch: gh-pages
4. Folder: / (root)
5. Save

## CI/CD Workflow

The `deploy-repo.yml` workflow:
1. Downloads the .deb package from the release
2. Creates the apt repository structure
3. Generates Packages and Release files
4. Deploys to GitHub Pages

## Security Notes

- Keep the GPG private key secure
- Never commit the private key to the repository
- Only the public key (`repo.key`) should be in the repo
- Rotate keys before expiration
