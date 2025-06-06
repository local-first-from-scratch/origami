You should be able to get the latest schema:

```console
$ migrate --migrations-dir tests/migrations schema user.v3
{
  "properties": {
    "email": {
      "type": "string"
    },
    "status": {
      "enum": [
        "active",
        "inactive",
        "pending"
      ]
    },
    "username": {
      "type": "string"
    }
  }
}

```

You should also be able to get previous schema:

```console
$ migrate --migrations-dir tests/migrations schema user.v2
{
  "properties": {
    "email": {
      "type": "string"
    },
    "status": {
      "type": "boolean"
    },
    "username": {
      "type": "string"
    }
  }
}

```

```console
$ migrate --migrations-dir tests/migrations schema user.v1
{
  "properties": {
    "email": {
      "type": "string"
    },
    "handle": {
      "type": "string"
    }
  }
}

```
