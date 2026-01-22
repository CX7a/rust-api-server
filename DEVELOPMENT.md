# Comprehensive Rust Backend Development Guide

## Quick Start

```bash
# Clone repository
git clone https://github.com/compilex7/backend.git
cd compilex7-backend

# Copy environment file
cp .env.example .env

# Start with Docker Compose (includes PostgreSQL)
docker-compose up -d

# Or setup locally
createdb compilex7
export DATABASE_URL="postgresql://username:password@localhost:5432/compilex7"
cargo run
```

Server runs on `http://localhost:8080`

## Architecture Overview

### Layered Architecture
```
┌─────────────────────────────────────┐
│        HTTP Handlers (Axum)         │
├─────────────────────────────────────┤
│       Business Logic Services       │
├─────────────────────────────────────┤
│        Database Layer (SQLx)        │
├─────────────────────────────────────┤
│      PostgreSQL Database            │
└─────────────────────────────────────┘
```

### Module Organization
- **handlers/**: Request processing and response formatting
- **services/**: Core business logic and algorithms
- **models/**: Data structures and DTOs
- **db/**: Database queries and migrations
- **utils/**: Helper functions and utilities
- **middleware_auth/**: Cross-cutting concerns
- **error/**: Error types and handling

## Key Components

### 1. Authentication System
- JWT token generation and validation
- Bcrypt password hashing
- Refresh token mechanism
- Role-based access control ready

**Security Features:**
- 1-hour access token expiry
- 7-day refresh token expiry
- Secure secret management via environment variables
- Password validation (minimum 8 chars, uppercase, numbers)

### 2. Multi-Agent System
Three specialized agents collaborate on code projects:

```rust
// Frontend Agent: UI/UX code generation
// Backend Agent: API and server code
// QA Agent: Test suite generation
```

Agents execute asynchronously using Tokio tasks, enabling parallel processing of multiple requests.

### 3. Code Analysis Engine
- **Complexity Analysis**: Cyclomatic complexity calculation
- **Security Scanning**: SQL injection, code injection, plaintext credentials detection
- **Performance Analysis**: Nested loop detection, excessive cloning, optimization suggestions
- **Maintainability Score**: Based on code size and comment ratio

### 4. AI Integration Service
Connects to OpenAI's ChatGPT API for:
- Code optimization recommendations
- Comprehensive code reviews
- Intelligent refactoring
- Performance improvement suggestions

**Configuration:**
```env
AI_API_KEY=sk-your-api-key
AI_API_URL=https://api.openai.com/v1
```

### 5. Analytics System
Event-driven analytics collecting:
- Code analysis metrics
- Agent execution statistics
- Project activity tracking
- User behavior metrics

Records stored in PostgreSQL for historical analysis and reporting.

## Database Schema

### Users Table
```sql
CREATE TABLE users (
    id UUID PRIMARY KEY,
    email VARCHAR(255) UNIQUE,
    password_hash VARCHAR(255),
    first_name VARCHAR(100),
    last_name VARCHAR(100),
    created_at TIMESTAMP
);
```

### Projects Table
```sql
CREATE TABLE projects (
    id UUID PRIMARY KEY,
    user_id UUID REFERENCES users(id),
    name VARCHAR(255),
    description TEXT,
    language VARCHAR(50),
    repository_url VARCHAR(255),
    created_at TIMESTAMP
);
```

### Code Files, Analysis Tasks, Agent Tasks, and Analytics Tables
See `src/db/mod.rs` for complete schema definitions.

## API Usage Examples

### Authentication Flow
```bash
# 1. Register
curl -X POST http://localhost:8080/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email": "user@example.com",
    "password": "SecurePass123",
    "first_name": "John"
  }'

# Response includes access_token and refresh_token

# 2. Use token in subsequent requests
curl -H "Authorization: Bearer YOUR_TOKEN" \
  http://localhost:8080/projects

# 3. Refresh token when expired
curl -X POST http://localhost:8080/auth/refresh \
  -H "Content-Type: application/json" \
  -d '{"refresh_token": "YOUR_REFRESH_TOKEN"}'
```

### Project Workflow
```bash
# Create project
curl -X POST http://localhost:8080/projects \
  -H "Authorization: Bearer TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "My Project",
    "language": "rust"
  }'

# Get projects
curl -H "Authorization: Bearer TOKEN" \
  http://localhost:8080/projects

# Update project
curl -X PUT http://localhost:8080/projects/{id} \
  -H "Authorization: Bearer TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"name": "Updated Name"}'
```

### Code Analysis
```bash
# Optimize code
curl -X POST http://localhost:8080/analysis/optimize \
  -H "Authorization: Bearer TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "code": "fn main() { println!(\"hello\"); }",
    "language": "rust"
  }'

# Review code
curl -X POST http://localhost:8080/analysis/review \
  -H "Authorization: Bearer TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "code": "...",
    "language": "python"
  }'
```

### Multi-Agent Execution
```bash
# Spawn backend agent
curl -X POST http://localhost:8080/agents/backend \
  -H "Authorization: Bearer TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "project_id": "uuid",
    "task_description": "Create user authentication API"
  }'

# Get task status
curl -H "Authorization: Bearer TOKEN" \
  http://localhost:8080/agents/status/{task_id}
```

## Development Workflow

### Local Development
```bash
# Watch mode for auto-reload
cargo watch -x run

# Run with logging
RUST_LOG=debug cargo run

# Run specific tests
cargo test code_analysis::tests::
```

### Testing Strategy
- Unit tests in each module (`#[cfg(test)]`)
- Integration tests for API endpoints
- Mock database interactions
- Example tests in `services/` and `utils/`

### Code Quality
```bash
# Format code
cargo fmt

# Lint with Clippy
cargo clippy -- -D warnings

# Check for security vulnerabilities
cargo audit

# Generate documentation
cargo doc --open
```

## Deployment Guide

### Docker Deployment
```bash
# Build image
docker build -t compilex7-api:latest .

# Run container
docker run -p 8080:8080 \
  -e DATABASE_URL="postgresql://..." \
  -e JWT_SECRET="your-secret" \
  -e AI_API_KEY="your-key" \
  compilex7-api:latest
```

### Docker Compose Full Stack
```bash
# Start all services
docker-compose up -d

# View logs
docker-compose logs -f api

# Stop services
docker-compose down
```

### Production Checklist
- [ ] Update `JWT_SECRET` to strong random value
- [ ] Configure AI_API_KEY with actual OpenAI key
- [ ] Set ENVIRONMENT to "production"
- [ ] Configure DATABASE_URL for production database
- [ ] Enable HTTPS/TLS
- [ ] Set up monitoring and logging
- [ ] Configure rate limiting
- [ ] Set up automated backups
- [ ] Review CORS settings for frontend domain

## Performance Optimization

### Database Optimization
```rust
// Use connection pooling
let pool = PgPoolOptions::new()
    .max_connections(20)
    .connect(database_url)
    .await?;

// Use efficient queries with limits
sqlx::query("SELECT * FROM projects LIMIT 50")
```

### Async/Concurrency
```rust
// Non-blocking agent execution
tokio::spawn(async move {
    agent.execute(task, context).await
});

// Parallel request handling
axum automatically handles multiple concurrent requests
```

### Caching Strategy
- JWT tokens cached in client
- Database connection pooling
- Consider Redis for session management (future enhancement)

## Monitoring & Debugging

### Logging
```bash
# Set log levels
RUST_LOG=compilex7=debug,axum=info cargo run

# JSON logging for production
RUST_LOG_FORMAT=json cargo run
```

### Debugging
```rust
// Add debug prints
tracing::debug!("Debug info: {:?}", variable);
tracing::error!("Error occurred: {}", error);

// Use dbg! macro in tests
let result = dbg!(function_call());
```

### Health Checks
```bash
# Server health
curl http://localhost:8080/health

# Database connectivity
curl http://localhost:8080/analytics/dashboard
```

## Security Best Practices

1. **Environment Variables**
   - Never commit `.env` to git
   - Use `.env.example` as template
   - Rotate secrets regularly

2. **Database Security**
   - Use parameterized queries (SQLx does this automatically)
   - Restrict database user permissions
   - Enable SSL for connections

3. **API Security**
   - JWT validation on protected routes
   - CORS configuration for frontend domain
   - Rate limiting (implement with tower middleware)
   - Input validation on all endpoints

4. **Password Security**
   - Bcrypt hashing (cost 12)
   - Minimum password requirements enforced
   - Salted and peppered in storage

## Troubleshooting

### Connection Issues
```bash
# Test database connection
psql postgresql://user:pass@localhost:5432/compilex7

# Check if server is running
curl -v http://localhost:8080/health
```

### Build Errors
```bash
# Clean build
cargo clean
cargo build

# Update dependencies
cargo update
```

### Runtime Errors
```bash
# Enable full backtrace
RUST_BACKTRACE=full cargo run

# Check logs
docker-compose logs api
```

## Contributing

1. Fork repository
2. Create feature branch: `git checkout -b feature/name`
3. Make changes following code style
4. Add tests for new functionality
5. Run `cargo fmt` and `cargo clippy`
6. Commit: `git commit -am 'Add feature'`
7. Push: `git push origin feature/name`
8. Create Pull Request

## License

MIT License - See LICENSE file

## Support & Documentation

- GitHub Issues: Report bugs and feature requests
- Discussions: Ask questions and share ideas
- Documentation: Generated with `cargo doc --open`
