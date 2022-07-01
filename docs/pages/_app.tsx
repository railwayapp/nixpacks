import React from "react";
import Head from "next/head";
import Link from "next/link";

import { Hero, SideNav, TableOfContents, TopNav } from "../components";

import "prismjs";
// Import other Prism themes here
import "prismjs/components/prism-bash.min";
import "prismjs/themes/prism.css";

import "../public/globals.css";

import type { AppProps } from "next/app";
import { useRouter } from "next/router";

const TITLE = "Nixpacks";
const DESCRIPTION = "App source + Nix packages + Docker = Image";

function collectHeadings(node: any, sections: any[] = []) {
  if (node) {
    if (node.name === "Heading") {
      const title = node.children[0];

      if (typeof title === "string") {
        sections.push({
          ...node.attributes,
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
        <TopNav coloured={isHome}>
          <Link href="/docs" passHref>
            <a className="hover:underline">Docs</a>
          </Link>
        </TopNav>

        <div className={`page`}>
          {isHome && <Hero />}

          <div
            className={`max-w-[90rem] mx-auto grid gap-8 ${
              !isHome ? "grid-cols-[auto_minmax(0px,1fr)]" : ""
            }`}
          >
            {!isHome && <SideNav />}

            <main
              className={`prose w-full max-w-4xl px-8 pt-20 pb-40 ${
                isHome ? "prose-lg mx-auto" : ""
              }`}
            >
              <Component {...pageProps} />
            </main>

            {/* {!isHome && <TableOfContents toc={toc} />} */}
          </div>
        </div>
      </div>
    </>
  );
}
