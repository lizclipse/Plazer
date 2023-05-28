// @refresh reload
import { Routes } from "@solidjs/router";
import { Suspense } from "solid-js";
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
import Hub from "./components/hub/Hub";
import { Contexts, useHubPosition } from "./contexts";

export function Root() {
  const { x, y } = useHubPosition();

  return (
    <Html lang="en">
      <Head>
        <Title>Plazer</Title>
        <Meta charset="utf-8" />
        <Meta name="viewport" content="width=device-width, initial-scale=1" />
      </Head>
      <Body
        style={{
          ["--hub-button-x"]: `${x()}px`,
          ["--hub-button-y"]: `${y()}px`,
        }}
      >
        <Suspense>
          <ErrorBoundary>
            <Hub />
            <main>
              <Routes>
                <FileRoutes />
              </Routes>
            </main>
          </ErrorBoundary>
        </Suspense>
        <Scripts />
      </Body>
    </Html>
  );
}

export default () => (
  <Contexts>
    <Root />
  </Contexts>
);
