/* eslint-disable @typescript-eslint/no-explicit-any */
/* eslint-disable @typescript-eslint/no-unsafe-return */
/* eslint-disable @typescript-eslint/no-unsafe-argument */
/* eslint-disable @typescript-eslint/no-explicit-any */
import {
  type Accessor,
  children,
  createContext,
  createEffect,
  createSignal,
  type JSX,
  on,
  type ParentProps,
  useContext,
} from "solid-js";
import { useNavigatorLanguage } from "solidjs-use";

export type TransFn<A extends [...any]> = (...args: A) => JSX.Element;

export type TransTree<T> = {
  [K in keyof T]: TransTreeProp<T[K]>;
};

type TransTreeProp<T> = TransFn<any> | TransTree<T>;

export type DeepPartial<T> = {
  [K in keyof T]?: DeepPartial<T[K]>;
};

export function createTransProxy<T extends TransTree<T>>(
  base: T,
  tree: DeepPartial<T>
): T {
  return new Proxy(base, {
    get(target, p) {
      const prop = p as keyof T;
      const baseValue = Reflect.get(target, prop);
      const treeValue = Reflect.get(tree, prop);

      if (typeof baseValue === "function") {
        if (typeof treeValue === "function") {
          return (...args: any[]) => {
            const result = baseValue(...args);
            return treeValue(...args) ?? result;
          };
        }

        return (...args: any[]) => {
          console.warn("Missing translation for key :", prop, treeValue);
          return baseValue(...args);
        };
      }

      return createTransProxy(baseValue, treeValue ?? {});
    },
  });
}

export type Translations = DeepPartial<Record<string, TransTree<any>>>;

const TransContext = createContext<Accessor<TransTree<any>>>();

export interface TransProviderProps<T> {
  base: TransTree<T>;
  translations: Translations;
}

export function TransProvider<T>(props: ParentProps<TransProviderProps<T>>) {
  const { language } = useNavigatorLanguage();
  const getTrans = (lang?: string | undefined) =>
    lang
      ? createTransProxy(props.base, props.translations[lang] ?? {})
      : props.base;

  const [current, setCurrent] = createSignal<TransTree<any>>(
    getTrans(language())
  );

  createEffect(
    on(
      language,
      () => {
        setCurrent(getTrans(language()));
      },
      { defer: true }
    )
  );

  return (
    <TransContext.Provider value={current}>
      {props.children}
    </TransContext.Provider>
  );
}

export function useTrans<T extends TransTree<T>>(): [Accessor<T>] {
  const current = useContext(TransContext);
  if (!current) {
    throw new Error("Missing TransProvider");
  }

  return [current as Accessor<T>];
}

export function Trans<T extends TransTree<T>>(props: {
  children: (t: T) => JSX.Element;
}): JSX.Element {
  const [t] = useTrans<T>();
  return <>{children(() => props.children(t()))}</>;
}
