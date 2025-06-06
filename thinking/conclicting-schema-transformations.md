# Schemas

## A

```
- add:
    name: a
    type: string
    default: "hello"
```

## B

```
- rename:
    from: a
    to: b
```

# Problem

Say these things happen:

1. We write to schema a, and therefore have a patch `{"op": "add", "path": "/a", "value": "hello"}`
2. We read at schema b, and overwrite the new value `{"op": "add", "path": "/b", "value": "world"}`

When we read at schema A again, what do we get? We start off with the following:

```json
[
  {"op": "add", "path": "/a", "value": "hello", "schema": "a"},
  {"op": "add", "path": "/b", "value": "world", "schema": "b"}
]
```

But to read at A or B again, we need to apply the `B` schema in either direction. If we're reading at `A` that gives us:

```json
[
  {"op": "add", "path": "/a", "value": "hello", "schema": "a"},
  {"op": "add", "path": "/a", "value": "world", "schema": "a"}
]
```

This isn't a problem, and "world" wins. If we read at `B` instead:

```json
[
  {"op": "add", "path": "/b", "value": "hello", "schema": "b"},
  {"op": "add", "path": "/b", "value": "world", "schema": "b"}
]
```

Still not a problem, because the second operation wins.

The problem I guess I'm worried about here is that the final value is determined by the ordering of the operations. If that's stable, I guess we're fine. But if it's not stable, or merging that way gives an incoherent answer, I want to find a way around it.

(n.b. I'm not actually worried about getting different answers for different schemas; that's expected. Why have a schema transformation layer at all if you're not OK with that? It's the point!)

One way I can think of: attach the write timestamp as well as the schema. (This would actually live a wrapper instead of the patch. Pretend, OK?)

```json
[
  {"op": "add", "path": "/a", "value": "hello", "schema": "a", "ts": "2@a"},
  {"op": "add", "path": "/b", "value": "world", "schema": "b", "ts": "1@b"}
]
```

Lexicographically, `2@a` wins over `1@b`, so we'd resolve this to `"hello"` instead of `"world"`.

We could implement this in several ways:

1. Sort the ops by `ts`. We'd still apply all the ops in the patch, but the largest `ts` would win.
2. Remove earlier ops before applying later ops

---

I'm also concerned about how this handles multiple values.
