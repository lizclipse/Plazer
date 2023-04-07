// @refresh reload
import { ApolloClient, InMemoryCache } from "@apollo/client/core";
import { ApolloProvider } from "@merged/solid-apollo";
import { Routes } from "@solidjs/router";
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

const client = new ApolloClient({
  uri: `${isServer ? "http://localhost:8080" : ""}/api/graphql`,
  cache: new InMemoryCache(),
});

export default function Root() {
  return (
    <Html lang="en">
      <Head>
        <Title>SolidStart - With Vitest</Title>
        <Meta charset="utf-8" />
        <Meta name="viewport" content="width=device-width, initial-scale=1" />
      </Head>
      <Body>
        <ApolloProvider client={client}>
          <Suspense>
            <ErrorBoundary>
              <Routes>
                <FileRoutes />
              </Routes>
            </ErrorBoundary>
          </Suspense>
        </ApolloProvider>
        <Scripts />
      </Body>
    </Html>
  );
}
