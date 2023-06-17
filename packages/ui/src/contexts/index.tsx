import { type FlowProps } from "solid-js";
import AccountProvider from "./account";
import ClientProvider from "./client";
import HubPositionProvider from "./hub";
import { baseTrans, translations, TransProvider } from "~/i18n";

export { useHubPosition, type HubPosition } from "./hub";
export { GQL_ACCOUNT, useAccount, type AccountCtx } from "./account";

export function Contexts(props: FlowProps) {
  return (
    <TransProvider base={baseTrans} translations={translations}>
      <HubPositionProvider>
        <AccountProvider>
          <ClientProvider>{props.children}</ClientProvider>
        </AccountProvider>
      </HubPositionProvider>
    </TransProvider>
  );
}
