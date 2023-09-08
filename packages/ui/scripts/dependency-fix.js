import fs from "node:fs/promises";
import path from "node:path";

const ADDED_STR = "// @ts-nocheck\n\n";
const TS_NO_CHECK_RE = /node_modules\/solid-start\/(.*)\.tsx?$/;

const SOLID_APOLLO_ORIGINAL = "} from '@apollo/client/core'";
const SOLID_APOLLO_UPDATED = "} from '@apollo/client/core/index.js'";
const SOLID_APOLLO_RE = /@merged\+solid-apollo(.*)\/dist\/es\/(.*)\.js$/;

main();

async function main() {
  const tsNoCheckFiles = [];
  const solidApolloFiles = [];
  for await (const f of walk(
    new URL("../../../node_modules", import.meta.url).pathname,
  )) {
    if (TS_NO_CHECK_RE.test(f) && !f.endsWith(".d.ts")) {
      tsNoCheckFiles.push(f);
    } else if (SOLID_APOLLO_RE.test(f)) {
      solidApolloFiles.push(f);
    }
  }

  const results = await Promise.allSettled([
    ...tsNoCheckFiles.map(addTsNoCheck),
    ...solidApolloFiles.map(remapApolloImport),
  ]);
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
}

async function* walk(dir) {
  for await (const d of await fs.opendir(dir)) {
    const entry = path.join(dir, d.name);
    if (d.isDirectory()) yield* walk(entry);
    else if (d.isFile()) yield entry;
  }
}

async function addTsNoCheck(file) {
  const content = (await fs.readFile(file)).toString();
  const stripped = file.match(/node_modules\/solid-start\/(.*)$/)[1];

  if (content.includes(ADDED_STR)) {
    console.debug(JSON.stringify(ADDED_STR), "is already in", stripped);
  } else {
    await fs.writeFile(file, ADDED_STR + content);
    console.debug(JSON.stringify(ADDED_STR), "added into", stripped);
  }
}

async function remapApolloImport(file) {
  const content = (await fs.readFile(file)).toString();
  const stripped = file.match(/dist\/es\/(.*)$/)[1];

  if (content.includes(SOLID_APOLLO_UPDATED)) {
    console.debug("import fix already applied to", stripped);
  } else {
    await fs.writeFile(
      file,
      content.replace(SOLID_APOLLO_ORIGINAL, SOLID_APOLLO_UPDATED),
    );
    console.debug("import fix applied to", stripped);
  }
}
