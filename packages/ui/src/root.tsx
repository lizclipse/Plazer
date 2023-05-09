// @refresh reload
import { ApolloClient, InMemoryCache } from "@apollo/client/core";
import { GraphQLWsLink } from "@apollo/client/link/subscriptions";
import { ApolloProvider } from "@merged/solid-apollo";
import { Routes } from "@solidjs/router";
import { createClient } from "graphql-ws";
import { Suspense } from "solid-js";
import { isServer } from "solid-js/web";
import {
  Body,
  FileRoutes,
  Head,
  Html,
  Meta,
  Scripts,
  Title,
} from "solid-start";
import { ErrorBoundary } from "solid-start/error-boundary";

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

export default function Root() {
  return (
    <Html lang="en">
      <Head>
        <Title>Commonwealthity</Title>
        <Meta charset="utf-8" />
        <Meta name="viewport" content="width=device-width, initial-scale=1" />
      </Head>
      <Body>
        <ApolloProvider client={client}>
          <Suspense>
            <ErrorBoundary>
              <main>
                <Routes>
                  <FileRoutes />
                </Routes>
              </main>
            </ErrorBoundary>
          </Suspense>
        </ApolloProvider>
        <Scripts />
      </Body>
    </Html>
  );
}
