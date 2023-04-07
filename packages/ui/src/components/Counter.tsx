import { createQuery, gql } from "@merged/solid-apollo";
import { Suspense } from "solid-js";
import "./Counter.scss";
import { isServer } from "solid-js/web";
import type { GetCountQuery, GetCountQueryVariables } from "./Counter.gql";

const base = isServer ? "http://localhost:8080" : "";

const QUERY_COUNT = gql`
  query GetCount {
    count
  }
`;

export default function Counter() {
  const count = createQuery<GetCountQuery, GetCountQueryVariables>(QUERY_COUNT);

  return (
    <Suspense fallback={<p>Loading...</p>}>
      <button class="increment">Clicks: {count()?.count}</button>
    </Suspense>
  );
}
