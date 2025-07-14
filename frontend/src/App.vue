<script setup lang="ts">
import { store } from 'store';
import { type Ref, ref } from 'vue';

type Row = {
  test: string;
};

const rows: Ref<Row[]> = ref([]);

const migrations = [
  {
    schema: 'test',
    version: 1,
    ops: [
      {
        add: {
          name: 'test',
          type: 'string',
          nullable: true,
        },
      },
    ],
  },
];

store<{ test: Row }>({ test: 1 }, migrations).then(async (s) => {
  console.log(s);
  console.log(await s.insert('test', { test: 'test' }));
  s.list('test').then((r) => {
    console.log(r)
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
