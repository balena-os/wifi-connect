# Deploying WiFi Connect under Portainer (non-Balena hosts)

This fork ships with a Balena setup (`Dockerfile.template` + `scripts/start.sh`).
On the **rnoid (Jetson) robots** we deploy with **Portainer** instead of
Balena, with **NetworkManager running on the host**. This guide runs the exact
same image there, with no code fork.

## Why this works without changing the app

WiFi Connect only needs to talk to **NetworkManager over the host system
D-Bus** — it does not depend on Balena OS. `scripts/start.sh` already:

- waits `START_SLEEP` (default 25 s) for NetworkManager to auto-connect before
  deciding the robot is offline (avoids raising the AP on every boot),
- tries to auto-join **saved networks that are in range** before raising the AP,
- builds a **per-robot SSID** from `BOT_ID` + `PORTAL_SSID`,
- supports a no-AP station mode with basic auth.

The only host-specific detail is the **D-Bus socket path**. Balena bind-mounts
it at `/host/run/dbus/...`; `start.sh` now honours a `DBUS_SYSTEM_BUS_ADDRESS`
override so a plain Docker host can mount the socket wherever it likes. The
provided compose mounts the host socket at the same default path, so nothing
else changes.

## Deploy

Deploy as its **own stack** (not bundled with the application stack), so it
comes up with the Docker daemon and stays available **even when the main app
stack is down** — which is exactly when an operator needs to fix the network.

```bash
# On the Jetson host, or as a Portainer stack from this compose file:
BOT_ID=rnoid01 docker compose -f docker-compose.portainer.yml up -d --build
```

In Portainer: **Stacks → Add stack**, paste `docker-compose.portainer.yml`, set
the `BOT_ID` environment variable per robot, and deploy.

## What the container needs (and why)

| Requirement | Reason |
| --- | --- |
| `network_mode: host` | The AP and captive-portal web server live on the host's wireless interface |
| `/run/dbus:/host/run/dbus` | Reach the host NetworkManager over D-Bus |
| `cap_add: NET_ADMIN` (or `privileged`) | Bring up the AP / run dnsmasq; some hosts need full `privileged` |
| `restart: always` | Survive reboots and crashes; come up with the Docker daemon |

## Configuration (environment)

| Variable | Default | Meaning |
| --- | --- | --- |
| `DBUS_SYSTEM_BUS_ADDRESS` | `unix:path=/host/run/dbus/system_bus_socket` | Host D-Bus socket path (match the volume mount) |
| `START_SLEEP` | `25` | Seconds to wait for NetworkManager before deciding offline |
| `PORTAL_SSID` | _(unset)_ | Base captive-portal SSID |
| `BOT_ID` | _(unset)_ | Prefix for the SSID → `${BOT_ID}-${PORTAL_SSID}`; **keep unique per robot** |
| `RECONNECT_ATTEMPTS` / `RECONNECT_DELAY` | `2` / `5` | Retry tuning for auto-joining saved networks |
| `NO_AP_MODE_PASSWORD` | _(unset)_ | If set, serve the UI in station mode with basic auth when already online |
| `LAUNCH_APP` | `1` | Set `0` to disable raising the AP (debugging) |

## Venue networks (pre-provisioning)

WiFi Connect's portal is **scan + type-password**. To let a robot **auto-join**
a known venue with no operator interaction, pre-provision that network as a
saved NetworkManager profile on the host (once), e.g. via the fleet-provisioning
flow:

```bash
nmcli connection add type wifi con-name "Venue-Guest" ssid "Venue-Guest" \
  wifi-sec.key-mgmt wpa-psk wifi-sec.psk "<password>" \
  connection.autoconnect yes connection.autoconnect-priority 50
```

`start.sh`'s `try_saved_networks()` then joins it automatically when in range,
and the AP is only raised as a fallback for networks the robot doesn't know.

## Caveats

- **Single radio**: the robot can't be an AP and a station at once. When it
  raises the setup AP it drops off normal WiFi; any SSH/Tailscale session
  riding WiFi will disconnect. Drive tests over Ethernet or a serial console.
- **dnsmasq**: WiFi Connect runs its **own** dnsmasq bound to the AP gateway
  (default `192.168.42.1`). With `network_mode: host`, ensure nothing else on
  the host binds the same address/ports on the AP interface.
- **Break-glass**: because this runs in a container, it is unavailable if the
  Docker daemon itself is down. Deploying it as its own always-on stack keeps
  that window as small as possible (it only depends on `dockerd`, not the app
  stack).
- **Unique `BOT_ID`**: cloned images with the same `BOT_ID` would broadcast the
  same setup SSID — set it per robot.
