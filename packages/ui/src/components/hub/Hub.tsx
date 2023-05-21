import { type Accessor, createSignal, type JSX, observable } from "solid-js";
import { useLocalStorage, useWindowSize } from "solidjs-use";
import styles from "./Hub.module.scss";
import HubButton from "./HubButton";
import HubCompanion from "./HubCompanion";

type State = "open" | "closing" | "closed";

interface MoveAnimationInit {
  button: Accessor<HTMLButtonElement | undefined>;
  x: Accessor<number>;
  y: Accessor<number>;
  state: Accessor<State>;
}

const animationOptions: KeyframeAnimationOptions = {
  duration: 200,
  easing: "ease-in-out",
};

function moveAnimation({ button, x, y, state }: MoveAnimationInit) {
  const [style, setStyle] = createSignal<JSX.CSSProperties>();
  const { width: screenWidth, height: screenHeight } = useWindowSize();

  // Using an observable here since we are responding to new changes, not handling
  // the current state, and this is the best way my head can deal with that.
  let listening = false;
  let anim: Animation | undefined;
  let original: { x: number; y: number } | undefined;
  observable(state).subscribe((state) => {
    // Skip initial value.
    if (!listening) {
      listening = true;
      return;
    }

    switch (state) {
      case "open":
        anim = button()?.animate(
          [
            { left: `${x()}px`, top: `${y()}px` },
            {
              left: `${screenWidth() / 2}px`,
              top: `${screenHeight() / 2}px`,
            },
          ],
          animationOptions
        );
        anim?.addEventListener("finish", () => {
          setStyle({ left: "50svw", top: "50svh" });
        });
        original = { x: x(), y: y() };
        break;
      case "closing": {
        anim?.cancel();
        if (!original) break;

        const { x: originalX, y: originalY } = original;
        anim = button()?.animate(
          [
            {
              left: `${screenWidth() / 2}px`,
              top: `${screenHeight() / 2}px`,
            },
            { left: `${originalX}px`, top: `${originalY}px` },
          ],
          animationOptions
        );
        anim?.addEventListener("finish", () => {
          setStyle(undefined);
        });
        break;
      }
    }
  });

  return { style };
}

export default function Hub() {
  const [dialog, setDialog] = createSignal<HTMLDialogElement>();
  const [button, setButton] = createSignal<HTMLButtonElement>();
  const [state, setState] = createSignal<State>("closed");

  const { width: screenWidth, height: screenHeight } = useWindowSize();

  const [x, setX] = useLocalStorage("hub-button-x", screenWidth() / 2);
  const [y, setY] = useLocalStorage("hub-button-y", screenHeight());

  const { style: buttonStyle } = moveAnimation({ button, x, y, state });

  return (
    <>
      <HubCompanion
        onClick={() => {
          dialog()?.showModal();
          setState("open");
        }}
        x={x}
        setX={setX}
        y={y}
        setY={setY}
        hidden={() => state() !== "closed"}
      />
      <dialog
        ref={setDialog}
        classList={{
          [styles.dialog]: true,
          [styles.fadeOut]: state() === "closing",
        }}
        onClick={() => {
          setState("closing");
        }}
        onAnimationEnd={() => {
          if (state() !== "closing") return;

          dialog()?.close();
          setState("closed");
        }}
      >
        <HubButton
          ref={setButton}
          style={{
            position: "fixed",
            left: `${x()}px`,
            top: `${y()}px`,
            ...buttonStyle(),
          }}
        />
      </dialog>
    </>
  );
}
