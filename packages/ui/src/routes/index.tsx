import { A } from "solid-start";
import "./index.scss";

export default function Home() {
  return (
    <main>
      <A href="/login">Login</A>
      <A href="/register">Create Account</A>
    </main>
  );
}
