import React from "react";
import Link from "next/link";

export const TopNav: React.FC<{
  coloured?: boolean;
  children?: React.ReactNode;
}> = ({ coloured, children }) => {
  return (
    <div
      className={`sticky top-0 z-10 w-full ${
        coloured ? "bg-fuchsia-400" : "bg-bg shadow"
      }`}
    >
      <nav
        className={`flex items-center justify-between px-8 py-5 w-full max-w-[90rem] mx-auto`}
      >
        <Link href="/" className="flex">
          📦 Nixpacks
        </Link>

        <section className="flex gap-4">{children}</section>
      </nav>
    </div>
  );
};
