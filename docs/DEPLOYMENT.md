# Hermes Watch deployment

Hermes is a long-lived Discord gateway process. GitHub Pages hosts the project page, but the bot itself must run on an always-on Linux host such as a small VPS, homelab server, or always-on workstation.

The supported deployment shape is a dedicated `hermes` system user, a root-owned environment file, a signed release archive, and a hardened `systemd` service.

## Install a signed release

On the target Linux host, install `cosign`, then choose a published version:

```bash
VERSION=v0.1.0
curl -fL "https://github.com/darkstardevx/hermes/releases/download/${VERSION}/hermes-${VERSION}-x86_64-unknown-linux-gnu.tar.gz" -o hermes.tar.gz
curl -fL "https://github.com/darkstardevx/hermes/releases/download/${VERSION}/hermes-${VERSION}-x86_64-unknown-linux-gnu.tar.gz.sha256" -o hermes.tar.gz.sha256
curl -fL "https://github.com/darkstardevx/hermes/releases/download/${VERSION}/hermes-${VERSION}-x86_64-unknown-linux-gnu.tar.gz.sigstore.json" -o hermes.tar.gz.sigstore.json

sha256sum --check hermes.tar.gz.sha256
cosign verify-blob hermes.tar.gz \
  --bundle hermes.tar.gz.sigstore.json \
  --certificate-identity-regexp 'https://github.com/darkstardevx/hermes/.github/workflows/release.yml@refs/tags/v.*' \
  --certificate-oidc-issuer 'https://token.actions.githubusercontent.com'
```

The release archive contains the Hermes binary, the service unit, and the environment template. The workflow uses GitHub Actions OIDC for keyless Sigstore signing; no private signing key is stored in the repository.

## Configure the service

```bash
sudo useradd --system --home /var/lib/hermes --shell /usr/sbin/nologin hermes
sudo install -d -o hermes -g hermes -m 0750 /var/lib/hermes
sudo install -d -m 0750 /etc/hermes
sudo install -m 0640 -o root -g hermes hermes.env.example /etc/hermes/hermes.env
sudoedit /etc/hermes/hermes.env
sudo install -m 0755 hermes /usr/local/bin/hermes
sudo install -m 0644 hermes.service /etc/systemd/system/hermes.service
sudo systemctl daemon-reload
sudo systemctl enable --now hermes.service
```

The environment file must contain `DISCORD_TOKEN`, `GITHUB_TOKEN`, and `DEV_LOG_CHANNEL_ID`. Keep it outside the repository and never paste its contents into an issue, commit, or Discord message.

## Operate Hermes

```bash
systemctl status hermes.service
journalctl -u hermes.service -f
systemctl restart hermes.service
systemctl stop hermes.service
```

The service starts at boot, restarts after a failure, writes dev-log state under `/var/lib/hermes`, and is restricted from changing the host system. It has no shell, device, namespace, or privileged Discord access.

## Update Hermes

Download and verify the next signed archive, install the new binary over `/usr/local/bin/hermes`, then restart the service:

```bash
sudo install -m 0755 hermes /usr/local/bin/hermes
sudo systemctl restart hermes.service
```

If a release fails verification, do not install it. Keep the previous binary until the new archive passes both checksum and Sigstore verification.
