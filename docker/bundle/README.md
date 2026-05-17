# HAAP Local Evaluation Bundle

A Docker Compose bundle that spins up a complete HAAP customer-side stack for
local evaluation:

- **CAA** (Customer Admin Agent) — two processes:
  - `caa-admin-auth` holds IK_c, runs the Admin Authenticator
  - `caa` runs the Admin Orchestrator (gRPC AdminControlPlane surface)
- **RSV** (Resource Server Verifier) — verifies HAAP tokens on inbound API requests
- **Redis** — replay-protection store for RSV

## Quick start

```bash
cd docker/bundle
cp .env.example .env
# Edit .env — provide HAWCX_ORG_ID, HAWCX_IK_C, HAAP_AUDIENCE_HASH,
# HAAP_SEALER_PASSPHRASE, and HAAP_BOOTSTRAP_OTRC.
docker compose up
```

When the bundle is up:

- CAA gRPC (AdminControlPlane): `localhost:9443`
- RSV HTTP verification endpoint: `localhost:8443`

## Prerequisites

For the services to actually do useful work, you need values that Hawcx
provisions for your tenant:

| Variable | Purpose |
|---|---|
| `HAWCX_ORG_ID` | Organization identifier |
| `HAWCX_IK_C` | Admin identity key (or `HAWCX_IK_C_FILE` path) |
| `HAAP_BOOTSTRAP_OTRC` | One-time recovery credential for first-run enrollment |
| `HAAP_AUDIENCE_HASH` | 32-byte hex audience hash registered with your AS |

Without these, the containers will start but fail to initialize. Contact your
Hawcx representative for evaluation credentials.

## What you can do with this bundle

- Stand up CAA + RSV locally for integration testing
- Develop customer-side code that consumes the AdminControlPlane gRPC API
- Issue test HAAP tokens against a local RSV
- Demonstrate HAAP integration in a sandbox without Kubernetes

## What this bundle does NOT cover

- **Production deployment** — evaluation only. Production uses Kubernetes Helm
  charts (future work). No TLS termination, no HA, no horizontal scaling.
- **AS (Authorization Server)** — the bundle points to Hawcx's AS by default
  (`HAAP_AS_URL`). Customers running their own AS configure this env var.
- **Agent runtime** — customers run their own agents. The Python SDK for
  CrewAI integration ships separately (deferred to v0.2).
- **mTLS certificates** — the bundle runs in non-production mode. Production
  needs `HAAP_CAA_MTLS_CERT`, `HAAP_CAA_MTLS_KEY`, and pinned server certs.

## Architecture details

### Why two CAA containers?

CAA is intentionally split into two processes with a strict trust boundary:

- `haap-admin-auth` is the only process that holds the customer identity key
  (`IK_c`). It exposes operations over a Unix domain socket.
- `haap-customer-admin-agent` (the orchestrator) handles routing, gRPC, and
  AS communication, but holds no cryptographic material.

They communicate via Unix socket at `/var/run/hawcx/admin-control.sock`,
mounted into both containers from the `caa_ipc` named volume.

### Why no Postgres?

CAA persists state to a file directory (`HAAP_CAA_STATE_DIR=/var/lib/haap/caa`)
backed by the `caa_state` named volume. Postgres is not required for alpha-1.

### Why no healthchecks on CAA/RSV?

Both images use `gcr.io/distroless/cc-debian12` as a base — no shell, `nc`,
`curl`, or `wget`. The binaries do not expose a `--health-check` flag. The
host-side `smoke-test.sh` probes TCP ports for liveness instead.

## Logs

```bash
docker compose logs -f caa
docker compose logs -f caa-admin-auth
docker compose logs -f rsv
```

## Stop / restart

```bash
docker compose down                # stop, keep volumes
docker compose down -v             # stop, remove volumes (FRESH START)
docker compose restart caa         # restart just CAA orchestrator
```

## Updating the version

Edit `HAAP_VERSION` in `.env` and:

```bash
docker compose pull
docker compose up -d
```

## Smoke test

The bundled `smoke-test.sh` brings up the bundle, waits for ports to open,
and verifies basic TCP reachability.

```bash
./smoke-test.sh
```

## Troubleshooting

**`caa-admin-auth` exits immediately:** Check that `HAWCX_IK_C` is set and
non-empty. The admin-auth process refuses to start without identity key
material.

**`caa` orchestrator logs say "ensure haap-admin-auth-bin is running":**
The orchestrator can't reach the admin-auth process over the Unix socket.
Verify the `caa_ipc` volume is mounted at `/var/run/hawcx` on both containers
and that `caa-admin-auth` is running (`docker compose ps`).

**RSV exits immediately:** Check `HAAP_AUDIENCE_HASH` is set to 64 hex chars
and that Redis is reachable on the `hawcx-net` network.

**Port conflicts:** Adjust `CAA_GRPC_PORT` / `RSV_PORT` in `.env`.

**CAA fails with "HAWCX_ORG_ID is required":** This is enforced by the
orchestrator's startup validator (Phase 2y). Set `HAWCX_ORG_ID` in `.env`.

## See also

- [HAAP SDK documentation](https://github.com/hawcx/hx_agentic_sdk/blob/main/README.md)
- [CAA documentation](https://github.com/hawcx/hx_agent_client_admin_service/blob/main/README.md)
- Production deployment guide — TBD with v0.2
- Python SDK for CrewAI — TBD with v0.2
