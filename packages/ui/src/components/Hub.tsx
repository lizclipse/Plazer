import { createSignal } from "solid-js";
import styles from "./Hub.module.scss";
import HubButton from "./HubButton";

export default function Hub() {
  const [dialog, setDialog] = createSignal<HTMLDialogElement>();
  const [open, setOpen] = createSignal(false);
  const [closing, setClosing] = createSignal(false);

  return (
    <>
      <HubButton
        onClick={() => {
          dialog()?.showModal();
          setOpen(true);
        }}
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
