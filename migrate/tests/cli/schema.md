You should be able to see the right schema for any migration. That way we get backward and forwards compatibility.

```console
$ migrate schema user 1
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
$ migrate schema user 2
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
$ migrate schema user 3
{
  "properties": {}
}
```
