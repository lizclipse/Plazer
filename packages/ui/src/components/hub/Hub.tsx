import { Trans, useTransContext } from "@mbarzda/solid-i18next";
import { type Accessor, createSignal, type JSX, observable } from "solid-js";
import { A } from "solid-start";
import { useLocalStorage, useWindowSize } from "solidjs-use";
import styles from "./Hub.module.scss";
import HubButton from "./HubButton";
import HubCompanion from "./HubCompanion";

type State = "closed" | "opening" | "open" | "closing";

interface MoveAnimationInit {
  button: Accessor<HTMLButtonElement | undefined>;
  x: Accessor<number>;
  y: Accessor<number>;
  state: Accessor<State>;
  onAnimEnd: () => void;
}

const animationOptions: KeyframeAnimationOptions = {
  duration: 200,
  easing: "ease-in-out",
};

function moveAnimation({ button, x, y, state, onAnimEnd }: MoveAnimationInit) {
  const [style, setStyle] = createSignal<JSX.CSSProperties>();
  const { width: screenWidth, height: screenHeight } = useWindowSize();

  // Using an observable here since we are responding to new changes, not handling
  // the current state, and this is the best way my head can deal with that.
  let anim: Animation | undefined;
  observable(state).subscribe((state) => {
    switch (state) {
      case "opening":
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
          onAnimEnd();
        });
        break;
      case "closing": {
        anim?.cancel();
        anim = button()?.animate(
          [
            {
              left: `${screenWidth() / 2}px`,
              top: `${screenHeight() / 2}px`,
            },
            { left: `${x()}px`, top: `${y()}px` },
          ],
          animationOptions
        );
        anim?.addEventListener("finish", () => {
          setStyle(undefined);
          onAnimEnd();
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
  const [t] = useTransContext();

  const { width: screenWidth, height: screenHeight } = useWindowSize();

  const [x, setX] = useLocalStorage("hub-button-x", screenWidth() / 2);
  const [y, setY] = useLocalStorage("hub-button-y", screenHeight());

  const { style: buttonStyle } = moveAnimation({
    button,
    x,
    y,
    state,
    onAnimEnd: () => {
      switch (state()) {
        case "opening":
          setState("open");
          break;
        case "closing":
          dialog()?.close();
          setState("closed");
          break;
      }
    },
  });

  return (
    <>
      <HubCompanion
        onClick={() => {
          dialog()?.showModal();
          setState("opening");
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
          [styles.fadeIn]: state() === "opening",
          [styles.fadeOut]: state() === "closing",
        }}
        onClick={() => {
          setState("closing");
        }}
      >
        <HubButton
          ref={setButton}
          title={t("nav.closeHub")}
          style={{
            position: "fixed",
            left: `${x()}px`,
            top: `${y()}px`,
            ...buttonStyle(),
          }}
        />
        <A href="/register" class={styles.navRegister}>
          <Trans key="nav.register">
            <span aria-hidden={true}>{""}</span>
            <span class={styles.inner}>{""}</span>
          </Trans>
        </A>
        <A href="/login" class={styles.navLogin}>
          <Trans key="nav.login">
            <span aria-hidden={true}>{""}</span>
            <span class={styles.inner}>{""}</span>
          </Trans>
        </A>
      </dialog>
    </>
  );
}
