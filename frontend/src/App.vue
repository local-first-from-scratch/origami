<script setup lang="ts">
import * as store from "store";
import { hub, watch } from "./doc";

console.log(store.dry_run());

const doc = watch(hub.create(store.RootKind.Map));

setInterval(() => {
  doc.value.set("x", new Date().getTime());
}, 1000);
</script>

<template>
  Doc: {{doc}}

  Current value: {{doc.current()}}
</template>
