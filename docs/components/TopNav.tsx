import { Dialog } from "@headlessui/react";
import Link from "next/link";
import { useRouter } from "next/router";
import React, { useEffect, useState } from "react";
import { Menu, X } from "react-feather";
import { GITHUB_URL } from "../constants";
import { GitHub } from "./icons/GitHub";
import { SidebarContent } from "./SideNav";

export const TopNav: React.FC<{
  coloured?: boolean;
  children?: React.ReactNode;
}> = ({ coloured, children }) => {
  const { pathname } = useRouter();
  const [isOpen, setIsOpen] = useState(false);

  const router = useRouter();
  useEffect(() => {
    const handleComplete = () => {
      setIsOpen(false);
    };

    router.events.on("routeChangeComplete", handleComplete);
    router.events.on("routeChangeError", handleComplete);

    return () => {
      router.events.off("routeChangeComplete", handleComplete);
      router.events.off("routeChangeError", handleComplete);
    };
  }, [router]);

  return (
    <div
      className={`sticky top-0 z-10 w-full ${
        coloured ? "bg-fuchsia-400" : "bg-bg shadow"
      }`}
    >
      <nav
        className={`flex items-center justify-between px-8 py-5 w-full max-w-[90rem] mx-auto`}
      >
        <Link href="/docs" className="flex">
          📦 Nixpacks
        </Link>

        <section className="flex gap-6 text-gray-500">
          <Link href="/docs/getting-started" passHref>
            <a
              className={`hidden md:block hover:text-fg ${
                pathname.startsWith("/docs") ? "text-fg" : ""
              }`}
            >
              Docs
            </a>
          </Link>

          <a
            href={GITHUB_URL}
            className="flex items-center space-x-2 hover:text-fg text-fg"
            target="_blank"
            rel="noopener noreferrer"
          >
            <GitHub className="w-5 h-5" />
          </a>

          <button
            onClick={() => setIsOpen(true)}
            className="block md:hidden text-fg"
          >
            {isOpen ? <X className="w-5 h-5" /> : <Menu className="w-5 h-5" />}
          </button>
        </section>
      </nav>

      {/* Mobile Nav */}
      <Dialog open={isOpen} onClose={() => setIsOpen(false)}>
        {/* The backdrop */}
        <div className="fixed inset-0 bg-black/30" aria-hidden="true" />

        <div className="fixed inset-0 bg-bg">
          <Dialog.Panel className="px-8 overflow-y-auto mt-[var(--top-nav-height)] py-4 max-h-[calc(100vh-var(--top-nav-height))]">
            <Dialog.Title className="hidden">Navigation</Dialog.Title>

            <SidebarContent className="w-full" />
          </Dialog.Panel>
        </div>
      </Dialog>
    </div>
  );
};
