import { Trans, useTransContext } from "@mbarzda/solid-i18next";
import { createSignal, Show } from "solid-js";
import { A } from "solid-start";
import styles from "./Hub.module.scss";
import HubButton from "./HubButton";
import HubCompanion from "./HubCompanion";
import { useAccount } from "~/contexts";

type State = "closed" | "opening" | "open" | "closing";

function Actions() {
  const [t] = useTransContext();

  return (
    <>
      <button
        title={t("nav.createPost")}
        class={styles.createPost}
        onClick={() => console.log("create post")}
      >
        <span>📝</span>
      </button>
    </>
  );
}

function NavButtons() {
  const [t] = useTransContext();

  return (
    <nav class={styles.navButtons}>
      <A href="/" title={t("nav.home")} activeClass={styles.activeNav} end>
        <span>🏡</span>
      </A>
      <A href="/search" title={t("nav.search")} activeClass={styles.activeNav}>
        <span>🔍</span>
      </A>
    </nav>
  );
}

function AnonLinks() {
  return (
    <>
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
    </>
  );
}

function AccountLinks({ close }: { readonly close: () => void }) {
  const { logout } = useAccount();

  return (
    <>
      <A
        href="/settings"
        class={styles.navSettings}
        activeClass={styles.activeNav}
      >
        <Trans key="nav.settings">
          <span aria-hidden>{""}</span>
          <span class={styles.inner}>{""}</span>
        </Trans>
      </A>
      <button
        class={styles.navLogout}
        onClick={() => {
          logout();
          // Logout seems to suppress the event bubbling.
          close();
        }}
      >
        <Trans key="nav.logout">
          <span aria-hidden>{""}</span>
          <span class={styles.inner}>{""}</span>
        </Trans>
      </button>
    </>
  );
}

export default function Hub() {
  const [dialog, setDialog] = createSignal<HTMLDialogElement>();
  const [state, setState] = createSignal<State>("closed");
  const [t] = useTransContext();
  const { account } = useAccount();

  const close = () => setState("closing");

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
        onClick={close}
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
        <NavButtons />
        <Show when={account()} fallback={<AnonLinks />}>
          <AccountLinks close={close} />
        </Show>
        <Actions />
      </dialog>
    </>
  );
}
