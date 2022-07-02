import NextLink from "next/link";
import React, { useMemo } from "react";

const isExternalLink = (href?: string): boolean =>
  href == null ||
  href.startsWith("http://") ||
  href.startsWith("https://") ||
  href.startsWith("//");

const useIsExternalLink = (href?: string) =>
  useMemo(() => isExternalLink(href), [href]);

export const ButtonLink: React.FC<{
  href: string;
  children?: React.ReactNode;
}> = ({ children, href, ...props }) => {
  const isExternal = useIsExternalLink(href);

  return (
    <NextLink href={href} passHref>
      <a
        {...props}
        className="px-4 py-2 bg-white rounded hover:bg-teal-500 hover:text-white"
        {...(isExternal && { target: "_blank" })}
      >
        {children}
      </a>
    </NextLink>
  );
};
