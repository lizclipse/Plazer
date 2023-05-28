import { type Accessor, createContext, useContext } from "solid-js";
import type { JSX } from "solid-js/web/types/jsx";
import {
  StorageSerializers,
  useLocalStorage,
  useSessionStorage,
} from "solidjs-use";
import type { Account, AuthenticatedAccount } from "~gen/graphql";

export interface AccountCtx {
  readonly account: Accessor<Account | undefined>;
  readonly refreshToken: Accessor<string | undefined>;
  readonly accessToken: Accessor<string | undefined>;
  readonly logout: () => void;
  readonly login: (account: AuthenticatedAccount) => void;
}

const AccountContext = createContext<AccountCtx>();

export function useAccount(): AccountCtx {
  // Will always be set by Contexts
  // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
  return useContext(AccountContext)!;
}

export function AccountProvider({
  children,
}: {
  children: () => JSX.Element;
}): JSX.Element {
  const [account, setAccount] = useLocalStorage<Account | undefined>(
    "account",
    undefined,
    { serializer: StorageSerializers.object }
  );
  const [refreshToken, setRefreshToken] = useLocalStorage<string | undefined>(
    "refreshToken",
    undefined,
    { serializer: StorageSerializers.string }
  );
  const [accessToken, setAccessToken] = useSessionStorage<string | undefined>(
    "accessToken",
    undefined,
    { serializer: StorageSerializers.string }
  );

  const logout = () => {
    setAccount(undefined);
    setRefreshToken(undefined);
    setAccessToken(undefined);
  };

  const login = (acc: AuthenticatedAccount) => {
    setAccount(acc.account);
    setRefreshToken(acc.refreshToken);
    setAccessToken(acc.accessToken);
  };

  return (
    <AccountContext.Provider
      value={{ account, refreshToken, accessToken, logout, login }}
    >
      {children()}
    </AccountContext.Provider>
  );
}
