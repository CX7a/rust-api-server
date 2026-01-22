# CompileX7 Backend - Rust API Server


![enter image description here](https://olive-chemical-haddock-701.mypinata.cloud/ipfs/bafybeifdyj4wobitac5sl4v6bnegjfbe7mnyt43ebmw6bec2lp6odpkysi)  

A high-performance, modular backend for CompileX7 - an AI-powered code engineering platform. Built with Rust, Axum, and PostgreSQL.

  

## Features

  

-  **AI-Powered Code Analysis**: Optimize, review, and refactor code using advanced AI

-  **Multi-Agent System**: Specialized agents for frontend, backend, and QA automation

-  **Project Management**: Full CRUD operations for development projects

-  **Code Analysis Engine**: Complexity analysis, security scanning, and performance detection

-  **Financial Analytics**: Dashboard and reporting for project metrics

-  **Authentication**: JWT-based authentication with secure password hashing

-  **Database Integration**: PostgreSQL with automatic migrations

-  **Async Architecture**: Built on Tokio for high-concurrency operations

-  **Error Handling**: Comprehensive error types and responses

-  **Modular Design**: Clean separation of concerns with services, handlers, and models

  

## Project Structure

  

```

src/
├── main.rs                    # Server bootstrap & route configuration
│
├── config/                     # Configuration management
│   └── mod.rs                  # Environment & app settings
│
├── db/                         # Database layer
│   ├── mod.rs                  # Database initialization
│   └── pool.rs                 # Connection pooling
│
├── models/                     # Data models & DTOs
│   ├── mod.rs
│   ├── user.rs                 # User domain models
│   ├── project.rs              # Project entities
│   └── analysis.rs             # Code analysis models
│
├── handlers/                   # HTTP request handlers (controllers)
│   ├── mod.rs
│   ├── auth.rs                 # Authentication endpoints
│   ├── projects.rs             # Project management APIs
│   ├── code_analysis.rs        # Code analysis endpoints
│   ├── agents.rs               # Multi-agent orchestration APIs
│   └── analytics.rs            # Analytics & reporting endpoints
│
├── services/                   # Core business logic
│   ├── mod.rs
│   ├── ai.rs                   # AI / LLM integration
│   ├── agent.rs                # Agent lifecycle & coordination
│   ├── code_analysis.rs        # Static & AI-assisted analysis logic
│   └── analytics.rs            # Metrics & insight processing
│
├── utils/                      # Shared utilities
│   ├── mod.rs
│   ├── jwt.rs                  # JWT creation & verification
│   ├── validation.rs           # Request validation helpers
│   └── crypto.rs               # Password hashing & security helpers
│
├── middleware_auth/            # Authentication middleware
│   ├── mod.rs
│   └── auth.rs                 # Request guards & token validation
│
├── error/                      # Centralized error handling
│   ├── mod.rs
│   └── api_error.rs            # API error definitions & mapping


```

  

## API Endpoints

  

### Authentication

-  `POST /auth/register` - Register new user

-  `POST /auth/login` - Login with credentials

-  `POST /auth/refresh` - Refresh access token

-  `POST /auth/logout` - Logout

  

### Projects

-  `GET /projects` - List all projects

-  `POST /projects` - Create new project

-  `GET /projects/:id` - Get project details

-  `PUT /projects/:id` - Update project

-  `DELETE /projects/:id` - Delete project

-  `GET /projects/:id/files` - List project files

  

### Code Analysis

-  `POST /analysis/optimize` - Optimize code

-  `POST /analysis/review` - Review code

-  `POST /analysis/refactor` - Refactor code

  

### Agents

-  `POST /agents/frontend` - Execute frontend agent

-  `POST /agents/backend` - Execute backend agent

-  `POST /agents/qa` - Execute QA agent

-  `GET /agents/status/:task_id` - Get agent task status

  

### Analytics

-  `GET /analytics/dashboard` - Get dashboard metrics

-  `GET /analytics/metrics` - Get historical metrics

-  `GET /analytics/reports` - List analytics reports

  

## Prerequisites

  

- Rust 1.70+ ([Install Rust](https://rustup.rs/))

- PostgreSQL 12+

- Git

  

## Setup & Installation

![enter image description here](https://olive-chemical-haddock-701.mypinata.cloud/ipfs/bafybeiabpycgm6f2pnparh26gylyzlkizwzuk7q5rf3aph2q6nz6bg7gom)
  

### 1. Clone the repository

```bash

git  clone  https://github.com/CX7a/backend.git
cd  compilex7-backend

```

  

### 2. Install Rust dependencies

```bash

cargo  build

```

  

### 3. Create PostgreSQL database

```bash

createdb  compilex7

```

  

### 4. Configure environment variables

Create a `.env` file in the project root:

  

```env

# Server Configuration

SERVER_ADDR=0.0.0.0:8080

ENVIRONMENT=development

LOG_LEVEL=info

  

# Database

DATABASE_URL=postgresql://username:password@localhost:5432/compilex7

  

# JWT Configuration

JWT_SECRET=your_super_secret_key_change_in_production

JWT_EXPIRY=3600

  

# AI Integration

AI_API_KEY=your_openai_api_key

AI_API_URL=https://api.openai.com/v1

```

  

### 5. Run the server

```bash

cargo  run

```

  

The server will be available at `http://localhost:8080`

  

## Development

  

### Building for production

```bash

cargo  build  --release

```

  

### Running tests

```bash

cargo  test

```

  

### Running with watch mode

```bash

cargo  install  cargo-watch

cargo  watch  -x  run

```

  

### Database migrations

The database migrations run automatically on server startup. Check `src/db/mod.rs` for the schema.

  

## API Usage Examples

  

### Register User

```bash

curl  -X  POST  http://localhost:8080/auth/register  \
-H "Content-Type: application/json" \
-d  '{
"email": "user@example.com",
"password": "SecurePass123",
"first_name": "John",
"last_name": "Doe"
}'

```

  

### Login

```bash

curl  -X  POST  http://localhost:8080/auth/login  \
-H "Content-Type: application/json" \
-d  '{
"email": "user@example.com",
"password": "SecurePass123"
}'

```

  

### Create Project

```bash

curl  -X  POST  http://localhost:8080/projects  \
-H "Content-Type: application/json" \
-H  "Authorization: Bearer YOUR_ACCESS_TOKEN"  \
-d '{
"name":  "My Project",
"description":  "Project description",
"language":  "rust",
"repository_url":  "https://github.com/user/repo"
}'

```

  

### Optimize Code

```bash

curl  -X  POST  http://localhost:8080/analysis/optimize  \
-H "Content-Type: application/json" \
-H  "Authorization: Bearer YOUR_ACCESS_TOKEN"  \
-d '{
"code":  "fn main() { for i in 0..10 { println!(\"{}\", i); } }",
"language":  "rust"
}'

```

  

## Architecture

  

### Request Flow

1. HTTP request arrives at Axum router

2. Authentication middleware validates JWT token

3. Request routed to appropriate handler

4. Handler delegates to service layer

5. Service layer performs business logic

6. Data layer interacts with PostgreSQL

7. Response serialized and returned to client

  

### Error Handling

Comprehensive error types defined in `src/error/mod.rs`:

-  `DatabaseError` - Database operation failures

-  `ValidationError` - Input validation failures

-  `AuthenticationError` - Auth token issues

-  `AuthorizationError` - Permission denied

-  `NotFoundError` - Resource not found

-  `ExternalApiError` - AI API failures

-  `InternalServerError` - Server errors

  

## Security

  

- JWT-based authentication with 1-hour access tokens

- Password hashing with bcrypt (cost factor 12)

- Secure token refresh mechanism (7-day refresh tokens)

- CORS protection enabled

- SQL injection prevention through parameterized queries

- Input validation on all endpoints

- Environment variable configuration (no hardcoded secrets)

  

## Performance

  

- Async/await architecture for high concurrency

- Connection pooling for database (5 connections)

- Non-blocking agent execution with Tokio tasks

- Efficient code analysis algorithms

- Optimized JWT token verification

  

## Deployment

  

### Docker

```dockerfile

FROM rust:latest as builder
WORKDIR /app
COPY . .
RUN cargo build --release
FROM debian:bookworm-slim
COPY --from=builder /app/target/release/compilex7 /usr/local/bin/
EXPOSE 8080
CMD ["compilex7"]

```

  

### Docker Compose

```yaml

version: '3'
services:
db:
image: postgres:15
environment:
POSTGRES_DB: compilex7
POSTGRES_PASSWORD: password
volumes:
- postgres_data:/var/lib/postgresql/data
api:
build: .
ports:
- "8080:8080"

environment:
DATABASE_URL: postgresql://postgres:password@db:5432/compilex7
JWT_SECRET: your_secret
AI_API_KEY: your_key
depends_on:
- db

volumes:
postgres_data:

```

  

## Scalability

  

- Stateless API design for horizontal scaling

- Connection pooling supports multiple instances

- Task-based agent execution allows parallel processing

- Event-driven analytics for asynchronous reporting

  

## Contributing

  

1. Fork the repository

2. Create a feature branch (`git checkout -b feature/amazing-feature`)

3. Commit changes (`git commit -m 'Add amazing feature'`)

4. Push to branch (`git push origin feature/amazing-feature`)

5. Open a Pull Request

  

## Testing

  

- Unit tests included in each module

- Integration tests for API endpoints

- Run tests with `cargo test`

- Use `cargo test -- --nocapture` to see output

  

## Troubleshooting

  

### Database connection failed

- Verify PostgreSQL is running

- Check DATABASE_URL is correct

- Ensure database exists: `createdb compilex7`

  

### JWT Secret not set

- Add `JWT_SECRET` to `.env` file

- Restart the server

  

### AI API errors

- Verify `AI_API_KEY` is valid

- Check `AI_API_URL` is correct

- Ensure API has appropriate rate limits

  

## Performance Monitoring

  

The analytics service tracks:

- Request counts and success rates

- Response times

- Code analysis metrics

- Active agent processes

- Project statistics

  

Access metrics at `GET /analytics/dashboard` and `GET /analytics/metrics`

  

## License

  

MIT License - See LICENSE file for details

  

## Support

  

For issues and feature requests, please create an issue on GitHub.

  

## Roadmap

  

- [ ] WebSocket support for real-time code collaboration

- [ ] Redis caching layer for performance

- [ ] Advanced ML-based code quality scoring

- [ ] Plugin system for custom agents

- [ ] GraphQL API support

- [ ] Enhanced financial analytics with charts

- [ ] Multi-tenant support

- [ ] Advanced role-based access control (RBAC)