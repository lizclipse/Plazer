import { A } from "solid-start";
import "./index.scss";
import { useAccount } from "~/contexts";

export default function Home() {
  const { account } = useAccount();

  return (
    <>
      <h1>Home</h1>
      <p>
        First of all, you can <A href="/login">login</A>
      </p>
      <p>
        Otherwise, feel free to <A href="/register">create an account</A>
      </p>
      <pre>{JSON.stringify(account(), null, 2)}</pre>
    </>
  );
}
