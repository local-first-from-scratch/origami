/// <reference types="@rsbuild/core/types" />

declare module "*.vue" {
	import type { DefineComponent } from "vue";

	// biome-ignore lint/complexity/noBannedTypes: todo
	// biome-ignore lint/suspicious/noExplicitAny: todo
	const component: DefineComponent<{}, {}, any>;
	export default component;
}
