import type { TypedDocumentNode } from "@apollo/client";
import { createMutation, gql } from "@merged/solid-apollo";
import { createRouteAction, redirect } from "solid-start";
import { GQL_BOARD_CARD } from "~/components/board/BoardCard";
import DisplayError from "~/components/DisplayError";
import styles from "~/form.module.scss";
import { Trans, useTrans } from "~/i18n";
import type {
  CreateBoard,
  CreateBoardMutation,
  CreateBoardMutationVariables,
} from "~gen/graphql";

const GQL: TypedDocumentNode<
  CreateBoardMutation,
  CreateBoardMutationVariables
> = gql`
  mutation CreateBoard($board: CreateBoard!) {
    createBoard(create: $board) {
      id
      ...BoardCardFields
    }
  }
  ${GQL_BOARD_CARD}
`;

const inputs = {
  handle: "handle",
  name: "name",
  desc: "description",
} as const;
export default function BoardCreate() {
  const [t] = useTrans();
  const [createBoard] = createMutation(GQL, {
    // TODO: figure out why the cache is broken or scrap it because fuck it
    refetchQueries: ["ListBoards"],
  });

  const [create, { Form }] = createRouteAction(async (form: FormData) => {
    const board: CreateBoard = {
      handle: form.get(inputs.handle) as string,
      name: form.get(inputs.name) as string,
      description: form.get(inputs.desc) as string,
    };

    await createBoard({ variables: { board } });
    return redirect(`/b/${board.handle}`);
  });

  return (
    <section class={styles.form}>
      <h1>
        <Trans>{(t) => t.board.create.title()}</Trans>
      </h1>
      <Form>
        <label for={inputs.handle}>
          <Trans>{(t) => t.board.create.handle()}</Trans>
        </label>
        <input
          id={inputs.handle}
          name={inputs.handle}
          autoCapitalize="off"
          spellcheck={false}
          autocorrect="off"
          maxlength={128}
        />

        <label for={inputs.name}>
          <Trans>{(t) => t.board.create.name()}</Trans>
        </label>
        <input
          id={inputs.name}
          name={inputs.name}
          autoCapitalize="on"
          spellcheck={false}
          autocorrect="off"
          maxlength={1024}
        />

        <label for={inputs.desc}>
          <Trans>{(t) => t.board.create.desc()}</Trans>
        </label>
        <input
          id={inputs.desc}
          name={inputs.desc}
          autoCapitalize="on"
          spellcheck={true}
          autocorrect="on"
          maxlength={32_768}
        />

        <button type="submit" disabled={create.pending}>
          <Trans>{(t) => t.board.create.submit()}</Trans>
        </button>

        <DisplayError
          error={() => create.error as unknown}
          keepSpacing
          overrides={() => t().board.errors}
        />
      </Form>
    </section>
  );
}
