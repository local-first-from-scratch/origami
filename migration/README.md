# Migration

A schema migration system inspired by [Cambria](https://www.inkandswitch.com/cambria/).

## Migrations

Migrations themselves look like this:

```yaml
operations:
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

operations:
  - add:
      name: due
      type: int
      nullable: true
  - rename:
      from: done
      to: status
  - convert:
      name: status
      mapping: # bidirectional mapping
        - [true, complete]
        - [false, unstarted]
```
