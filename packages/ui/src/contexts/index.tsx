import { TransProvider } from "@mbarzda/solid-i18next";
import { type JSX } from "solid-js";
import { i18next } from "../i18n";
import AccountProvider from "./account";
import ClientProvider from "./client";
import HubPositionProvider from "./hub";

export { useHubPosition, type HubPosition } from "./hub";
export { GQL_ACCOUNT, useAccount, type AccountCtx } from "./account";

export interface ContextsProps {
  readonly children: () => JSX.Element;
}

export function Contexts({ children }: ContextsProps) {
  return (
    <TransProvider instance={i18next}>
      <HubPositionProvider>
        {() => (
          <AccountProvider>
            {() => <ClientProvider>{children}</ClientProvider>}
          </AccountProvider>
        )}
      </HubPositionProvider>
    </TransProvider>
  );
}
