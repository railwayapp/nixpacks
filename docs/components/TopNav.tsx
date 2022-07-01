import React from "react";
import Link from "next/link";
import { useRouter } from "next/router";

export const TopNav: React.FC<{
  coloured?: boolean;
  children?: React.ReactNode;
}> = ({ coloured, children }) => {
  return (
    <nav
      className={`flex items-center justify-between px-8 py-4 ${
        coloured ? "bg-indigo-300" : "border-b border-gray-100"
      }`}
    >
      <Link href="/" className="flex">
        Nixpacks
      </Link>

      <section className="flex gap-4">{children}</section>

      {/* <style jsx>
        {`
          nav {
            top: 0;
            width: 100%;
            display: flex;
            align-items: center;
            justify-content: space-between;
            gap: 1rem;
            padding: 1rem 2rem;
            border-bottom: 1px solid var(--border-color);
            background-color: var(--primary);
          }
          nav :global(a) {
            text-decoration: none;
          }
          section {
            display: flex;
            gap: 1rem;
            padding: 0;
          }
        `}
      </style> */}
    </nav>
  );
};
