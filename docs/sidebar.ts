export type ISidebarSection = {
  text: string;
  href?: string;
  links?: ISidebarItem[];
};

export type ISidebarItem = { text: string; href: string };

export const sidebarItems: ISidebarSection[] = [
  { href: "/docs", text: "Introduction" },
  { href: "/docs/getting-started", text: "Getting Started" },
  { href: "/docs/install", text: "Installation" },
  { href: "/docs/how-it-works", text: "How it Works" },
  {
    text: "Guides",
    links: [
      {
        text: "Configuring Builds",
        href: "/docs/guides/configuring-builds",
      },
    ],
  },
  {
    text: "Configuration",
    links: [
      { text: "File", href: "/docs/configuration/file" },
      {
        text: "Environment",
        href: "/docs/configuration/environment",
      },
      { text: "Procfile", href: "/docs/configuration/procfile" },
      { text: "Caching", href: "/docs/configuration/caching" },
    ],
  },
  { text: "CLI Reference", href: "/docs/cli" },
  {
    text: "Language Support",
    links: [
      { href: "/docs/providers/clojure", text: "Clojure" },
      { href: "/docs/providers/cobol", text: "Cobol" },
      { href: "/docs/providers/crystal", text: "Crystal" },
      { href: "/docs/providers/csharp", text: "C#/.NET" },
      { href: "/docs/providers/dart", text: "Dart" },
      { href: "/docs/providers/deno", text: "Deno" },
      { href: "/docs/providers/elixir", text: "Elixir" },
      { href: "/docs/providers/fsharp", text: "F#" },
      { href: "/docs/providers/gleam", text: "Gleam" },
      { href: "/docs/providers/go", text: "Go" },
      { href: "/docs/providers/haskell", text: "Haskell" },
      { href: "/docs/providers/java", text: "Java" },
      { href: "/docs/providers/lunatic", text: "Lunatic" },
      { href: "/docs/providers/node", text: "Node" },
      { href: "/docs/providers/php", text: "PHP" },
      { href: "/docs/providers/python", text: "Python" },
      { href: "/docs/providers/ruby", text: "Ruby" },
      { href: "/docs/providers/rust", text: "Rust" },
      { href: "/docs/providers/scheme", text: "Scheme" },
      { href: "/docs/providers/staticfile", text: "Staticfile" },
      { href: "/docs/providers/swift", text: "Swift" },
      { href: "/docs/providers/scala", text: "Scala" },
      { href: "/docs/providers/zig-lang", text: "Zig" },
    ],
  },
  {
    text: "Deploying",
    links: [
      { text: "Railway", href: "/docs/deploying/railway" },
      { text: "Coherence", href: "/docs/deploying/coherence" },
      { text: "Coolify", href: "/docs/deploying/coolify" },
      { text: "Dokku", href: "/docs/deploying/dokku" },
      { text: "Dokploy", href: "/docs/deploying/dokploy" },
      { text: "Easypanel", href: "/docs/deploying/easypanel" },
      { text: "Flightcontrol", href: "/docs/deploying/flightcontrol" },
      { text: "GitHub Actions", href: "/docs/deploying/github-actions" },
      { text: "PipeOps", href: "/docs/deploying/pipeops" },
      { text: "Stacktape", href: "/docs/deploying/stacktape" },
    ],
  },
];
