import type { TypedDocumentNode } from "@apollo/client";
import { createMutation, gql } from "@merged/solid-apollo";
import { createRouteAction, redirect } from "solid-start";
import { GQL_BOARD_CARD } from "~/components/board/BoardCard";
import DisplayError from "~/components/DisplayError";
import styles from "~/form.module.scss";
import { Trans, useTrans } from "~/i18n";
import type { FormFields, TFormData } from "~/types";
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

interface Inputs {
  handle?: string;
  name?: string;
  desc?: string;
}

const inputs: FormFields<Inputs> = {
  handle: "handle",
  name: "name",
  desc: "desc",
};

export default function BoardCreate() {
  const [t] = useTrans();
  const [createBoard] = createMutation(GQL);

  const [create, { Form }] = createRouteAction(
    async (form: TFormData<Inputs>) => {
      const board: CreateBoard = {
        handle: form.get(inputs.handle),
        name: form.get(inputs.name),
        description: form.get(inputs.desc),
      };

      const result = await createBoard({ variables: { board } });
      return redirect(`/b/${encodeURIComponent(result.createBoard.handle)}`);
    },
  );

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
