import { A } from "@solidjs/router";
import styles from "./board-index.module.scss";
import BoardList from "~/components/board/BoardList";
import { Trans } from "~/i18n";

export default function BoardsIndex() {
  return (
    <>
      <BoardHeader />
      <BoardList />
    </>
  );
}

function BoardHeader() {
  return (
    <nav class={styles.nav}>
      <A class={styles.create} href="/b/_/create">
        <Trans>{(t) => t.board.nav.createBoard()}</Trans>
      </A>
    </nav>
  );
}
