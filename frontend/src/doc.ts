import { type Handle, Hub } from "store";
import { type Ref, onUnmounted, shallowRef, triggerRef } from "vue";

export const hub = new Hub();

export function watch(handle: Handle): Ref<Handle> {
  const wrapped: Ref<Handle> = shallowRef(handle);

  const subscriptionId = handle.subscribe(() => {
    console.log("triggered handle subscription");
    triggerRef(wrapped);
  });

  onUnmounted(() => {
    hub.unsubscribe(subscriptionId);
  });

  return wrapped;
}
