import { createInstance } from "i18next";
import { isServer } from "solid-js/web";
import enCore from "./en/core.json";

export const defaultNS = "core";

export const resources = {
  "en-GB": {
    core: enCore,
  },
} as const;

const i18next = createInstance();

await i18next.init({
  lng: "en-GB",
  fallbackLng: "en-GB",
  debug: import.meta.env.DEV && !isServer,
  defaultNS,
  resources,
  returnNull: false,
});

export { i18next };
