import { ApolloError } from "@apollo/client/core";
import { createMutation, gql } from "@merged/solid-apollo";
import { createRouteAction, redirect } from "solid-start";
import type {
  CreateAccountMutation,
  CreateAccountMutationVariables,
} from "./register.gql";

const GQL = gql`
  mutation CreateAccount($account: CreateAccount!) {
    createAccount(create: $account) {
      account {
        id
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
  const [createAccount] = createMutation<
    CreateAccountMutation,
    CreateAccountMutationVariables
  >(GQL);

  const [create, { Form }] = createRouteAction(async (form: FormData) => {
    const account = {
      handle: form.get(inputs.handle) as string,
      pword: form.get(inputs.pword) as string,
    };

    try {
      const result = await createAccount({ variables: { account } });
      console.debug(result);
      return redirect("/");
    } catch (err) {
      if (err instanceof ApolloError) {
        console.log(err.graphQLErrors);
      }
      console.error(err);
      throw err;
    }
  });

  return (
    <Form>
      <label for={inputs.handle}>Login</label>
      <input id={inputs.handle} name={inputs.handle} />

      <label for={inputs.pword}>Password</label>
      <input id={inputs.pword} name={inputs.pword} type="password" />

      <button type="submit" disabled={create.pending}>
        Create Account
      </button>
    </Form>
  );
}
