import { ApolloClient, InMemoryCache } from "@apollo/client/core";
import { GraphQLWsLink } from "@apollo/client/link/subscriptions";
import { TransProvider } from "@mbarzda/solid-i18next";
import { ApolloProvider } from "@merged/solid-apollo";
import { createClient } from "graphql-ws";
import {
  type Accessor,
  createContext,
  type JSX,
  type Setter,
  useContext,
} from "solid-js";
import { isServer } from "solid-js/web";
import { useLocalStorage, useWindowSize } from "solidjs-use";
import { i18next } from "./i18n";

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

export interface HubPosition {
  x: Accessor<number>;
  setX: Setter<number>;
  y: Accessor<number>;
  setY: Setter<number>;
}

const HubPositionContext = createContext<HubPosition>();

export function useHubPosition(): HubPosition {
  // Will always be set by Contexts
  // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
  return useContext(HubPositionContext)!;
}

export function Contexts({ children }: { children: () => JSX.Element }) {
  const { width: screenWidth, height: screenHeight } = useWindowSize();
  const [x, setX] = useLocalStorage("hub-button-x", screenWidth() / 2);
  const [y, setY] = useLocalStorage("hub-button-y", screenHeight());

  return (
    <ApolloProvider client={client}>
      <TransProvider instance={i18next}>
        <HubPositionContext.Provider value={{ x, setX, y, setY }}>
          {children()}
        </HubPositionContext.Provider>
      </TransProvider>
    </ApolloProvider>
  );
}
