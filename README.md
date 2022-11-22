# Getting started

The easiest way to run the app is Docker Compose.  
https://docs.docker.com/compose/

```bash
docker compose up
```

# Using the API

Every response will contain the following JSON object:

| Field   | Type              | Optional | Description                                                |
|---------|-------------------|-------|------------------------------------------------------------|
| error   | boolean           | no    | Indicates whether the request has failed.                  |
| message | string            | yes   | The error description. Always present in case of an error. |
| result  | endpoint specific | yes   | The result of an operation.                                |

## POST /users

Create a user with a username and password. The username length must be between 3 and 32 (inclusive). The password length must be between 3 and 128 (inclusive).

The `result` field is always null.

### Examples

```bash
curl --http2 -k -X POST 'https://localhost:8443/users' \
  -H 'Content-Type: application/json' \
  -d '{"username":"hello","password":"world"}'
```

**201 Created**

```json
{
  "error": false
}
```

**409 Conflict**

```json
{
  "error": true,
  "message": "Username already exists"
}
```

# Running endpoint tests

1. Start the PostgreSQL instance

```bash
docker run --rm -e POSTGRES_PASSWORD=postgres -p 5432:5432 postgres:alpine
```

2. Apply migrations

```bash
docker run --rm --net=host -v $PWD/migrations/:/flyway/sql flyway/flyway:latest-alpine -url=jdbc:postgresql://localhost/postgres -user=postgres -password=postgres migrate
```

3. Run the application

```bash
RUST_LOG=trace cargo run
```

4. Run the tests

```bash
cargo test -- --nocapture
```
