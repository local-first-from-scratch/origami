You should be able to see the right schema for any migration. That way we get backward and fowards compatibility.

```console
$ migrate schema user.v1 --dir tests/migrations
{
  "properties": {
    "handle": {
      "nullable": true,
      "type": "string"
    }
  }
}
```

```console
$ migrate schema user.v2 --dir tests/migrations
{
  "properties": {
    "username": {
      "nullable": true,
      "type": "string"
    }
  }
}
```

```console
$ migrate schema user.v3 --dir tests/migrations
{
  "properties": {}
}
```
