import { subscribe, unsubscribe } from "store";
import { type Ref, onUnmounted, ref } from "vue";

export function useDoc<T>(id: string): Ref<T | null> {
	const doc: Ref<T | null> = ref(null);

	const subscriptionId = subscribe(id, (newDoc: T) => {
		console.log("new doc", newDoc);
		doc.value = newDoc;
	});

	onUnmounted(() => {
		unsubscribe(subscriptionId);
	});

	return doc;
}
