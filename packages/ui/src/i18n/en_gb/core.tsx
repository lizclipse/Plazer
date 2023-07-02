import type { JSX } from "solid-js";

export default {
  nav: {
    closeHub: () => "Close Hub",
    openHub: () => "Open Hub",

    login: (props: { span: JSX.HTMLElementTags["span"] }) => (
      <>
        {"👤 "}
        <span {...props.span}>Login</span>
      </>
    ),
    register: (props: { span: JSX.HTMLElementTags["span"] }) => (
      <>
        {"✍️ "}
        <span {...props.span}>Register</span>
      </>
    ),
    settings: (props: { span: JSX.HTMLElementTags["span"] }) => (
      <>
        {"⚙️ "}
        <span {...props.span}>Settings</span>
      </>
    ),
    logout: (props: { span: JSX.HTMLElementTags["span"] }) => (
      <>
        {"🚪 "}
        <span {...props.span}>Logout</span>
      </>
    ),

    home: () => "Home",
    search: () => "Search",

    createPost: () => "Create Post",

    notFound: () => "Not Found",
  },
  errors: {
    Unknown: () => "An unknown error occurred",
    Unauthenticated: () => "You are not logged in",
    Unauthorized: () => "You are not authorised to do this",
    CredentialsInvalid: () => "Invalid login credentials",
    UnavailableIdent: () => "This username is unavailable",
    MissingIdent: () => "Handle cannot be empty",
    JwtMalformed: () => "The given JWT is malformed",
    JwtExpired: () => "The given JWT has expired",
    JwtInvalid: () => "The given JWT is invalid",
    WsInitNotObject: () =>
      "The WebSocket initialisation object is not an object",
    WsInitTokenNotString: () =>
      "The WebSocket initialisation token is not a string",
    ServerMisconfigured: () => "The server is misconfigured",
    InternalServerError: () => "An internal server error occurred",
    NotImplemented: () => "This feature is not implemented",
  },
  account: {
    userId: () => "Username",
    password: () => "Password",
    createTitle: () => "Create a new account",
    createSubmit: () => "Create Account",
    loginTitle: () => "Login",
    loginSubmit: () => "Login",
  },
} as const;
