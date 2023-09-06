import {
  ApolloClient,
  gql,
  InMemoryCache,
  type TypedDocumentNode,
} from "@apollo/client/core";
import { BatchHttpLink } from "@apollo/client/link/batch-http";
import { GraphQLWsLink } from "@apollo/client/link/subscriptions";
import { ApolloProvider } from "@merged/solid-apollo";
import { createClient as createWsClient } from "graphql-ws";
import { createEffect, type FlowProps, on, onCleanup } from "solid-js";
import { isServer } from "solid-js/web";
import { GQL_ACCOUNT, tokenExpiry, useAccount } from "./account";
import type { RefreshMutation, RefreshMutationVariables } from "~gen/graphql";

const GQL_REFRESH: TypedDocumentNode<
  RefreshMutation,
  RefreshMutationVariables
> = gql`
  ${GQL_ACCOUNT}
  mutation Refresh($token: String!) {
    refresh(refreshToken: $token) {
      ...AccountFields
    }
  }
`;

const refreshBuffer = 2 * 60_000;

export default function ClientProvider(props: FlowProps) {
  const { account, accessToken, refreshToken, login, logout } = useAccount();

  const cache = new InMemoryCache();

  let restartRequestedBeforeConnected = false;
  let gracefullyRestart = () => {
    restartRequestedBeforeConnected = true;
  };

  const client = new ApolloClient({
    link: isServer
      ? new BatchHttpLink({
          // TODO: use env var
          uri: "http://localhost:8080/api/graphql",
          batchInterval: 10,
          batchMax: 20,
        })
      : new GraphQLWsLink(
          createWsClient({
            url: `ws://${window.location.host}/api/graphql/ws`,
            lazyCloseTimeout: 30_000,
            keepAlive: 60_000,
            connectionParams: () => ({ token: accessToken() }),
            on: {
              connected: (sock) => {
                const socket = sock as WebSocket; // Known in browser env
                gracefullyRestart = () => {
                  if (socket.readyState === WebSocket.OPEN) {
                    socket.close(4205, "Client Restart");
                  }
                };

                // just in case you were eager to restart
                if (restartRequestedBeforeConnected) {
                  restartRequestedBeforeConnected = false;
                  gracefullyRestart();
                }
              },
            },
          }),
        ),
    cache,
    ssrMode: isServer,
  });

  createEffect(
    on(
      accessToken,
      (token, prevToken) => {
        if (token === prevToken) {
          return token;
        }

        // Defer so that queries can settle.
        setTimeout(gracefullyRestart);
        return token;
      },
      { defer: true },
    ),
    accessToken(),
  );

  let tokenRefreshId = 0;
  let failedAttempts = 0;
  const refreshTokens = (refresh: string) => {
    const cid = ++tokenRefreshId;
    client
      .mutate({
        mutation: GQL_REFRESH,
        variables: { token: refresh },
      })
      .then(({ data }) => {
        if (!data || cid !== tokenRefreshId) {
          return;
        }

        failedAttempts = 0;
        login(data.refresh);
        console.debug("Refreshed tokens");
      })
      .catch((err) => {
        console.error("Failed to refresh tokens", err);
        if (cid !== tokenRefreshId) {
          return;
        }

        failedAttempts++;
        if (failedAttempts > 3) {
          console.warn("Failed to refresh tokens too many times");
          logout();
        } else {
          setTimeout(() => refreshTokens(refresh), 5_000);
        }
      });
  };

  let id = account()?.id;
  createEffect(
    on([account, accessToken], ([account, access]) => {
      const currentId = account?.id;
      if (id !== currentId) {
        void client.resetStore();
        id = currentId;
      }

      const refresh = refreshToken();
      if (refresh) {
        if (!access) {
          refreshTokens(refresh);
          return;
        }

        const expiry = tokenExpiry(access);
        const now = Date.now();
        const refreshAt = expiry - now - refreshBuffer;

        const ref = setTimeout(() => refreshTokens(refresh), refreshAt);
        onCleanup(() => clearTimeout(ref));
      }
    }),
  );

  return <ApolloProvider client={client}>{props.children}</ApolloProvider>;
}
