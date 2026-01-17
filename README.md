# oxgate

Secure authentication service powered by Ory Hydra.

## Architecture

```
┌─────────────────┐
│  oxgate-web     │  Next.js Frontend
│  (Port 3000)    │
└────────┬────────┘
         │
         ▼
┌─────────────────┐     ┌─────────────────┐
│  oxgate-api     │────▶│  Ory Hydra      │
│  (Port 8080)    │     │  OAuth2 Server  │
│  Rust/axum      │     │  (Port 4444)    │
└────────┬────────┘     └─────────────────┘
         │
         ▼
┌─────────────────┐
│  PostgreSQL     │
│  (Port 5432)    │
└─────────────────┘
```

## Features

### Implemented (Phases 1-6)

- **Phase 1**: Login authentication
- **Phase 2**: OAuth2 consent handling
- **Phase 3**: Logout flow
- **Phase 4**: User registration and password reset
- **Phase 5**: Two-factor authentication (TOTP)
- **Phase 6**: Social login (Google/GitHub)

### Phase 7 (Current): Frontend

- Next.js 15 web application
- All authentication flows
- User management UI
- 2FA settings page

## Quick Start

### Prerequisites

- Docker & Docker Compose
- Rust 1.84+ (for local development)
- Bun 1.x (for frontend development)

### Using Docker Compose (Recommended)

```bash
# Start all services
docker-compose up -d

# Check logs
docker-compose logs -f

# Stop all services
docker-compose down
```

Services will be available at:
- oxgate-web: http://localhost:3000
- oxgate-api: http://localhost:8080
- Hydra Public: http://localhost:4444
- Hydra Admin: http://localhost:4445
- PostgreSQL: localhost:5432

### Local Development

#### Backend (oxgate-api)

```bash
# Copy environment file
cp .env.example .env

# Run migrations
sqlx migrate run

# Start development server
cargo run
```

#### Frontend (oxgate-web)

```bash
cd oxgate-web

# Install dependencies
bun install

# Copy environment file
cp .env.local.example .env.local

# Start development server
bun dev
```

## Environment Variables

See `.env.example` for all available configuration options.

### Required Variables

```bash
# Database
DATABASE_URL=postgres://oxgate:password@localhost:5432/oxgate

# Hydra
HYDRA_ADMIN_URL=http://localhost:4445

# 2FA & OAuth
TOTP_ISSUER=oxgate
ENCRYPTION_KEY=<32-byte-base64>
OAUTH_STATE_SECRET=<32-byte-base64>
```

### Optional Variables

```bash
# Email (for password reset)
SMTP_HOST=smtp.example.com
SMTP_PORT=587
SMTP_USERNAME=noreply@example.com
SMTP_PASSWORD=<your-smtp-password>
SMTP_FROM=noreply@example.com

# Social Login
GOOGLE_CLIENT_ID=<your-google-client-id>
GOOGLE_CLIENT_SECRET=<your-google-client-secret>
GITHUB_CLIENT_ID=<your-github-client-id>
GITHUB_CLIENT_SECRET=<your-github-client-secret>
```

## Project Structure

```
oxgate/
├── src/                    # Rust backend source
│   ├── handlers/          # HTTP handlers
│   ├── services/          # Business logic
│   ├── repositories/      # Database access
│   ├── models/            # Data models
│   └── ...
├── oxgate-web/            # Next.js frontend
│   ├── src/
│   │   ├── app/          # Pages
│   │   ├── components/   # UI components
│   │   └── lib/          # Utilities
│   └── ...
├── migrations/            # Database migrations
├── docs/                  # Documentation
├── docker-compose.yml     # Docker orchestration
├── Dockerfile            # Backend container
└── .env.example          # Environment template
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/health` | Health check |
| POST | `/api/login` | User authentication |
| POST | `/api/consent` | OAuth2 consent |
| POST | `/api/logout` | Logout |
| POST | `/api/register` | User registration |
| POST | `/api/password-reset/request` | Request password reset |
| POST | `/api/password-reset/confirm` | Confirm password reset |
| POST | `/api/2fa/setup` | Setup 2FA |
| POST | `/api/2fa/verify` | Verify 2FA |
| POST | `/api/2fa/disable` | Disable 2FA |
| GET | `/api/oauth/google` | Google OAuth |
| GET | `/api/oauth/github` | GitHub OAuth |

## Testing

### Backend

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_name
```

### Frontend

```bash
cd oxgate-web

# Type check
bun run type-check

# Lint
bun run lint
```

## Security

This is an authentication service with strict security requirements:

- **No `.unwrap()` or `panic!()`** in production code
- **Password hashing** with Argon2
- **Timing attack protection**
- **SQL injection prevention** (parameterized queries)
- **Secrets protection** with `SecretBox`
- **HTTPS required** in production

See `docs/03_security.md` for details.

## Documentation

- [Requirements](docs/01_requirements.md)
- [API Design](docs/02_api_design.md)
- [Security Guidelines](docs/03_security.md)
- [Hydra Integration](docs/04_hydra_integration.md)

## Development Workflow

```bash
# Format code
cargo fmt

# Run linter
cargo clippy -- -D warnings

# Run tests
cargo test

# Build for production
cargo build --release
```

## Production Deployment

1. Update environment variables (especially secrets!)
2. Enable HTTPS/TLS
3. Configure CORS properly
4. Set up monitoring and logging
5. Run database migrations
6. Build and deploy containers

## License

This project is part of the oxgate authentication service.
