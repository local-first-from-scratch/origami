import { defineConfig } from '@rsbuild/core';
import { pluginVue } from '@rsbuild/plugin-vue';
import { pluginWasmPack } from 'rsbuild-plugin-wasmpack';

export default defineConfig({
  plugins: [
    pluginVue(),
    pluginWasmPack({
      crates: [
        {
          path: '../store', // The path to your Rust crate
          target: 'bundler', // The target environment (e.g., 'web', 'nodejs')
        },
      ],
    }),
  ],
});
