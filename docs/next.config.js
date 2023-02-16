const withMarkdoc = require("@markdoc/next.js");

module.exports =
  withMarkdoc(/* config: https://markdoc.io/docs/nextjs#options */)({
    pageExtensions: ["js", "jsx", "ts", "tsx", "md", "mdoc"],

    async redirects() {
      return [
        { source: "/", destination: "/docs/getting-started", permanent: false },
        {
          source: "/install.sh",
          destination:
            "https://raw.githubusercontent.com/railwayapp/nixpacks/main/install.sh",
            permanent: false,
        },
        {
            source: "/install.ps1",
            destination: "https://raw.githubusercontent.com/railwayapp/nixpacks/main/install.ps1",
            permanent: false,
        }
      ];
    },
  });
