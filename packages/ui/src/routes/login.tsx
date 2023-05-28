import type { TypedDocumentNode } from "@apollo/client";
import { Trans } from "@mbarzda/solid-i18next";
import { createMutation, gql } from "@merged/solid-apollo";
import { createRouteAction, redirect } from "solid-start";
import styles from "./register.module.scss";
import DisplayError from "~/components/DisplayError";
import { GQL_ACCOUNT, useAccount } from "~/contexts";
import type { LoginMutation, LoginMutationVariables } from "~gen/graphql";

const GQL: TypedDocumentNode<LoginMutation, LoginMutationVariables> = gql`
  ${GQL_ACCOUNT}
  mutation Login($creds: AuthCreds!) {
    login(creds: $creds) {
      ...AccountFields
    }
  }
`;

const inputs = {
  handle: "handle",
  pword: "password",
} as const;
export default function Login() {
  const { login } = useAccount();
  const [requestLogin] = createMutation(GQL);

  const [create, { Form }] = createRouteAction(async (form: FormData) => {
    const creds = {
      handle: form.get(inputs.handle) as string,
      pword: form.get(inputs.pword) as string,
    };

    const result = await requestLogin({ variables: { creds } });
    login(result.login);
    return redirect("/");
  });

  return (
    <section class={styles.form}>
      <h1>
        <Trans key="account.loginTitle" />
      </h1>
      <Form>
        <label for={inputs.handle}>
          <Trans key="account.handle" />
        </label>
        <input
          id={inputs.handle}
          name={inputs.handle}
          autoCapitalize="off"
          spellcheck={false}
          autocorrect="off"
          required
        />

        <label for={inputs.pword}>
          <Trans key="account.password" />
        </label>
        <input
          id={inputs.pword}
          name={inputs.pword}
          type="password"
          required
          minlength={8}
        />

        <button type="submit" disabled={create.pending}>
          <Trans key="account.loginSubmit" />
        </button>

        <DisplayError error={() => create.error as unknown} keepSpacing />
      </Form>
    </section>
  );
}
