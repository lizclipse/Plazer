import type { JSX } from "solid-js";
import type { Error as BackendError } from "~gen/backend";

export type ErrorCode = BackendError["code"];
export type ErrorCodeI18n = ErrorCode | "Unknown";
export type ErrorI18n = Record<ErrorCodeI18n, () => JSX.Element>;

export interface TFormData<T extends { [K in keyof T]?: File | string | null }>
  extends FormData {
  get<K extends keyof T>(name: K): ToNull<T[K]>;
  get(name: string): unknown;
}

export type FormFields<T extends { [K in keyof T]?: File | string | null }> = {
  [K in keyof T]-?: K;
};

type ToNull<T> = T extends undefined ? Exclude<T, undefined> | null : T;
