import type { TypedDocumentNode } from "@apollo/client/core";
import { gql } from "@merged/solid-apollo";
import { A } from "@solidjs/router";
import { Show } from "solid-js";
import styles from "./BoardCard.module.scss";
import { useTrans } from "~/i18n";
import type { BoardCardFieldsFragment } from "~gen/graphql";

export const GQL_BOARD_CARD: TypedDocumentNode<BoardCardFieldsFragment, void> =
  gql`
    fragment BoardCardFields on Board {
      id
      handle
      name
      description
    }
  `;

export interface BoardCardProps {
  readonly board: BoardCardFieldsFragment;
}

export default function BoardCard(props: BoardCardProps) {
  const [t] = useTrans();

  return (
    <article class={styles.article}>
      <hgroup>
        <h2>{props.board.name || `@${props.board.handle}`}</h2>
        <A href={`/b/${props.board.handle}`} aria-label={t().board.nav.view()}>
          <span class={styles.default}>📁</span>
          <span class={styles.hover}>📂</span>
        </A>
      </hgroup>
      <Show when={props.board.description}>
        <hr />
        <p>{props.board.description}</p>
      </Show>
    </article>
  );
}
