# Migration

A schema migration system inspired by [Cambria](https://www.inkandswitch.com/cambria/).

## Migrations

Migrations themselves look like this:

```yaml
ops:
  - add:
      name: title
      type: string
      default: ""
  - add:
      name: done
      type: bool
      default: false
```

You'd store this in a file named something like `task.v1.yaml`, where `task.v1` becomes the name of the schema. As with other migration tools, you should check the migrations into source control and avoid changing them once they're applied.

As you discover more about the system, you can add more fields or change existing ones:

```yaml
# The version this migration is based off of. For the initial migration for each
# kind of object you're using, you can omit this. For subsequent migrations, it's required.
base: task.v1

ops:
  - add:
      name: due
      type: int
      nullable: true
  - rename:
      from: done
      to: status
  - convert:
      name: status
      fromType: bool
      toType: string
      forward:
        true: complete
        false: unstarted
      reverse:
        complete: true
        unstarted: false
        inProgress: false
```

## The Graph

Once you define schemas, we can try to find a way to convert between them. For example, if we start at `task.v1` and try to go to `task.v2`, we need to apply the operations in `task.v2`. However, if we have a older version of the software that needs to read `task.v1`, that's also possible: all the operations can run in reverse!

This is also why the migrations are specified statically rather than in code: we need to be able to download migrations at runtime for older clients to be able to read data written in new formats. This increases flexibility of the entire system.

As a whole, you should be able to convert between any two versions of the schema (with some notable limitations listed below) because this crate creates and maintains a graph of migrations which it can walk to determine the correct migration to apply.

## Operations

### Add / Remove

You can add and remove fields from a schema by using the `add` and `remove` options. Here are the fields:

| Field | Notes |
|-|-|
| `name` | The name of the field (in an object) to add or remove |
| `type` | The type of the field for schema generation (accepted values TBD) |
| `nullable` | Whether or not the `type` includes `null` |
| `items` | If `type` is `list`, the type of items in the list. |
| `default` | A default to use when reading data that does not have this field set yet |

`add` and `remove` share the same schema because they mirror each other and may need to run in either direction, depending on the schema version you're reading.

`nullable` is true by default. If you set it to `false`, `default` is required.

### Rename

Change an object field's name.

| Field | Notes |
|-|-|
| `from` | The original name |
| `to` | The new name |

`rename` is its own reverse operation; we just switch the fields.

### Extract / Embed

`extract` overwrites an object with the value of one of its keys. `embed` converts a value into an embedded object.

| Field | Notes |
|-|-|
| `host` | The location of the object to extract from |
| `name` | The name of the key to extract |

An example is in order. Say you have this structure:

```json
{
  "user": {
    "id": 1234,
    "username": "toughguy1995"
  }
}
```

If you'd like `user` to be identified by `id` instead, here's how you'd write it:

```yaml
extract:
  host: user
  name: id
```

In this case, you'd end up with this.

```json
{
  "user": 1234
}

This brings us to the main limitation of this migration: all the other keys in the embedded object will be unavailable after an `extract` migration. So, if you ran that migration in reverse, you'd end up with:

```json5
{
  "user": {
    "id": 1234
    // no username!
  }
}
```

You can get around this in some cases by specifying `remove` operations for the other fields. So a migration like this:

```yaml
operations:
  - in:
      name: user
      ops:
        - remove:
            name: username
            type: string
  - extract:
      host: user
      name: id
```

Could possibly recover the fields (but not the field information) from the document:

```json
{
  "user": {
    "id": 1234,
    "username": null
  }
}
```

This behavior could hypothetically be extended to allow for custom code or remote retrieval. In practice, however, it's unlikely: keep your documents flat normally and you won't need this.

### Wrap / Head

`wrap` converts from a single value to a list. `head` replaces a list with the first item.

| Field | Notes |
|-|-|
| `name` | The name of the field to operate on |

This has a number of subtle things that can go wrong. See [the Cambria paper](https://www.inkandswitch.com/cambria/) for details.

### In

`in` allows you to operate at keys in embedded objects.

| Field | Notes |
|-|-|
| `name` | Field to navigate to |
| `ops` | Operations to apply at the given field |

`in` can be nested for deeper operations.

### Map

`map` allows you to perform operations on each element of a list.

| Field | Notes |
|-|-|
| `ops` | Operations to apply to each element |

Combine with `in` to navigate to a key containing the list.

### Convert

`convert` converts values from one type to another by following a mapping.

| Field | Notes |
|-|-|
| `name` | The field to convert |
| `fromType` | The type we're expecting to start with |
| `toType` | The type we're expecting to end up with |
| `forward` | A mapping from existing values to new values |
| `reverse` | A mapping from new values to existing values |
