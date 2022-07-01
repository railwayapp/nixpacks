import NextLink from "next/link";
import { useMemo } from "react";

const isExternalLink = (href?: string): boolean =>
  href == null ||
  href.startsWith("http://") ||
  href.startsWith("https://") ||
  href.startsWith("//");

const useIsExternalLink = (href?: string) =>
  useMemo(() => isExternalLink(href), [href]);

export const Link = ({ children, href, ...props }) => {
  const isExternal = useIsExternalLink(href);

  return (
    <NextLink href={href} passHref>
      <a
        {...props}
        className="bg-white border-2 border-white px-4 py-2 rounded hover:bg-indigo-300 hover:border-white"
        {...(isExternal && { target: "_blank" })}
      >
        {children}
      </a>
    </NextLink>
  );
};
