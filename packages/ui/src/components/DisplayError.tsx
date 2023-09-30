import { ApolloError } from "@apollo/client/core";
import { Show } from "solid-js";
import styles from "./DisplayError.module.scss";
import { Trans } from "~/i18n";
import type { ErrorI18n } from "~/types";
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
    return { message, code: extensions?.code as BackendError["code"] };
  } else if (err instanceof Error) {
    return unknownError;
  } else if (typeof err === "string") {
    return { message: err, code: "Unknown" };
  } else {
    return unknownError;
  }
}

export interface DisplayErrorProps {
  readonly error: () => unknown;
  readonly keepSpacing?: boolean;
  readonly overrides?: () => Partial<ErrorI18n>;
}

export default function DisplayError(props: DisplayErrorProps) {
  return (
    <Show
      when={parseError(props.error())}
      fallback={
        props.keepSpacing ? (
          <p classList={{ [styles.error]: true, [styles.hidden]: true }}>
            &nbsp;
          </p>
        ) : undefined
      }
    >
      {(err) => (
        <p class={styles.error} role="alert">
          <Show
            when={props.overrides?.()[err().code]?.()}
            fallback={<Trans>{(t) => t.core.errors[err().code]()}</Trans>}
          >
            {(override) => override()}
          </Show>
        </p>
      )}
    </Show>
  );
}
