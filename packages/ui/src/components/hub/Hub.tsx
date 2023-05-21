import { createSignal } from "solid-js";
import { useLocalStorage, useWindowSize } from "solidjs-use";
import styles from "./Hub.module.scss";
import HubCompanion from "./HubCompanion";

export default function Hub() {
  const [dialog, setDialog] = createSignal<HTMLDialogElement>();
  const [_open, setOpen] = createSignal(false);
  const [closing, setClosing] = createSignal(false);

  const { width: screenWidth, height: screenHeight } = useWindowSize();

  const [x, setX] = useLocalStorage("hub-button-x", screenWidth() / 2);
  const [y, setY] = useLocalStorage("hub-button-y", screenHeight());

  return (
    <>
      <HubCompanion
        onClick={() => {
          dialog()?.showModal();
          setOpen(true);
        }}
        x={x}
        setX={setX}
        y={y}
        setY={setY}
      />
      <dialog
        ref={setDialog}
        classList={{
          [styles.dialog]: true,
          [styles.fadeOut]: closing(),
        }}
        onClick={() => {
          setClosing(true);
          setOpen(false);
        }}
        onAnimationEnd={() => {
          if (!closing()) return;

          dialog()?.close();
          setClosing(false);
        }}
      ></dialog>
    </>
  );
}
