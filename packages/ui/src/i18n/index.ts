import en_gb from "./en_gb";
import { Trans as TransBase, useTrans as useTransBase } from "./trans";

export { TransProvider, type TransProviderProps } from "./trans";

export const baseTrans = en_gb;
export type BaseTrans = typeof baseTrans;

export const translations = {
  "en-GB": en_gb,
};

export const useTrans = useTransBase<BaseTrans>;
export const Trans = TransBase<BaseTrans>;
