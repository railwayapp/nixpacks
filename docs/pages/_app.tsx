import type { AppProps } from "next/app";
import Head from "next/head";
import { useRouter } from "next/router";
import "prismjs";
import { Hero, SideNav, TableOfContents, TopNav } from "../components";

// Import other Prism themes here
import "prismjs/components/prism-bash.min";
import "prismjs/components/prism-toml.min";
import "prismjs/components/prism-json.min";
import "../public/prism-one-light.css";

import "../public/globals.css";

const TITLE = "Nixpacks";
const DESCRIPTION = "App source + Nix packages + Docker = Image";

function collectHeadings(node: any, sections: any[] = []) {
  if (node) {
    if (node.name === "Heading") {
      const title = node.children[0];

      if (typeof title === "string") {
        sections.push({
          ...node.attributes,
          // id,
          title,
        });
      }
    }

    if (node.children) {
      for (const child of node.children) {
        collectHeadings(child, sections);
      }
    }
  }

  return sections;
}

export default function MyApp({ Component, pageProps }: AppProps) {
  const { markdoc } = pageProps;
  const { pathname } = useRouter();
  const isHome = pathname === "/";

  let title = TITLE;
  let description = DESCRIPTION;

  if (markdoc) {
    if (markdoc.frontmatter.title) {
      title = markdoc.frontmatter.title;
    }
    if (markdoc.frontmatter.description) {
      description = markdoc.frontmatter.description;
    }
  }

  if (!isHome) {
    title = `${title} | Nixpacks`;
  }

  const currentFile = markdoc?.file.path;

  const toc = pageProps.markdoc?.content
    ? collectHeadings(pageProps.markdoc.content)
    : [];

  return (
    <>
      <Head>
        <title>{title}</title>
        <meta name="viewport" content="width=device-width, initial-scale=1.0" />
        <meta name="referrer" content="strict-origin" />
        <meta name="title" content={title} />
        <meta name="description" content={description} />

        <link rel="shortcut icon" href="/favicon.ico" />
        <link rel="icon" href="/favicon.ico" />
      </Head>

      <div className={`app grid gap-0 grid-rows-[auto_1fr] min-h-screen`}>
        <TopNav />

        <div className={`page`}>
          {isHome && <Hero />}

          <div
            className={`max-w-[90rem] mx-auto grid gap-8 ${
              !isHome
                ? "grid-cols-1 md:grid-cols-[auto_minmax(0px,1fr)] lg:grid-cols-[auto_minmax(0px,1fr)_auto]"
                : ""
            }`}
          >
            {!isHome && <SideNav className="hidden md:block" />}

            <main
              className={`prose w-full max-w-4xl px-8 py-12 md:pt-20 md:pb-40 ${
                isHome ? "prose-lg mx-auto" : ""
              }`}
            >
              <Component {...pageProps} />
            </main>

            {!isHome && (
              <TableOfContents
                toc={toc}
                className="hidden lg:block"
                currentFile={currentFile}
              />
            )}
          </div>
        </div>
      </div>
    </>
  );
}
