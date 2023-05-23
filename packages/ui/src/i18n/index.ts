import { createInstance } from "i18next";
import { isServer } from "solid-js/web";
import enCommon from "./en/common.json";

export const defaultNS = "common";

export const resources = {
  "en-GB": {
    common: enCommon,
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
