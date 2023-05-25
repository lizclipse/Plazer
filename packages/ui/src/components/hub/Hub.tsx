import { Trans, useTransContext } from "@mbarzda/solid-i18next";
import { createSignal } from "solid-js";
import { A } from "solid-start";
import styles from "./Hub.module.scss";
import HubButton from "./HubButton";
import HubCompanion from "./HubCompanion";

type State = "closed" | "opening" | "open" | "closing";

export default function Hub() {
  const [dialog, setDialog] = createSignal<HTMLDialogElement>();
  const [state, setState] = createSignal<State>("closed");
  const [t] = useTransContext();

  return (
    <>
      <HubCompanion
        onClick={() => {
          dialog()?.showModal();
          requestAnimationFrame(() => setState("opening"));
        }}
        hidden={() => state() !== "closed"}
      />
      <dialog
        ref={setDialog}
        classList={{
          [styles.dialog]: true,
          [styles.open]: state() !== "closed",
          [styles.fadeOut]: state() === "closing",
        }}
        onClick={() => {
          setState("closing");
        }}
      >
        <HubButton
          title={t("nav.closeHub")}
          class={styles.hubButton}
          onTransitionEnd={() => {
            switch (state()) {
              case "opening":
                setState("open");
                break;
              case "closing":
                dialog()?.close();
                setState("closed");
                break;
            }
          }}
        />
        <A
          href="/"
          title={t("nav.home")}
          class={styles.navHome}
          activeClass={styles.activeNav}
          end
        >
          <span>🏡</span>
        </A>
        <A
          href="/search"
          title={t("nav.search")}
          class={styles.navSearch}
          activeClass={styles.activeNav}
          end
        >
          <span>🔍</span>
        </A>
        <A
          href="/register"
          class={styles.navRegister}
          activeClass={styles.activeNav}
        >
          <Trans key="nav.register">
            <span aria-hidden>{""}</span>
            <span class={styles.inner}>{""}</span>
          </Trans>
        </A>
        <A href="/login" class={styles.navLogin} activeClass={styles.activeNav}>
          <Trans key="nav.login">
            <span aria-hidden>{""}</span>
            <span class={styles.inner}>{""}</span>
          </Trans>
        </A>
        <button title={t("nav.createPost")} class={styles.createPost}>
          <span>📝</span>
        </button>
      </dialog>
    </>
  );
}
