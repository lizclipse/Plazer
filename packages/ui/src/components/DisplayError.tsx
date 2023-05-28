import { ApolloError } from "@apollo/client/core";
import { Trans } from "@mbarzda/solid-i18next";
import { Show } from "solid-js";
import styles from "./DisplayError.module.scss";
import type { Error as BackendError } from "~gen/backend";

export interface Err {
  readonly message: string;
  readonly code: BackendError["code"] | "Unknown";
}

const unknownError: Err = { message: "Unknown error", code: "Unknown" };

export function parseError(err: unknown): Err | undefined {
  if (!err) {
    return undefined;
  } else if (err instanceof ApolloError) {
    const gqlError = err.graphQLErrors[0];
    if (!gqlError) {
      return unknownError;
    }
    const { message, extensions } = gqlError;
    return { message, code: extensions.code as BackendError["code"] };
  } else if (err instanceof Error) {
    return unknownError;
  } else if (typeof err === "string") {
    return { message: err, code: "Unknown" };
  } else {
    return unknownError;
  }
}

export function DisplayError({ err }: { err: () => unknown }) {
  return (
    <Show when={parseError(err())}>
      {(error) => (
        <p class={styles.error} role="alert">
          <Trans
            key={`errors.${error().code}`}
            options={{ defaultValue: error().message }}
          />
        </p>
      )}
    </Show>
  );
}
