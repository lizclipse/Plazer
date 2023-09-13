import { createSignal, Show } from "solid-js";
import { A } from "solid-start";
import styles from "./Hub.module.scss";
import HubButton from "./HubButton";
import HubCompanion from "./HubCompanion";
import { useAccount } from "~/contexts";
import { Trans, useTrans } from "~/i18n";

type State = "closed" | "opening" | "open" | "closing";

export default function Hub() {
  const [dialog, setDialog] = createSignal<HTMLDialogElement>();
  const [state, setState] = createSignal<State>("closed");
  const [t] = useTrans();
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
          title={t().core.nav.closeHub()}
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

function Actions() {
  const [t] = useTrans();

  return (
    <>
      <button
        title={t().core.nav.createPost()}
        class={styles.createPost}
        onClick={() => console.log("create post")}
      >
        <span>📝</span>
      </button>
    </>
  );
}

function NavButtons() {
  const [t] = useTrans();

  return (
    <nav class={styles.navButtons}>
      <A
        href="/"
        title={t().core.nav.home()}
        class="btn"
        activeClass={styles.activeNav}
        end
      >
        <span>🏡</span>
      </A>
      <A
        href="/b"
        title={t().core.nav.boards()}
        class="btn"
        activeClass={styles.activeNav}
      >
        <span>🗃️</span>
      </A>
      <A
        href="/search"
        title={t().core.nav.search()}
        class="btn"
        activeClass={styles.activeNav}
      >
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
        class={"btn " + styles.navRegister}
        activeClass={styles.activeNav}
      >
        <Trans>
          {(t) => t.core.nav.register({ span: { class: styles.inner } })}
        </Trans>
      </A>
      <A
        href="/login"
        class={"btn " + styles.navLogin}
        activeClass={styles.activeNav}
      >
        <Trans>
          {(t) => t.core.nav.login({ span: { class: styles.inner } })}
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
        class={"btn " + styles.navSettings}
        activeClass={styles.activeNav}
      >
        <Trans>
          {(t) => t.core.nav.settings({ span: { class: styles.inner } })}
        </Trans>
      </A>
      <button
        class={"btn " + styles.navLogout}
        onClick={() => {
          logout();
          // Logout seems to suppress the event bubbling.
          close();
        }}
      >
        <Trans>
          {(t) => t.core.nav.logout({ span: { class: styles.inner } })}
        </Trans>
      </button>
    </>
  );
}
