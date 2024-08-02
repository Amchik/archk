# `archk.ceheki.org/api`

> **Status**: Work In Progress
>
> - Need to plan possible actions.
> - Need to implement possible actions and spaces logs.
>
> For any questions email [@Amchik](https://github.com/Amchik) (email address in profile)
> or PM in [telegram](https://t.me/platfoxxx).

## Hacking

Use environment variables from `env.sh`:
```console
$ . env.sh
or...
$ DATABASE_URL=sqlite://archk.db cargo ...
```

Before running create `config.yml`:
```console
$ cp config{.example,}.yml
```

## Usage

See `config.example.yml`.

```console
$ docker run \
    --env CONFIG_PATH=/storage/config.yml \
    -v /opt/archk-dev:/storage:Z \
    -d \
    -p 8000:8000 \
    --name archk-api \
    archk
```

After start, admin user can be created thought empty invite (literally `"invite": ""`):

```http
PUT /api/v1/user HTTP/1.1
Content-Type: application/json

{
    "username": "admin",
    "password": "12345678",
    "invite": ""
}
```

## Documentation

API documentation in progress (sorry). Some models in `archk` crate documentated in `cargo doc [--open]`.

You can see list of endpoints in `archk-api/src/v1/mod.rs`. Authorization using bearer token, example:

```http
GET /api/v1/users HTTP/1.1
Authorization: Bearer acp_YySZC5EBAACVdPM2GvCnwQ
```
