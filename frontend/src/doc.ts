import { Hub } from "store";
import { type Ref, onUnmounted, ref } from "vue";

const hub = new Hub();

export function useDoc<T>(id: string): Ref<T | null> {
  const doc: Ref<T | null> = ref(null);

  const subscriptionId = hub.subscribe(id, (newDoc: T) => {
    console.log("new doc", newDoc);
    doc.value = newDoc;
  });

  onUnmounted(() => {
    hub.unsubscribe(subscriptionId);
  });

  return doc;
}
