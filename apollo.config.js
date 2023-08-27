module.exports = {
  client: {
    service: {
      name: "plazer",
      localSchemaFile: "./schema.gql",
    },
    includes: ["./packages/ui/src/**/*.{tsx,ts}"],
  },
};
