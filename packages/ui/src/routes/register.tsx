import { type TypedDocumentNode } from "@apollo/client/core";
import { createMutation, gql } from "@merged/solid-apollo";
import { createRouteAction, redirect } from "solid-start";
import styles from "./register.module.scss";
import { DisplayError } from "~/components/DisplayError";
import { useAccount } from "~/contexts";
import type {
  CreateAccountMutation,
  CreateAccountMutationVariables,
} from "~gen/graphql";

const GQL: TypedDocumentNode<
  CreateAccountMutation,
  CreateAccountMutationVariables
> = gql`
  mutation CreateAccount($account: CreateAccount!) {
    createAccount(create: $account) {
      account {
        id
        handle
        revokedAt
      }
      refreshToken
      accessToken
    }
  }
`;

const inputs = {
  handle: "handle",
  pword: "password",
} as const;
export default function Register() {
  const { login } = useAccount();
  const [createAccount] = createMutation(GQL);

  const [create, { Form }] = createRouteAction(async (form: FormData) => {
    const account = {
      handle: form.get(inputs.handle) as string,
      pword: form.get(inputs.pword) as string,
    };

    const result = await createAccount({ variables: { account } });
    login(result.createAccount);
    return redirect("/");
  });

  return (
    <section class={styles.form}>
      <h1>Create a new account</h1>
      <Form>
        <label for={inputs.handle}>Login</label>
        <input id={inputs.handle} name={inputs.handle} required />

        <label for={inputs.pword}>Password</label>
        <input
          id={inputs.pword}
          name={inputs.pword}
          type="password"
          required
          minlength={8}
        />

        <button type="submit" disabled={create.pending}>
          Create Account
        </button>

        <DisplayError err={() => create.error as unknown} />
      </Form>
    </section>
  );
}
