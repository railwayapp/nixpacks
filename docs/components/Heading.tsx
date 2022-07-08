import Link from "next/link";
import React from "react";

export const Heading: React.FC<{
  id: string;
  level: number;
  className?: string;
  children: React.ReactNode;
}> = ({ id = "", level = 1, children, className }) => {
  return React.createElement(
    `h${level}`,
    {
      className: ["heading", "group", "max-w-max", className]
        .filter(Boolean)
        .join(" "),
    },
    <HeadingContent id={id}>{children}</HeadingContent>
  );
};

const HeadingContent: React.FC<{ id: string; children: React.ReactNode }> = ({
  id = "",
  children,
}) => {
  return (
    <>
      <span
        id={id}
        aria-hidden="true"
        className="absolute inline-block w-px heading-anchor"
        style={{
          marginTop: "calc(-1 * (var(--top-nav-height) + 2rem))",
        }}
      />
      <Link href={`#${id}`} passHref>
        <a
          className="no-underline hover:text-current"
          style={{ fontWeight: "inherit" }}
        >
          {children}

          <span className="ml-2 font-mono font-semibold text-gray-300 opacity-0 group-hover:opacity-100">
            #
          </span>
        </a>
      </Link>
    </>
  );
};
