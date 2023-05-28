import { ApolloClient, InMemoryCache } from "@apollo/client/core";
import { GraphQLWsLink } from "@apollo/client/link/subscriptions";
import { TransProvider } from "@mbarzda/solid-i18next";
import { ApolloProvider } from "@merged/solid-apollo";
import { createClient } from "graphql-ws";
import { type JSX } from "solid-js";
import { isServer } from "solid-js/web";
import { i18next } from "../i18n";
import { AccountProvider } from "./account";
import { HubPositionProvider } from "./hub";

export { useHubPosition, type HubPosition } from "./hub";
export { useAccount, type AccountCtx } from "./account";

const cache = new InMemoryCache();

const client = new ApolloClient(
  isServer
    ? { uri: "http://localhost:8080/api/graphql", cache }
    : {
        link: new GraphQLWsLink(
          createClient({
            url: "ws://localhost:3000/api/graphql/ws",
            lazyCloseTimeout: 30_000,
            keepAlive: 60_000,
          })
        ),
        cache,
      }
);

export function Contexts({ children }: { children: () => JSX.Element }) {
  return (
    <ApolloProvider client={client}>
      <TransProvider instance={i18next}>
        <HubPositionProvider>
          {() => <AccountProvider>{children}</AccountProvider>}
        </HubPositionProvider>
      </TransProvider>
    </ApolloProvider>
  );
}
