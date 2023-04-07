import {
  createMutation,
  createQuery,
  gql,
  useApollo,
} from "@merged/solid-apollo";
import {
  createComputed,
  createMemo,
  createSignal,
  onCleanup,
  Suspense,
} from "solid-js";
import "./Counter.scss";
import { isServer } from "solid-js/web";
import type {
  GetCountQuery,
  GetCountQueryVariables,
  IncrementMutation,
  IncrementMutationVariables,
  OnCountChangedSubscription,
  OnCountChangedSubscriptionVariables,
} from "./Counter.gql";

const QUERY_COUNT = gql`
  query GetCount {
    count
  }
`;

const SUBSCRIPTION_COUNT = gql`
  subscription OnCountChanged {
    changes
  }
`;

const MUTATION_INCREMENT = gql`
  mutation Increment($by: Int) {
    increment(by: $by)
  }
`;

export default function Counter() {
  const countQuery = createQuery<GetCountQuery, GetCountQueryVariables>(
    QUERY_COUNT
  );
  const count = createMemo(() => countQuery()?.count);

  const [countStore, setCount] = createSignal<number | undefined>(undefined);
  createComputed(() => setCount(count()));

  const client = useApollo();
  if (!isServer) {
    const sub = client
      .subscribe<
        OnCountChangedSubscription,
        OnCountChangedSubscriptionVariables
      >({
        query: SUBSCRIPTION_COUNT,
      })
      .subscribe(({ data }) => {
        if (data) {
          setCount(data.changes);
        }
      });

    onCleanup(() => sub.unsubscribe());
  }

  const [increment] = createMutation<
    IncrementMutation,
    IncrementMutationVariables
  >(MUTATION_INCREMENT);

  return (
    <Suspense fallback={<p>Loading...</p>}>
      <button class="increment" onClick={() => void increment()}>
        Clicks: {countStore()}
      </button>
    </Suspense>
  );
}
