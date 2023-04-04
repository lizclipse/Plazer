import fs from 'node:fs';

const ADDED_STR = "// @ts-nocheck\n\n";
const FILES = [
  "../../node_modules/.pnpm/solid-start@0.2.24_@solidjs+meta@0.28.4_@solidjs+router@0.8.2_solid-js@1.7.2_solid-start-node_rc6adjatyfg7tlpbwy4hzftybm/node_modules/solid-start/api/index.ts",
  "../../node_modules/.pnpm/solid-start@0.2.24_@solidjs+meta@0.28.4_@solidjs+router@0.8.2_solid-js@1.7.2_solid-start-node_rc6adjatyfg7tlpbwy4hzftybm/node_modules/solid-start/api/internalFetch.ts",
  "../../node_modules/.pnpm/solid-start@0.2.24_@solidjs+meta@0.28.4_@solidjs+router@0.8.2_solid-js@1.7.2_solid-start-node_rc6adjatyfg7tlpbwy4hzftybm/node_modules/solid-start/api/middleware.ts",
  "../../node_modules/.pnpm/solid-start@0.2.24_@solidjs+meta@0.28.4_@solidjs+router@0.8.2_solid-js@1.7.2_solid-start-node_rc6adjatyfg7tlpbwy4hzftybm/node_modules/solid-start/api/router.ts",
  "../../node_modules/.pnpm/solid-start@0.2.24_@solidjs+meta@0.28.4_@solidjs+router@0.8.2_solid-js@1.7.2_solid-start-node_rc6adjatyfg7tlpbwy4hzftybm/node_modules/solid-start/api/types.ts",
  "../../node_modules/.pnpm/solid-start@0.2.24_@solidjs+meta@0.28.4_@solidjs+router@0.8.2_solid-js@1.7.2_solid-start-node_rc6adjatyfg7tlpbwy4hzftybm/node_modules/solid-start/data/createRouteAction.tsx",
  "../../node_modules/.pnpm/solid-start@0.2.24_@solidjs+meta@0.28.4_@solidjs+router@0.8.2_solid-js@1.7.2_solid-start-node_rc6adjatyfg7tlpbwy4hzftybm/node_modules/solid-start/data/createRouteData.tsx",
  "../../node_modules/.pnpm/solid-start@0.2.24_@solidjs+meta@0.28.4_@solidjs+router@0.8.2_solid-js@1.7.2_solid-start-node_rc6adjatyfg7tlpbwy4hzftybm/node_modules/solid-start/data/Form.tsx",
  "../../node_modules/.pnpm/solid-start@0.2.24_@solidjs+meta@0.28.4_@solidjs+router@0.8.2_solid-js@1.7.2_solid-start-node_rc6adjatyfg7tlpbwy4hzftybm/node_modules/solid-start/data/FormError.tsx",
  "../../node_modules/.pnpm/solid-start@0.2.24_@solidjs+meta@0.28.4_@solidjs+router@0.8.2_solid-js@1.7.2_solid-start-node_rc6adjatyfg7tlpbwy4hzftybm/node_modules/solid-start/entry-client/mount.tsx",
  "../../node_modules/.pnpm/solid-start@0.2.24_@solidjs+meta@0.28.4_@solidjs+router@0.8.2_solid-js@1.7.2_solid-start-node_rc6adjatyfg7tlpbwy4hzftybm/node_modules/solid-start/entry-client/StartClient.tsx",
  "../../node_modules/.pnpm/solid-start@0.2.24_@solidjs+meta@0.28.4_@solidjs+router@0.8.2_solid-js@1.7.2_solid-start-node_rc6adjatyfg7tlpbwy4hzftybm/node_modules/solid-start/entry-server/render.ts",
  "../../node_modules/.pnpm/solid-start@0.2.24_@solidjs+meta@0.28.4_@solidjs+router@0.8.2_solid-js@1.7.2_solid-start-node_rc6adjatyfg7tlpbwy4hzftybm/node_modules/solid-start/entry-server/StartServer.tsx",
  "../../node_modules/.pnpm/solid-start@0.2.24_@solidjs+meta@0.28.4_@solidjs+router@0.8.2_solid-js@1.7.2_solid-start-node_rc6adjatyfg7tlpbwy4hzftybm/node_modules/solid-start/error-boundary/ErrorBoundary.tsx",
  "../../node_modules/.pnpm/solid-start@0.2.24_@solidjs+meta@0.28.4_@solidjs+router@0.8.2_solid-js@1.7.2_solid-start-node_rc6adjatyfg7tlpbwy4hzftybm/node_modules/solid-start/index.tsx",
  "../../node_modules/.pnpm/solid-start@0.2.24_@solidjs+meta@0.28.4_@solidjs+router@0.8.2_solid-js@1.7.2_solid-start-node_rc6adjatyfg7tlpbwy4hzftybm/node_modules/solid-start/islands/index.tsx",
  "../../node_modules/.pnpm/solid-start@0.2.24_@solidjs+meta@0.28.4_@solidjs+router@0.8.2_solid-js@1.7.2_solid-start-node_rc6adjatyfg7tlpbwy4hzftybm/node_modules/solid-start/islands/router.ts",
  "../../node_modules/.pnpm/solid-start@0.2.24_@solidjs+meta@0.28.4_@solidjs+router@0.8.2_solid-js@1.7.2_solid-start-node_rc6adjatyfg7tlpbwy4hzftybm/node_modules/solid-start/islands/server-router.tsx",
  "../../node_modules/.pnpm/solid-start@0.2.24_@solidjs+meta@0.28.4_@solidjs+router@0.8.2_solid-js@1.7.2_solid-start-node_rc6adjatyfg7tlpbwy4hzftybm/node_modules/solid-start/root/Document.tsx",
  "../../node_modules/.pnpm/solid-start@0.2.24_@solidjs+meta@0.28.4_@solidjs+router@0.8.2_solid-js@1.7.2_solid-start-node_rc6adjatyfg7tlpbwy4hzftybm/node_modules/solid-start/root/InlineStyles.tsx",
  "../../node_modules/.pnpm/solid-start@0.2.24_@solidjs+meta@0.28.4_@solidjs+router@0.8.2_solid-js@1.7.2_solid-start-node_rc6adjatyfg7tlpbwy4hzftybm/node_modules/solid-start/root/Links.tsx",
  "../../node_modules/.pnpm/solid-start@0.2.24_@solidjs+meta@0.28.4_@solidjs+router@0.8.2_solid-js@1.7.2_solid-start-node_rc6adjatyfg7tlpbwy4hzftybm/node_modules/solid-start/router.tsx",
  "../../node_modules/.pnpm/solid-start@0.2.24_@solidjs+meta@0.28.4_@solidjs+router@0.8.2_solid-js@1.7.2_solid-start-node_rc6adjatyfg7tlpbwy4hzftybm/node_modules/solid-start/server/middleware.ts",
  "../../node_modules/.pnpm/solid-start@0.2.24_@solidjs+meta@0.28.4_@solidjs+router@0.8.2_solid-js@1.7.2_solid-start-node_rc6adjatyfg7tlpbwy4hzftybm/node_modules/solid-start/server/responses.ts",
  "../../node_modules/.pnpm/solid-start@0.2.24_@solidjs+meta@0.28.4_@solidjs+router@0.8.2_solid-js@1.7.2_solid-start-node_rc6adjatyfg7tlpbwy4hzftybm/node_modules/solid-start/server/server-functions/server.ts",
  "../../node_modules/.pnpm/solid-start@0.2.24_@solidjs+meta@0.28.4_@solidjs+router@0.8.2_solid-js@1.7.2_solid-start-node_rc6adjatyfg7tlpbwy4hzftybm/node_modules/solid-start/server/server-functions/types.ts",
  "../../node_modules/.pnpm/solid-start@0.2.24_@solidjs+meta@0.28.4_@solidjs+router@0.8.2_solid-js@1.7.2_solid-start-node_rc6adjatyfg7tlpbwy4hzftybm/node_modules/solid-start/server/ServerContext.tsx",
  "../../node_modules/.pnpm/solid-start@0.2.24_@solidjs+meta@0.28.4_@solidjs+router@0.8.2_solid-js@1.7.2_solid-start-node_rc6adjatyfg7tlpbwy4hzftybm/node_modules/solid-start/session/cookie.ts",
  "../../node_modules/.pnpm/solid-start@0.2.24_@solidjs+meta@0.28.4_@solidjs+router@0.8.2_solid-js@1.7.2_solid-start-node_rc6adjatyfg7tlpbwy4hzftybm/node_modules/solid-start/session/cookies.ts",
  "../../node_modules/.pnpm/solid-start@0.2.24_@solidjs+meta@0.28.4_@solidjs+router@0.8.2_solid-js@1.7.2_solid-start-node_rc6adjatyfg7tlpbwy4hzftybm/node_modules/solid-start/session/cookieSigning.ts",
  "../../node_modules/.pnpm/solid-start@0.2.24_@solidjs+meta@0.28.4_@solidjs+router@0.8.2_solid-js@1.7.2_solid-start-node_rc6adjatyfg7tlpbwy4hzftybm/node_modules/solid-start/session/memoryStorage.ts",
  "../../node_modules/.pnpm/solid-start@0.2.24_@solidjs+meta@0.28.4_@solidjs+router@0.8.2_solid-js@1.7.2_solid-start-node_rc6adjatyfg7tlpbwy4hzftybm/node_modules/solid-start/session/sessions.ts",
  "../../node_modules/.pnpm/solid-start@0.2.24_@solidjs+meta@0.28.4_@solidjs+router@0.8.2_solid-js@1.7.2_solid-start-node_rc6adjatyfg7tlpbwy4hzftybm/node_modules/solid-start/types.ts",
];

Promise.allSettled(FILES.map(addTsNoCheck)).then((results) => {
  let hasErrors = false;

  for (const result of results) {
    if (result.status === "rejected") {
      hasErrors = true;
      console.error(result.reason);
    }
  }

  if (hasErrors) {
    process.exit(1);
  }
});

async function addTsNoCheck(file) {
  const content = fs.readFileSync(file).toString();

  if (content.includes(ADDED_STR)) {
    console.debug(JSON.stringify(ADDED_STR), "is already in", file);
  } else {
    fs.writeFileSync(file, ADDED_STR + content);
    console.debug(JSON.stringify(ADDED_STR), "added into", file);
  }
}
