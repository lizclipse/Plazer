import { A } from "solid-start";
import "./index.scss";

export default function Home() {
  return (
    <>
      <h1>Home</h1>
      <p>
        First of all, you can <A href="/login">login</A>
      </p>
      <p>
        Otherwise, feel free to <A href="/register">create an account</A>
      </p>
    </>
  );
}
