<div align="center">
  <img src="frontend/public/favicon/opencab-main.png" alt="OpenCab Logo" width="350">

  **A secure patient management system for healthcare practitioners**

  Modern, privacy-focused application for managing patient records, companies, and generating professional invoices.

  [Features](#-features) • [Tech Stack](#️-tech-stack) • [Getting Started](#-getting-started) • [Development](#-development) • [Deployment](#-deployment)

</div>

---

## 📖 About

OpenCab is a full-stack web application designed to help healthcare practitioners manage their patient records securely. Built with a focus on data privacy and security, it provides encrypted storage for sensitive patient information including social security numbers, along with automated invoice generation and delivery capabilities.

Key capabilities include comprehensive patient management, company and intervention tracking with PDF invoice generation, medical appointment management with accounting exports, multi-office support, and automated email delivery through background job processing.

This is a personal project built to explore modern web technologies including Rust backend development, secure cryptographic practices, and asynchronous background job processing.

## ✨ Features

### Patient Management
- **Secure Patient Records** - Store patient information with AES-GCM encryption for sensitive data
- **SSN Protection** - Double-layer security with encrypted storage and hashed indexing for fast, secure lookups
- **Multi-Office Support** - Manage patients across multiple practitioner offices

### Appointment Management
- **Medical Appointments** - Create, edit, and delete medical appointments with patients
- **Appointment Tracking** - Track appointments with date, price, and office information
- **Accounting Export** - Export appointments to Excel format for accounting purposes, organized by office with automatic email delivery

### Invoice Generation & Delivery
- **PDF Invoices** - Generate professional PDF invoices with native Rust PDF generation
- **Digital Signatures** - Add practitioner signatures to invoices
- **Email Delivery** - Send invoices directly to patients via email with automatic PDF attachment
- **Custom Invoice Dates** - Set specific dates for invoices instead of using the current date
- **Business Information** - Complete practitioner business details integration

### Company Management
- **Company Directory** - Create and manage companies with contact details, SIRET number, and billing address
- **Company Interventions** - Track billable interventions per company with description, quantity, unit price (HT), and VAT rate (0%, 5.5%, 10%, 20%)
- **Company Invoices** - Generate and download PDF invoices for companies with automatic HT/TVA/TTC calculations
- **Intervention History** - View all past interventions and generated invoices per company

### Office Management
- **Multiple Offices** - Create and manage multiple practitioner office locations
- **Office-Patient Linking** - Associate patients with specific practitioner offices
- **Office Details on Invoices** - Automatically include office address and information on generated invoices

## 🛠️ Tech Stack

### Backend
- **[Rust](https://www.rust-lang.org/)** - Systems programming language for performance and safety
- **[Axum](https://github.com/tokio-rs/axum)** - Ergonomic and modular web framework built on Tokio
- **[SeaORM](https://www.sea-ql.org/SeaORM/)** - Async ORM for database operations
- **PostgreSQL** - Primary database (SQLite supported for development)
- **Background Workers** - Custom asynchronous job processing with Tokio channels for email delivery and long-running tasks
- **SMTP Integration** - Email delivery system via Lettre for invoice and appointment export distribution
- **Excel Generation** - Native Rust Excel file generation with `rust_xlsxwriter` for accounting exports

### Frontend
- **[TypeScript](https://www.typescriptlang.org/)** - Type-safe JavaScript
- **[Vite](https://vite.dev/)** - Fast build tool
- **[Bun](https://bun.sh/)** - JavaScript runtime and package manager
- **[React 19](https://react.dev/)** - UI library
- **[TanStack](https://tanstack.com)** - Type-safe routing
- **[Tailwind CSS 4](https://tailwindcss.com/)** - Utility-first CSS framework
- **[ShadCN](https://ui.shadcn.com/)** - Accessible UI components
- **[React Hook Form](https://react-hook-form.com/)** - Form management with Zod validation

### Security & Encryption
- **AES-GCM** - Symmetric encryption for sensitive data
- **Argon2** - Password hashing
- **Base64** - Encoding for encrypted data

### DevOps
- **Docker** - Multi-stage optimized builds
- **Google Cloud Run** - Serverless container deployment
- **Distroless Images** - Minimal attack surface for production

## 🚀 Getting Started

### Prerequisites
- Rust 1.88+ ([Install](https://rustup.rs/))
- Bun ([Install](https://bun.sh/docs/installation))
- PostgreSQL 15+ or SQLite for development
- Docker (optional, for containerized deployment)

### Environment Setup

Create a `.env.local` file in the root directory (or simply copy the `.env` file):

```env
# Database
DATABASE_URL=postgres://user:password@localhost:5432/opencab

# Encryption Keys (generate secure random keys)
ENCRYPTION_KEY=your-32-byte-base64-encoded-key
SSN_SALT_KEY=your-secure-salt-key

# JWT
JWT_SECRET=your-jwt-secret-key

# SMTP Email Configuration (for invoice delivery)
SMTP_SERVER_HOST=smtp.your-provider.com
SMTP_SERVER_PORT=465
SMTP_AUTH_USER=your-email@example.com
SMTP_AUTH_PASSWORD=your-smtp-password

# Supabase Storage (optional, for invoice storage)
SUPABASE_URL=your-supabase-url
SUPABASE_KEY=your-supabase-key
SUPABASE_BUCKET=your-bucket-name
```

### Installation

1. **Clone the repository**
```bash
git clone https://github.com/yourusername/opencab.git
cd opencab
```

2. **Install backend dependencies**
```bash
cargo build
```

3. **Install frontend dependencies**
```bash
cd frontend
bun install
```

4. **Run database migrations**
```bash
cargo run --bin migration up
```

5. **Start the development servers**

In one terminal (backend):
```bash
cargo run
```

In another terminal (frontend):
```bash
cd frontend
bun run dev
```

The application will be available at `http://localhost:5173` (frontend) with API at `http://localhost:5150` (backend).

## 💻 Development

### Project Structure
```
opencab/
├── src/
│   ├── main.rs             # Application entry point & setup
│   ├── app_state.rs        # Shared application state
│   ├── router.rs           # HTTP routing configuration
│   ├── controllers/        # HTTP request handlers
│   ├── models/             # Database models & business logic
│   ├── services/           # Business services (crypto, invoice, etc.)
│   ├── validators/         # Request validation logic
│   ├── workers/            # Background job workers (Tokio channels)
│   ├── middleware/         # Custom middleware (auth, etc.)
│   └── config/             # Configuration structs
├── frontend/
│   ├── src/
│   │   ├── routes/         # TanStack Router routes
│   │   ├── components/     # React components
│   │   ├── api/            # API client & types
│   │   ├── hooks/          # Custom React hooks
│   │   └── i18n/           # Translations
│   └── public/             # Static assets
├── migration/              # Database migrations (SeaORM)
├── config/                 # YAML environment configurations
└── dockerfile              # Multi-stage production build
```

### Database Migrations

Create a new migration:
```bash
sea-orm-cli migrate generate migration_name
```

Run migrations:
```bash
cargo run --bin migration up
```

Rollback last migration:
```bash
cargo run --bin migration down
```

### Code Quality

Backend linting:
```bash
cargo clippy
cargo fmt
```

Frontend linting:
```bash
cd frontend
bun run lint
```

### Testing

Run Rust tests:
```bash
cargo test
```

## 🚢 Deployment

### Docker Build

The project includes an optimized multi-stage Dockerfile:

```bash
# Build the image
docker build -t opencab .

# Run the container
docker run -p 5150:5150 --env-file .env opencab
```

The Dockerfile uses:
- **cargo-chef** for faster Rust dependency caching
- **Bun** for fast frontend builds
- **Distroless** base image for minimal attack surface
- **Multi-stage builds** for optimal layer caching

## Building the Docker Image

Using the `dockerfile` file, you can build your Docker container image.

From the `oven/bun` image, build the frontend. **→ Apple users:** disable the _Use Rosetta for x86_64/amd64 emulation on Apple Silicon_ from the Docker Desktop settings. We will build a `linux/amd64` image and its seems that [tailwind has issues with Apple Rosetta to build our CSS](https://github.com/tailwindlabs/tailwindcss/issues/18315#issuecomment-2984442515).

From the `rust` image, build the backend.
Then copy all the files from the frontend and backend builds into a new `distroless/cc-debian12` image.

⚠️ **Backend configuration**:

- `localhost` won't work for the host in production as you will access the service from outside the container (with a ping on the domain name that will point to the container). Thus the host must be `0.0.0.0`.
- For the CORS configuration, you can set the `Access-Control-Allow-Origin` header to `*` or to your Google Cloud Run domain (as it is Client Side Rendering).

⚠️ **Backend build**:

- When running a simple `cargo run --release`, the binary is built for the host architecture. If you want to build it for a specific target, you can use `cargo build --release --target x86_64-unknown-linux-gnu` (or any other target). Google Cloud Run uses `x86_64-unknown-linux-gnu` as the target architecture, so you should build your binary for that target.
- The docker image architecture must be a `linux/amd64` image. When running a simple `docker build . -t test`, it will build the image for the host architecture. If you want to build it for a specific target, you can use `docker build . -t test --platform linux/amd64` (or any other target).

## Deploying on Google Cloud Run

**Prerequisites**: your docker image must be pulled on a public or private registry accessible by Google Cloud Run. For instance, you can use Docker Hub (just push your built image to your Docker Hub account).

To deploy your application on Google Cloud Run, visit the [Google Cloud Console](https://console.cloud.google.com/run) and follow these steps:

- Create a global project if you don't have one.
- Enable the Cloud Run API.
- Visit the Cloud Run page and click on "Deploy a container".

**Service Configuration**:

- **Container image URL**: the URL of your Docker image (e.g., `your-id/your-image-name`).
- **Service name**: the name of your service (e.g., `my-app`).
- **Region**: select the region where you want to deploy your service (e.g., `europe-west9`).
- **Authentication**: If you want to use JWT authentication, you can allow unauthenticated invocations.
- **Billing**: First option, serverless, is the best option for small applications.

Then proceed to the container configuration.

**Container Configuration**:

- **Container port**: set the Google Cloud Run default port (8080). **⚠️ The `server.port` value must be read from the PORT Google Cloud Run env variable**. Otherwise, you app would not be able to restart properly after a cold start. To do that change the hardcoded from to `{{ get_env(name="PORT", default="5150") }}`.
- **Environment variables**: you can set environment variables for your application (e.g., `DATABASE_URL`, `JWT_SECRET`, `ENVIRONMENT` etc.).
- **Health checks**: you can enable health checks to ensure your service is running correctly. In our case, select HTTP and the path to your health check endpoint. Don't forget to **add a small delay** because your application might take some time to start up (e.g., 10 seconds).

Then proceed to the last steps.

- **Revision scaling**: reduce the number of instances to 3 (can be changed later to upgrade)

## 🔒 Security Considerations

This project is provided as-is for educational and personal use.
