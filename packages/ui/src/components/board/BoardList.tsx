import { type TypedDocumentNode } from "@apollo/client/core";
import { createQuery, gql } from "@merged/solid-apollo";
import { createSignal, For, Show } from "solid-js";
import BoardCard, { GQL_BOARD_CARD } from "./BoardCard";
import styles from "./BoardList.module.scss";
import { useTrans } from "~/i18n";
import type { ListBoardsQuery, ListBoardsQueryVariables } from "~gen/graphql";

const GQL: TypedDocumentNode<ListBoardsQuery, ListBoardsQueryVariables> = gql`
  query ListBoards($first: Int, $after: String, $last: Int, $before: String) {
    boards(first: $first, after: $after, last: $last, before: $before) {
      edges {
        cursor
        node {
          ...BoardCardFields
        }
      }
      pageInfo {
        startCursor
        hasPreviousPage
        endCursor
        hasNextPage
      }
    }
  }
  ${GQL_BOARD_CARD}
`;

type Cursor = { dir: "forwards" | "backwards"; cursor: string };

export default function BoardList() {
  const [t] = useTrans();

  const [cursor, setCursor] = createSignal<Cursor | undefined>(undefined);

  const data = createQuery(GQL, () => {
    const { dir, cursor: cur } = cursor() ?? {};
    return {
      variables: {
        first: 10,
        after: dir === "forwards" ? cur : undefined,
        before: dir === "backwards" ? cur : undefined,
      },
    };
  });

  const startCursor = () => {
    const pageInfo = data()?.boards.pageInfo;
    return pageInfo?.hasPreviousPage ? pageInfo?.startCursor : undefined;
  };

  const endCursor = () => {
    const pageInfo = data()?.boards.pageInfo;
    return pageInfo?.hasNextPage ? pageInfo?.endCursor : undefined;
  };

  return (
    <>
      <Show when={data()}>
        {(boards) => (
          <For each={boards().boards.edges}>
            {(edge) => <BoardCard board={edge.node} />}
          </For>
        )}
      </Show>
      <nav class={styles.pagination} aria-label={t().board.nav.ariaList()}>
        <Show when={startCursor()}>
          {(cursor) => (
            <button
              type="button"
              onClick={() => {
                setCursor({ dir: "backwards", cursor: cursor() });
              }}
            >
              {t().board.nav.previous()}
            </button>
          )}
        </Show>
        <Show when={endCursor()}>
          {(cursor) => (
            <button
              type="button"
              class={styles.next}
              onClick={() => {
                setCursor({ dir: "forwards", cursor: cursor() });
              }}
            >
              {t().board.nav.next()}
            </button>
          )}
        </Show>
      </nav>
    </>
  );
}
