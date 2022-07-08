import React, { useEffect, useMemo, useState } from "react";
import Link from "next/link";
import debounce from "lodash.debounce";
import { useIsMounted } from "../hooks/useIsMounted";
import { useRouter } from "next/router";
import { GITHUB_EDIT_URL } from "../constants";
import { ArrowRight } from "react-feather";

const useActiveHeaderId = () => {
  const [hashHeaderId, setHashHeaderId] = useState(() =>
    typeof window !== "undefined" && window.location.hash !== ""
      ? window.location.hash.replace("#", "")
      : null
  );

  const [activeHeaderId, setActiveHeaderId] = useState<string | null>(
    hashHeaderId
  );

  const onChangeHash = (id: string) => {
    setHashHeaderId(id);
    setActiveHeaderId(id);
  };

  const router = useRouter();
  useEffect(() => {
    const handleComplete = () => {
      setHashHeaderId(null);
      setActiveHeaderId(null);
    };

    router.events.on("routeChangeComplete", handleComplete);
    router.events.on("routeChangeError", handleComplete);

    return () => {
      router.events.off("routeChangeComplete", handleComplete);
      router.events.off("routeChangeError", handleComplete);
    };
  }, [router]);

  useEffect(() => {
    if (hashHeaderId != null) {
      setActiveHeaderId(hashHeaderId);
      return;
    }

    const handleScroll = debounce(() => {
      const headings = Array.from(
        document.querySelectorAll(
          "main h1.heading .heading-anchor,h2.heading .heading-anchor,h3.heading .heading-anchor"
        )
      );
      const visibleHeadings = headings.filter(
        (h) => h.getBoundingClientRect().top >= 10
      );

      const topHeading = visibleHeadings[0];
      if (topHeading != null) {
        setActiveHeaderId(topHeading.id);
      }
    }, 50);

    window.addEventListener("scroll", handleScroll);

    return () => {
      window.removeEventListener("scroll", handleScroll);
    };
  }, [hashHeaderId]);

  return { activeHeaderId, onChangeHash };
};

export const TableOfContents: React.FC<{
  toc: any;
  currentFile: string;
  className?: string;
}> = ({ toc, currentFile, className }) => {
  const items = toc.filter(
    (item: any) => item.id && (item.level === 2 || item.level === 3)
  );

  const { activeHeaderId, onChangeHash } = useActiveHeaderId();
  const editPageURL = `${GITHUB_EDIT_URL}${currentFile}`;

  // Don't render unless we are mounted so that we can use the window hash
  const isMounted = useIsMounted();
  if (!isMounted || items.length <= 1) {
    return null;
  }

  return (
    <nav className={`toc w-56 px-4 ${className}`}>
      <div
        className="sticky pt-20"
        style={{
          height: "calc(100vh - var(--top-nav-height) - 4px)",
          top: "calc(var(--top-nav-height) + 4px)",
        }}
      >
        <p className="mb-4 text-sm font-semibold">On this page</p>

        <ul className="space-y-2 font-mono text-sm">
          {items.map((item: any) => {
            const href = `#${item.id}`;
            const active = item.level !== 1 && activeHeaderId === item.id;

            return (
              <li key={item.title}>
                <Link href={href} passHref>
                  <a
                    onClick={() => onChangeHash(item.id)}
                    className={[
                      active
                        ? "text-fuchsia-700"
                        : "text-gray-500 hover:text-fg",
                      item.level === 1
                        ? "font-semibold"
                        : item.level === 2
                        ? "font-medium"
                        : "pl-6",
                    ]
                      .filter(Boolean)
                      .join(" ")}
                  >
                    {item.title}
                  </a>
                </Link>
              </li>
            );
          })}
        </ul>

        <hr className="my-8" />

        <a
          href={editPageURL}
          target="_blank"
          rel="noopener noreferrer"
          className="flex items-center text-xs text-gray-500 hover:text-fuchsia-700"
        >
          Edit this page on GitHub
          <ArrowRight size={14} className="ml-2 text-current" />
        </a>
      </div>
    </nav>
  );
};
