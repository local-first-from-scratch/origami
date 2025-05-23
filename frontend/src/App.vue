<script setup lang="ts">
import * as store from "store";
import { useDoc } from "./doc";

console.log(store.dry_run());

const doc = useDoc("12345");
</script>

<template>
  {{doc}}
</template>
