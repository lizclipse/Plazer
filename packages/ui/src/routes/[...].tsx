import { Trans } from "~/i18n";

export default function NotFound() {
  return (
    <p>
      <Trans>{(t) => t.core.nav.notFound()}</Trans>
    </p>
  );
}
