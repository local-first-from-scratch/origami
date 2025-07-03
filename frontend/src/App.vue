<script setup lang="ts">
import { store } from 'store';
import { ref, type Ref } from 'vue';

const rows: Ref<any[]> = ref([]);

store < { test: any }>({ test: "bogus" }).then((s) => {
  console.log(s);
  s.insert("test", "1", {})
  s.list('test').then((r) => {
    rows.value = r;
  });
});
</script>

<template>
    <div class="content">
        <h1>Rsbuild with Vue</h1>
        <p>Start building amazing things with Rsbuild.</p>
        <template v-for="row in rows">
            <code>{{ row }}</code>
        </template>
    </div>
</template>

<style scoped>
.content {
    display: flex;
    min-height: 100vh;
    line-height: 1.1;
    text-align: center;
    flex-direction: column;
    justify-content: center;
}

.content h1 {
    font-size: 3.6rem;
    font-weight: 700;
}

.content p {
    font-size: 1.2rem;
    font-weight: 400;
    opacity: 0.5;
}
</style>
