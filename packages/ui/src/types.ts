import type { JSX } from "solid-js";
import type { Error as BackendError } from "~gen/backend";

export type ErrorCode = BackendError["code"];
export type ErrorCodeI18n = ErrorCode | "Unknown";
export type ErrorI18n = Record<ErrorCodeI18n, () => JSX.Element>;
