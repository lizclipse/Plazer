import type { TypedDocumentNode } from "@apollo/client";
import { createQuery, gql } from "@merged/solid-apollo";
import { Show } from "solid-js";
import { useParams } from "solid-start";
import styles from "./handle.module.scss";
import type { GetBoardQuery, GetBoardQueryVariables } from "~gen/graphql";

const GQL: TypedDocumentNode<GetBoardQuery, GetBoardQueryVariables> = gql`
  query GetBoard($handle: String!) {
    board(handle: $handle) {
      id
      handle
      name
      description
    }
  }
`;

type BoardParams = {
  handle: string;
};

export default function BoardsItem() {
  const params = useParams<BoardParams>();
  const handle = () => params.handle;

  const data = createQuery(GQL, () => ({
    variables: {
      handle: handle(),
    },
  }));

  return (
    <>
      <header class={styles.header}>
        <Show when={data()?.board?.name}>
          {(name) => (
            <>
              <h1>{name()}</h1>
            </>
          )}
        </Show>
        <h2>@{handle()}</h2>
      </header>
      <Show when={data()?.board?.description}>
        {(desc) => <p class={styles.desc}>{desc()}</p>}
      </Show>
    </>
  );
}
