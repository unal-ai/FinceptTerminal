# Fincept Terminal - Production Deployment Guide

This guide covers deploying the Fincept Terminal Web Server in a production environment.

## Table of Contents
- [Prerequisites](#prerequisites)
- [Deployment Options](#deployment-options)
- [Docker Deployment](#docker-deployment)
- [Direct Deployment](#direct-deployment)
- [SSL/TLS Configuration](#ssltls-configuration)
- [Monitoring & Logging](#monitoring--logging)
- [Security Checklist](#security-checklist)
- [Troubleshooting](#troubleshooting)

## Prerequisites

- Docker and Docker Compose (for containerized deployment)
- Rust 1.75+ (for direct deployment)
- Python 3.11+ (for analytics scripts)
- SSL certificate (for HTTPS)
- Domain name (recommended)

## Deployment Options

### Option 1: Docker Compose (Recommended)

The simplest way to deploy for production:

```bash
# Clone the repository
git clone https://github.com/Fincept-Corporation/FinceptTerminal.git
cd FinceptTerminal

# Copy and configure environment
cp .env.production.example .env.production
# Edit .env.production with your settings

# Deploy with production profile
docker-compose -f docker-compose.web.yml --profile production up -d
```

### Option 2: Docker Only

```bash
# Build the image
docker build -t fincept-server -f Dockerfile.web .

# Run with environment variables
docker run -d \
  --name fincept-server \
  -p 3000:3000 \
  -v fincept-data:/app/data \
  -e FINCEPT_HOST=0.0.0.0 \
  -e FINCEPT_PORT=3000 \
  -e RUST_LOG=info \
  --restart unless-stopped \
  fincept-server
```

### Option 3: Direct Binary

```bash
# Build the release binary
cd fincept-terminal-desktop/src-tauri
cargo build --release --features web --bin fincept-server

# Run the server
FINCEPT_HOST=0.0.0.0 \
FINCEPT_PORT=3000 \
RUST_LOG=info \
./target/release/fincept-server
```

## Docker Deployment

### Full Stack with Nginx

```bash
# Start all services including Nginx reverse proxy
docker-compose -f docker-compose.web.yml --profile production up -d

# Check status
docker-compose -f docker-compose.web.yml ps

# View logs
docker-compose -f docker-compose.web.yml logs -f
```

### Scaling

To run multiple backend instances:

```bash
docker-compose -f docker-compose.web.yml up -d --scale fincept-server=3
```

Update `nginx.conf` to include all instances in the upstream block.

## SSL/TLS Configuration

### Using Let's Encrypt with Certbot

```bash
# Install certbot
apt-get install certbot

# Generate certificate
certbot certonly --standalone -d your-domain.com

# Copy certificates to nginx ssl directory
mkdir -p ./ssl
cp /etc/letsencrypt/live/your-domain.com/fullchain.pem ./ssl/
cp /etc/letsencrypt/live/your-domain.com/privkey.pem ./ssl/
```

### Update nginx.conf

Uncomment the HTTPS server block in `nginx.conf` and update:
- `server_name your-domain.com;`
- SSL certificate paths

### Auto-renewal

```bash
# Add to crontab
0 0 * * * certbot renew --quiet && docker-compose restart nginx
```

## Monitoring & Logging

### Log Formats

The server uses structured logging with the following format:
```
[request_id] METHOD /path -> STATUS (duration_ms)
```

### View Logs

```bash
# Docker logs
docker logs -f fincept-server

# With Docker Compose
docker-compose -f docker-compose.web.yml logs -f fincept-server
```

### Health Checks

The server exposes two health endpoints:

1. **Health Check** (`/api/health`)
   - Always returns 200 if server is running
   - Use for liveness probes

2. **Readiness Check** (`/api/ready`)
   - Returns 200 only if database is connected
   - Use for readiness probes in Kubernetes

```bash
# Health check
curl http://localhost:3000/api/health

# Readiness check
curl http://localhost:3000/api/ready
```

### Prometheus Metrics (Future)

Metrics endpoint can be enabled via environment variable:
```bash
FINCEPT_METRICS_ENABLED=true
FINCEPT_METRICS_PORT=9090
```

## Security Checklist

### Before Production

- [ ] Change default ports if needed
- [ ] Configure CORS origins to specific domains
- [ ] Enable SSL/TLS
- [ ] Set up firewall rules
- [ ] Configure rate limiting in Nginx
- [ ] Review and set appropriate timeouts
- [ ] Set up log rotation
- [ ] Configure backup for database volume
- [ ] Review environment variables

### Environment Variables

```bash
# Production settings
FINCEPT_HOST=0.0.0.0
FINCEPT_PORT=3000
RUST_LOG=warn  # Use 'warn' in production, 'info' or 'debug' for troubleshooting
```

### Firewall Rules

```bash
# Allow only necessary ports
ufw allow 80/tcp
ufw allow 443/tcp
ufw deny 3000/tcp  # Block direct access to backend
```

### Rate Limiting

The Nginx configuration includes rate limiting:
- API endpoints: 30 requests/second with burst of 50
- Health checks: 10 requests/second

Adjust in `nginx.conf` as needed.

## Troubleshooting

### Common Issues

#### Server won't start
```bash
# Check logs
docker logs fincept-server

# Verify port availability
netstat -tlnp | grep 3000
```

#### Database connection issues
```bash
# Check if database file exists
docker exec fincept-server ls -la /app/data/

# Check permissions
docker exec fincept-server id
```

#### SSL certificate issues
```bash
# Verify certificate
openssl x509 -in ./ssl/fullchain.pem -text -noout

# Test SSL connection
curl -v https://your-domain.com/api/health
```

### Performance Tuning

1. **Increase worker connections** in `nginx.conf`
2. **Adjust rate limits** based on expected traffic
3. **Scale horizontally** with multiple backend instances
4. **Enable response compression** (already configured in Nginx)

### Getting Help

- GitHub Issues: https://github.com/Fincept-Corporation/FinceptTerminal/issues
- Documentation: https://docs.fincept.in

---

*Last updated: 2026-01-07*
