# Docker Setup for Observation Tools Server

This guide explains how to run the Observation Tools Server using Docker.

## Quick Start

### Using Docker Compose (Recommended)

The easiest way to run the server is using Docker Compose:

```bash
# Build and start the server
docker compose up -d

# View logs
docker compose logs -f

# Stop the server
docker compose down
```

The server will be available at `http://localhost:3000`.

### Using Docker directly

Build the image:

```bash
docker build -t observation-tools-server .
```

Run the container:

```bash
docker run -d \
  -p 3000:3000 \
  -v observation-data:/data \
  --name observation-tools-server \
  observation-tools-server
```

## Configuration

The server is configured using environment variables:

| Variable | Description | Default |
|----------|-------------|---------|
| `HOST` | Server bind address | `0.0.0.0` |
| `PORT` | Server port | `3000` |
| `DATA_DIR` | Directory for data storage | `/data` |
| `STATIC_DIR` | Directory for static files | `/app/static` |
| `RUST_LOG` | Logging level (trace, debug, info, warn, error) | `info` |

### Custom Configuration

You can customize the configuration by:

1. **Creating a `.env` file:**

   ```bash
   cp .env.example .env
   # Edit .env with your settings
   ```

2. **Modifying docker-compose.yml:**

   Edit the `environment` section in `docker-compose.yml`:

   ```yaml
   environment:
     - HOST=0.0.0.0
     - PORT=8080
     - RUST_LOG=debug
   ```

3. **Using environment variables with Docker:**

   ```bash
   docker run -d \
     -p 8080:8080 \
     -e PORT=8080 \
     -e RUST_LOG=debug \
     -v observation-data:/data \
     observation-tools-server
   ```

## Data Persistence

Data is stored in a Docker volume named `observation-data`. This ensures your data persists across container restarts and updates.

To back up your data:

```bash
# Create a backup
docker run --rm \
  -v observation-data:/data \
  -v $(pwd):/backup \
  ubuntu tar czf /backup/observation-data-backup.tar.gz /data
```

To restore from a backup:

```bash
# Restore from backup
docker run --rm \
  -v observation-data:/data \
  -v $(pwd):/backup \
  ubuntu tar xzf /backup/observation-data-backup.tar.gz -C /
```

## Development

For local development without Docker, you can run the server directly:

```bash
# Set environment variables (optional)
export PORT=3000
export DATA_DIR=.observation-tools
export RUST_LOG=debug

# Run the server
cargo run --release --bin observation-tools
```

## Troubleshooting

### Port already in use

If port 3000 is already in use, change the port mapping in `docker-compose.yml`:

```yaml
ports:
  - "8080:3000"  # Map host port 8080 to container port 3000
```

Or with `docker run`:

```bash
docker run -d -p 8080:3000 -v observation-data:/data observation-tools-server
```

### View logs

```bash
# Docker Compose
docker compose logs -f

# Docker
docker logs -f observation-tools-server
```

### Container won't start

Check the logs for errors:

```bash
docker compose logs
```

Ensure the port is not in use:

```bash
lsof -i :3000
```

## Upgrading

To upgrade to a new version:

```bash
# Pull the latest code
git pull

# Rebuild and restart
docker compose up -d --build
```

Data will be preserved in the volume.
