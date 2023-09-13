import type { ErrorI18n } from "~/types";

export default {
  nav: {
    createBoard: () => "Create a new board",
    ariaList: () => "Board page navigation",
    previous: () => "Previous page",
    next: () => "Next page",
    view: () => "View board",
  },

  create: {
    title: () => "Create a new board",
    handle: () => "Handle",
    name: () => "Name",
    desc: () => "Description",
    submit: () => "Create",
  },

  errors: {
    UnavailableIdent: () => "This handle is not available.",
  } satisfies Partial<ErrorI18n>,
} as const;
