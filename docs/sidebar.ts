export interface SidebarSection {
  title: string;
  href: string;
  links?: SidebarLink[];
}

export interface SidebarLink {
  href: string;
  text: string;
}

export const sidebarItems: SidebarSection[] = [
  {
    title: "Overview",
    href: "/docs",
    links: [
      { href: "/docs/getting-started", text: "Get Started" },
      { href: "/docs/install", text: "Installation" },
    ],
  },
  {
    title: "Concepts",
    href: "/docs/concepts",
  },
  {
    title: "Configuration",
    href: "/docs/config",
  },
  {
    title: "Language Support",
    href: "/docs/providers",
    links: [
      { href: "/docs/providers/node", text: "Node" },
      { href: "/docs/providers/crystal", text: "Crystal" },
      { href: "/docs/providers/csharp", text: "C#" },
      { href: "/docs/providers/dart", text: "Dart" },
      { href: "/docs/providers/deno", text: "Deno" },
      { href: "/docs/providers/go", text: "Go" },
      { href: "/docs/providers/haskell", text: "Haskell" },
      { href: "/docs/providers/java", text: "Java" },
      { href: "/docs/providers/php", text: "PHP" },
      { href: "/docs/providers/python", text: "Python" },
      { href: "/docs/providers/ruby", text: "Ruby" },
      { href: "/docs/providers/rust", text: "Rust" },
      { href: "/docs/providers/staticfile", text: "Staticfile" },
      { href: "/docs/providers/swift", text: "Swift" },
      { href: "/docs/providers/zig", text: "Zig" },
    ],
  },
  {
    title: "CLI Reference",
    href: "/docs/cli",
  },
  {
    title: "Railway",
    href: "/docs/railway",
  },
];
