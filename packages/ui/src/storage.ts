import {
  createSignal,
  getOwner,
  onMount,
  runWithOwner,
  type Signal,
} from "solid-js";
import { isServer } from "solid-js/web";
import {
  type StorageLike,
  syncSignal,
  useStorage,
  type UseStorageOptions,
} from "solidjs-use";

export function createClientInit(fn: () => void): void {
  if (!import.meta.env.START_SSR) {
    fn();
    return;
  }

  if (!isServer) {
    onMount(() => {
      const owner = getOwner();

      setTimeout(() => {
        runWithOwner(owner, fn);
      }, 50);
    });
  }
}

export function createStorage<T>(
  key: string,
  initialValue: T,
  options?: UseStorageOptions<T>,
  storage: StorageLike | undefined = globalThis.localStorage
): Signal<T> {
  if (isServer) {
    return createSignal(initialValue);
  }

  const [value, setValue] = createSignal(initialValue);
  const [store, setStore] = useStorage<T>(key, initialValue, storage, options);

  createClientInit(() => {
    setValue(() => store());
    syncSignal([store, setStore], [value, setValue]);
  });

  return [value, setValue];
}
