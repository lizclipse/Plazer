import type { TypedDocumentNode } from "@apollo/client";
import { createMutation, gql } from "@merged/solid-apollo";
import { createRouteAction, redirect } from "solid-start";
import DisplayError from "~/components/DisplayError";
import { GQL_ACCOUNT, useAccount } from "~/contexts";
import styles from "~/form.module.scss";
import { Trans } from "~/i18n";
import type { LoginMutation, LoginMutationVariables } from "~gen/graphql";

const GQL: TypedDocumentNode<LoginMutation, LoginMutationVariables> = gql`
  mutation Login($creds: AuthCreds!) {
    login(creds: $creds) {
      ...AccountFields
    }
  }
  ${GQL_ACCOUNT}
`;

const inputs = {
  userId: "userId",
  pword: "password",
} as const;
export default function Login() {
  const { login } = useAccount();
  const [requestLogin] = createMutation(GQL);

  const [create, { Form }] = createRouteAction(async (form: FormData) => {
    const creds = {
      userId: form.get(inputs.userId) as string,
      pword: form.get(inputs.pword) as string,
    };

    const result = await requestLogin({ variables: { creds } });
    login(result.login);
    return redirect("/");
  });

  return (
    <section class={styles.form}>
      <h1>
        <Trans>{(t) => t.core.account.loginTitle()}</Trans>
      </h1>
      <Form>
        <label for={inputs.userId}>
          <Trans>{(t) => t.core.account.userId()}</Trans>
        </label>
        <input
          id={inputs.userId}
          name={inputs.userId}
          autoCapitalize="off"
          spellcheck={false}
          autocorrect="off"
          required
        />

        <label for={inputs.pword}>
          <Trans>{(t) => t.core.account.password()}</Trans>
        </label>
        <input
          id={inputs.pword}
          name={inputs.pword}
          type="password"
          required
          minlength={8}
        />

        <button type="submit" disabled={create.pending}>
          <Trans>{(t) => t.core.account.loginSubmit()}</Trans>
        </button>

        <DisplayError error={() => create.error as unknown} keepSpacing />
      </Form>
    </section>
  );
}
