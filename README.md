# Getting started

The easiest way to run the app is Docker Compose.  
https://docs.docker.com/compose/

```bash
docker compose up
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

# Using the API

### Response format

Every response will contain the following JSON object:

| Field   | Type              | Nullable | Description                                                |
|---------|-------------------|----------|------------------------------------------------------------|
| error   | boolean           | no       | Indicates whether the request has failed.                  |
| message | string            | yes      | The error description. Always present in case of an error. |
| result  | endpoint specific | yes      | The result of an operation.                                |

### Authorization

Most endpoints require you to set an `Authorization` header containing the authorization token. You can obtain it via the [/users/@me/sessions](#post-usersmesessions) endpoint.

## Endpoints

### POST /users

Create a user.

#### Request Payload:

| Field    | Type              | Required | Description                                                                 |
|----------|-------------------|----------|-----------------------------------------------------------------------------|
| username | string            | yes      | The username. Its length must be between 3 and 32 characters (inclusive).  |
| password | string            | yes     | The password. Its length must be between 3 and 128 characters (inclusive). |

On success, the `result` field will contain a [User](#user) object.

#### Examples

```bash
curl -k -X POST 'https://localhost:8443/users' \
  -H 'Content-Type: application/json' \
  -d '{"username":"hello","password":"world"}'
```

**201 Created**

```json
{
  "error": false,
  "result": {
    "id": 1,
    "username": "hello"
  }
}
```

**409 Conflict**

```json
{
  "error": true,
  "message": "Username already exists"
}
```

### POST /users/@me/sessions

Create a session.

#### Request Payload:

| Field    | Type              | Required | Description                                                                 |
|----------|-------------------|----------|-----------------------------------------------------------------------------|
| username | string            | yes      | The username. Its length must be between 3 and 32 characters (inclusive).  |
| password | string            | yes     | The password. Its length must be between 3 and 128 characters (inclusive). |

On success, the `result` field will contain a `string` authorization token.

#### Examples

```bash
curl -k -X POST 'https://localhost:8443/users/@me/sessions' \
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

### POST /users/@me/tweets

Create a tweet.

#### Request Payload:

| Field    | Type              | Required | Description                                                                       |
|----------|-------------------|----------|-----------------------------------------------------------------------------------|
| text     | string            | yes      | The tweet content. Its length must be between 1 and 4096 characters (inclusive). |

On success, the `result` field will contain a [Tweet](#tweet) object.

**Requires authorization*

#### Examples

```bash
curl -k -X POST 'https://localhost:8443/users/@me/tweets' \
  -H 'Content-Type: application/json' \
  -H 'Authorization: your_token' \
  -d '{"text":"a"}'
```

**201 Created**

```json
{
  "error": false,
  "result": {
    "id": 1,
    "text": "a",
    "like_count": 0,
    "time_created": 1669185715
  }
}
```

**401 Unauthorized**

```json
{
  "error": true,
  "message": "Unauthorized"
}
```

### GET /users/@me/tweets

Get tweets.

#### Optional Query Parameters:

| Name   | Type   | Description                                                                  |
|--------|--------|------------------------------------------------------------------------------|
| limit  | number | The maximum number of tweets to return. The default and maximum value is 50. |
| offset | number | The number of tweets to skip. The default value is 0.                        |

On success, the `result` field will contain an array of [Tweet](#tweet) objects.

**Requires authorization*

#### Examples

```bash
curl -k -X GET 'https://localhost:8443/users/@me/tweets?offset=0&limit=50' \
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
      "like_count": 10,
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

### PATCH /users/@me/tweets/{tweet.id}

Edit a tweet.

#### Request Payload:

| Field | Type   | Required | Description                                                                       |
|-------|--------|----------|-----------------------------------------------------------------------------------|
| text  | string | yes      | The tweet content. Its length must be between 1 and 4096 characters (inclusive). |

On success, the `result` field will contain a [Tweet](#tweet) object.

**Requires authorization*

#### Examples

```bash
curl -k -X PATCH 'https://localhost:8443/users/@me/tweets/1' \
  -H 'Content-Type: application/json' \
  -H 'Authorization: your_token' \
  -d '{"text":"Hello, World!"}'
```

**200 OK**

```json
{
  "error": false,
  "result": {
    "id": 1,
    "text": "Hello, World!",
    "like_count": 10,
    "time_created": 1669185715
  }
}
```

**401 Unauthorized**

```json
{
  "error": true,
  "message": "Unauthorized"
}
```

### DELETE /users/@me/tweets/{tweet.id}

Delete a tweet.

The `result` field is always `null`.

**Requires authorization*

#### Examples

```bash
curl -k -X DELETE 'https://localhost:8443/users/@me/tweets/1' \
  -H 'Authorization: your_token'
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

### POST /users/@me/liked_tweets

Like a tweet.

#### Request Payload:

| Field    | Type   | Required | Description                                                                 |
|----------|--------|----------|-----------------------------------------------------------------------------|
| tweet_id | number | yes      | The tweet ID.                                                               |

The `result` field is always `null`.

**Requires authorization*

#### Examples

```bash
curl -k -X POST 'https://localhost:8443/users/@me/liked_tweets' \
  -H 'Content-Type: application/json' \
  -H 'Authorization: your_token' \
  -d '{"tweet_id":1}'
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
  "message": "Tweet already liked"
}
```

### DELETE /users/@me/liked_tweets

Unlike a tweet.

#### Request Payload:

| Field    | Type   | Required | Description                                                                 |
|----------|--------|----------|-----------------------------------------------------------------------------|
| tweet_id | number | yes      | The tweet ID.                                                               |

The `result` field is always `null`.

**Requires authorization*

#### Examples

```bash
curl -k -X DELETE 'https://localhost:8443/users/@me/liked_tweets' \
  -H 'Content-Type: application/json' \
  -H 'Authorization: your_token' \
  -d '{"tweet_id":1}'
```

**200 OK**

```json
{
  "error": false
}
```

**404 Not Found**

```json
{
  "error": true,
  "message": "Not Found"
}
```

## Objects

### User

#### Structure

| Field    | Type   | Nullable | Description   |
|----------|--------|----------|---------------|
| id       | number | no       | The user ID.  |
| username | string | no       | The username. |

#### Example

```json
{
  "id": 1,
  "username": "hello"
}
```

### Tweet

#### Structure

| Field        | Type   | Nullable | Description                               |
|--------------|--------|----------|-------------------------------------------|
| id           | number | no       | The tweet ID.                             |
| text         | string | no       | The tweet content.                        |
| like_count   | number | no       | The number of likes.                      |
| time_created | number | no       | The UNIX time when the tweet was created. |

#### Example

```json
{
  "id": 1,
  "text": "Hello, World!",
  "like_count": 10,
  "time_created": 1669185715
}
```