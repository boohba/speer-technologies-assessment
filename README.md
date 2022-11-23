# Getting started

The easiest way to run the app is Docker Compose.  
https://docs.docker.com/compose/

```bash
docker compose up
```

# Using the API

Every response will contain the following JSON object:

| Field   | Type              | Required | Description                                                |
|---------|-------------------|----------|------------------------------------------------------------|
| error   | boolean           | res      | Indicates whether the request has failed.                  |
| message | string            | no       | The error description. Always present in case of an error. |
| result  | endpoint specific | no       | The result of an operation.                                |

Most endpoints will require you to send an `Authorization` header containing the authorization token. You can obtain it via the `POST /sessions` endpoint.

## POST /users

| Field    | Type              | Required | Description                                                                 |
|----------|-------------------|----------|-----------------------------------------------------------------------------|
| username | string            | yes      | The username. It's length must be between 3 and 32 characters (inclusive).  |
| password | string            | yes     | The password. It's length must be between 3 and 128 characters (inclusive). |

On success, the `result` field will contain a `number` user ID.

### Examples

```bash
curl --http2 -k -X POST 'https://localhost:8443/users' \
  -H 'Content-Type: application/json' \
  -d '{"username":"hello","password":"world"}'
```

**201 Created**

```json
{
  "error": false,
  "result": 1
}
```

**409 Conflict**

```json
{
  "error": true,
  "message": "Username already exists"
}
```

## POST /sessions

| Field    | Type              | Required | Description                                                                 |
|----------|-------------------|----------|-----------------------------------------------------------------------------|
| username | string            | yes      | The username. It's length must be between 3 and 32 characters (inclusive).  |
| password | string            | yes     | The password. It's length must be between 3 and 128 characters (inclusive). |

On success, the `result` field will contain a `string` authorization token.

### Examples

```bash
curl --http2 -k -X POST 'https://localhost:8443/sessions' \
  -H 'Content-Type: application/json' \
  -d '{"username":"hello","password":"world"}'
```

**201 Created**

```json
{
  "error": false,
  "result": "..."
}
```

**401 Unauthorized**

```json
{
  "error": true,
  "message": "Unauthorized"
}
```

## POST /tweets

**Requires authorization*

| Field    | Type              | Required | Description                                                                       |
|----------|-------------------|----------|-----------------------------------------------------------------------------------|
| text     | string            | yes      | The tweet content. It's length must be between 1 and 4096 characters (inclusive). |

On success, the `result` field will contain a `number` tweet ID.

### Examples

```bash
curl --http2 -k -X POST 'https://localhost:8443/tweets' \
  -H 'Content-Type: application/json' \
  -H 'Authorization: your_token' \
  -d '{"text":"a"}'
```

**201 Created**

```json
{
  "error": false,
  "result": 1
}
```

**401 Unauthorized**

```json
{
  "error": true,
  "message": "Unauthorized"
}
```

## GET /tweets

**Requires authorization*

#### Optional query parameters

| Name   | Type   | Description                                                                  |
|--------|--------|------------------------------------------------------------------------------|
| limit  | number | The maximum number of tweets to return. The default and maximum value is 50. |
| offset | number | The number of tweets to skip. The default value is 0.                        |

On success, the `result` field will contain an array of tweets.

### Examples

```bash
curl --http2 -k -X GET 'https://localhost:8443/tweets?offset=0&limit=50' \
  -H 'Authorization: your_token'
```

**200 OK**

```json
{
  "error": false,
  "result": [
    {
      "id": 1,
      "text": "Hello, World!",
      "time_created": 1669185715
    }
  ]
}
```

**401 Unauthorized**

```json
{
  "error": true,
  "message": "Unauthorized"
}
```

## PATCH /tweets

**Requires authorization*

| Field | Type   | Required | Description                                                                       |
|-------|--------|----------|-----------------------------------------------------------------------------------|
| id    | number | yes      | The tweet ID.                                                                     |
| text  | string | yes      | The tweet content. It's length must be between 1 and 4096 characters (inclusive). |

The `result` field is always `null`.

### Examples

```bash
curl --http2 -k -X PATCH 'https://localhost:8443/tweets' \
  -H 'Content-Type: application/json' \
  -H 'Authorization: your_token' \
  -d '{"id":1,"text":"Hello, World!"}'
```

**200 OK**

```json
{
  "error": false
}
```

**401 Unauthorized**

```json
{
  "error": true,
  "message": "Unauthorized"
}
```

## DELETE /tweets

**Requires authorization*

| Field | Type   | Required | Description                                                                       |
|-------|--------|----------|-----------------------------------------------------------------------------------|
| id    | number | yes      | The tweet ID.                                                                     |

The `result` field is always `null`.

### Examples

```bash
curl --http2 -k -X DELETE 'https://localhost:8443/tweets' \
  -H 'Content-Type: application/json' \
  -H 'Authorization: your_token' \
  -d '{"id":1}'
```

**200 OK**

```json
{
  "error": false
}
```

**401 Unauthorized**

```json
{
  "error": true,
  "message": "Unauthorized"
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
