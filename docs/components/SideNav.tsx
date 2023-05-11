import Link from "next/link";
import { useRouter } from "next/router";
import React, { useState } from "react";
import { ChevronDown, ChevronRight } from "react-feather";
import { ISidebarSection, sidebarItems } from "../sidebar";

export const SideNav: React.FC<{ className?: string }> = ({ className }) => {
  return (
    <nav className={`sidenav text-sm px-4 w-[240px] ${className ?? ""}`}>
      <div
        className="sticky pt-8 pb-4 overflow-y-auto sidebar-container"
        style={{
          top: "calc(var(--top-nav-height) + 4px)",
        }}
      >
        <SidebarContent />
      </div>
      {/* using css fallback properties for cross-browser compatibility  */}
      {/* read more: https://modernweb.com/using-css-fallback-properties-for-better-cross-browser-compatibility/ */}
      <style jsx> {`
        .sidebar-container {
          height: calc(100vh - var(--top-nav-height) - 4px);
          height: calc(100dvh - var(--top-nav-height) - 4px);
        }
      `}
      </style>
    </nav>
  );
};

export const SidebarContent: React.FC<{ className?: string }> = ({
  className,
}) => {
  return (
    <ul className={`space-y-1 ${className ?? ""}`}>
      {sidebarItems.map((item) => (
        <SidebarSection key={item.text} section={item} />
      ))}
    </ul>
  );
};

const getSidebarItemStyles = (active?: boolean) =>
  `flex justify-between items-center cursor-pointer px-2 py-[6px] rounded ${
    active
      ? "bg-fuchsia-100 border-fuchsia-500 text-fuchsia-700 font-semibold"
      : "text-gray-500"
  } hover:bg-fuchsia-100 hover:text-fuchsia-700`;

const SidebarSection: React.FC<{
  section: ISidebarSection;
}> = ({ section }) => {
  const [isOpen, setIsOpen] = useState(true);

  return (
    <li>
      {section.href != null ? (
        <SidebarLink text={section.text} href={section.href} />
      ) : (
        <>
          <button
            className={`${getSidebarItemStyles()} flex items-center justify-between w-full mb-1`}
            onClick={() => setIsOpen(!isOpen)}
          >
            {section.text}

            {isOpen ? <ChevronDown size={16} /> : <ChevronRight size={16} />}
          </button>

          {isOpen && section.links != null && (
            <ul className="ml-4 space-y-1">
              {section.links.map((link) => (
                <li key={link.href}>
                  <SidebarLink text={link.text} href={link.href} />
                </li>
              ))}
            </ul>
          )}
        </>
      )}
    </li>
  );
};

const SidebarLink: React.FC<{
  text: string;
  href: string;
  hasSubLinks?: boolean;
}> = ({ text, href }) => {
  const { pathname } = useRouter();
  const active = href != null && pathname === href;

  return (
    <Link href={href} passHref>
      <a className={getSidebarItemStyles(active)}>{text}</a>
    </Link>
  );
};
