# oxgate-web

Next.js frontend for oxgate authentication service.

## Tech Stack

- **Framework**: Next.js 15 (App Router)
- **Language**: TypeScript 5.x
- **Runtime**: Bun
- **Styling**: Tailwind CSS 4
- **UI Components**: shadcn/ui
- **Form**: React Hook Form + Zod
- **HTTP**: TanStack Query

## Getting Started

### Prerequisites

- Bun 1.x
- Node.js 18+ (for compatibility)
- oxgate-api running on http://localhost:8080

### Installation

```bash
# Install dependencies
bun install

# Create environment file
cp .env.local.example .env.local

# Start development server
bun dev
```

The application will be available at http://localhost:3000

### Environment Variables

Create `.env.local`:

```env
NEXT_PUBLIC_API_URL=http://localhost:8080
```

## Project Structure

```
oxgate-web/
├── src/
│   ├── app/                  # Next.js App Router pages
│   │   ├── login/           # Login page
│   │   ├── consent/         # Consent page
│   │   ├── logout/          # Logout page
│   │   ├── register/        # User registration
│   │   ├── password-reset/  # Password reset pages
│   │   ├── settings/2fa/    # 2FA settings
│   │   ├── layout.tsx       # Root layout
│   │   └── providers.tsx    # TanStack Query provider
│   ├── components/
│   │   └── ui/              # shadcn/ui components
│   └── lib/
│       ├── api-client.ts    # API client
│       ├── validations.ts   # Zod schemas
│       └── utils.ts         # Utilities
├── public/                   # Static assets
├── package.json
├── tsconfig.json
├── tailwind.config.ts
└── next.config.ts
```

## Features

### Phase 1-3: Core OAuth2 Flow
- Login page (`/login?login_challenge=xxx`)
- Consent page (`/consent?consent_challenge=xxx`)
- Logout page (`/logout?logout_challenge=xxx`)

### Phase 4: User Management
- User registration (`/register`)
- Password reset request (`/password-reset/request`)
- Password reset confirm (`/password-reset/confirm?token=xxx`)

### Phase 5: Two-Factor Authentication
- 2FA setup and management (`/settings/2fa`)
- QR code display
- TOTP code verification

## Development

### Commands

```bash
# Development
bun dev

# Build
bun run build

# Start production server
bun start

# Lint
bun run lint

# Type check
bun run type-check
```

### Code Style

- **TypeScript**: Strict mode enabled
- **Formatting**: Follows Next.js conventions
- **Components**: Client components for interactivity
- **Forms**: React Hook Form with Zod validation
- **API**: TanStack Query for data fetching

## Docker

### Build

```bash
docker build -t oxgate-web .
```

### Run

```bash
docker run -p 3000:3000 \
  -e NEXT_PUBLIC_API_URL=http://localhost:8080 \
  oxgate-web
```

## API Integration

All API calls go through the centralized API client (`src/lib/api-client.ts`).

### Example Usage

```typescript
import { apiClient } from "@/lib/api-client";
import { useMutation } from "@tanstack/react-query";

const loginMutation = useMutation({
  mutationFn: (data) => apiClient.login({
    login_challenge: challenge,
    email: data.email,
    password: data.password,
  }),
  onSuccess: (data) => {
    window.location.href = data.redirect_to;
  },
});
```

## Security

- **No Credentials Storage**: All authentication flows redirect to Hydra
- **Challenge-Based**: Uses OAuth2 challenge-response pattern
- **HTTPS Only**: Production deployments must use HTTPS
- **CORS**: Only accepts requests from configured origins

## License

This project is part of the oxgate authentication service.
