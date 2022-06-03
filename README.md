# RestKV

[![Build](https://github.com/ahxxm/restkv/actions/workflows/rust.yml/badge.svg)](https://github.com/ahxxm/restkv/actions/workflows/rust.yml)

A simple, unreliable Key-Value store for serverless applications.

## API

Try [https://kv.ahxxm.com](https://kv.ahxxm.com).

|Method|Path|Result|Note|
|----|----|----|----|
|POST|/new|token||
|POST|/{token}/{key}|key|value in HTTP body|
|GET|/{token}/{key}|value||
|GET|/keys/{token}|list of keys under the token||

*Value <=4KB, will overwrite.*

POST curl equivalent: `curl -X POST -d 'value' https://kv.ahxxm.com/token/key`.

## Deploy Your Own

RestKV is now designed to be behind a reverse proxy, it listens at `0.0.0.0:8080`(hard-coded).

Check `docker-compose.yml`.
