import type { JSX } from "solid-js";
import styles from "./HubButton.module.scss";

export type HubButtonProps = JSX.HTMLElementTags["button"];

export default function HubButton(props: HubButtonProps) {
  return (
    <button
      {...props}
      class={styles.button + (props.class ? " " + props.class : "")}
    >
      💠
    </button>
  );
}
