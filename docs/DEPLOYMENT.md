# Hermes Watch deployment

Hermes is a long-lived Discord gateway process. GitHub Pages hosts the project page, but the bot itself must run on an always-on Linux host such as a small VPS, homelab server, or always-on workstation.

The supported deployment shape is a dedicated `hermes` system user, a root-owned environment file, a signed release archive, and a hardened `systemd` service.

## Oracle Always Free ARM setup

Create the account at [Oracle Cloud Free Tier](https://signup.oraclecloud.com/). Choose the home region carefully: Always Free compute must be provisioned there. Oracle may require phone and card verification; its documentation says the card is not charged unless the account is upgraded. The free A1 allocation is 2 OCPUs and 12 GB of memory, subject to regional capacity and idle-instance reclamation.

In the OCI console:

1. Open **Compute → Instances → Create instance**.
2. Name it `hermes-watch`.
3. Choose an **Always Free Eligible** Ubuntu ARM64 image.
4. Change shape to **VM.Standard.A1.Flex**, using **1 OCPU and 6 GB RAM** for Hermes.
5. Create or select a VCN with a public subnet and assign a public IPv4 address.
6. Add an SSH public key generated on your machine. Do not create a password-only login.
7. Do not open an inbound Discord port. Hermes connects outbound to Discord over HTTPS/WebSocket; SSH port 22 is the only port needed for administration.

Oracle can report `Out of host capacity` for Always Free shapes. If that happens, try another availability domain in the home region or retry later. Keep the VM active enough to avoid Oracle's idle-resource reclamation policy.

After the instance receives a public IP, connect with the Ubuntu account and install the signed `aarch64-unknown-linux-gnu` archive below.

## Install a signed release

On the target Linux host, install `cosign`, then choose a published version. Use the `aarch64-unknown-linux-gnu` archive for Oracle Ampere A1:

```bash
VERSION=v0.1.0
TARGET=aarch64-unknown-linux-gnu
curl -fL "https://github.com/darkstardevx/hermes/releases/download/${VERSION}/hermes-${VERSION}-${TARGET}.tar.gz" -o hermes.tar.gz
curl -fL "https://github.com/darkstardevx/hermes/releases/download/${VERSION}/hermes-${VERSION}-${TARGET}.tar.gz.sha256" -o hermes.tar.gz.sha256
curl -fL "https://github.com/darkstardevx/hermes/releases/download/${VERSION}/hermes-${VERSION}-${TARGET}.tar.gz.sigstore.json" -o hermes.tar.gz.sigstore.json

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
