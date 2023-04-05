import { createResource, Suspense } from "solid-js";
import "./Counter.scss";
import { isServer } from "solid-js/web";

const base = isServer ? "http://localhost:8080" : "";

export default function Counter() {
  // const [count, setCount] = createSignal(0);
  const [count] = createResource(() =>
    fetch(`${base}/api/graphql`, {
      method: "POST",
      body: JSON.stringify({
        operationName: null,
        variables: {},
        query: "{\n  count\n}\n",
      }),
      headers: {
        "Content-Type": "application/json",
      },
    })
      .then((res) => res.json())
      // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
      .then((res) => res.data.count as number)
  );

  return (
    <Suspense fallback={<p>Loading...</p>}>
      <button class="increment">Clicks: {count()}</button>
    </Suspense>
  );
}
