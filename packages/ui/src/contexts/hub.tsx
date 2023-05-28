import {
  type Accessor,
  createContext,
  type FlowProps,
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

const maxInitialDistance = 1000;
const repositionOffset = 100;

export default function HubPositionProvider(props: FlowProps): JSX.Element {
  const { width: screenWidth, height: screenHeight } = useWindowSize();
  const [x, setX] = useLocalStorage("hub-button-x", screenWidth() / 2);
  const [y, setY] = useLocalStorage("hub-button-y", screenHeight());

  for (const [axis, setAxis, screen] of [
    [x, setX, screenWidth],
    [y, setY, screenHeight],
  ] as const) {
    const ax = axis();
    const sc = screen();
    if (ax < -maxInitialDistance) {
      setAxis(-repositionOffset);
    } else if (ax > sc + maxInitialDistance) {
      setAxis(sc + repositionOffset);
    }
  }

  return (
    <HubPositionContext.Provider value={{ x, setX, y, setY }}>
      {props.children}
    </HubPositionContext.Provider>
  );
}
