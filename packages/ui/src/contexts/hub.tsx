import {
  type Accessor,
  createContext,
  type JSX,
  type Setter,
  useContext,
} from "solid-js";
import { useLocalStorage, useWindowSize } from "solidjs-use";

export interface HubPosition {
  readonly x: Accessor<number>;
  readonly setX: Setter<number>;
  readonly y: Accessor<number>;
  readonly setY: Setter<number>;
}

const HubPositionContext = createContext<HubPosition>();

export function useHubPosition(): HubPosition {
  // Will always be set by Contexts
  // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
  return useContext(HubPositionContext)!;
}

export default function HubPositionProvider({
  children,
}: {
  readonly children: () => JSX.Element;
}): JSX.Element {
  const { width: screenWidth, height: screenHeight } = useWindowSize();
  const [x, setX] = useLocalStorage("hub-button-x", screenWidth() / 2);
  const [y, setY] = useLocalStorage("hub-button-y", screenHeight());

  return (
    <HubPositionContext.Provider value={{ x, setX, y, setY }}>
      {children()}
    </HubPositionContext.Provider>
  );
}
