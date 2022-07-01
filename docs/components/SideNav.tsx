import React, { useState } from "react";
import { useRouter } from "next/router";
import Link from "next/link";
import { sidebarItems, ISidebarSection } from "../sidebar";
import { Section } from "./Section";
import { ChevronDown, ChevronRight } from "react-feather";

export const SideNav = () => {
  const router = useRouter();

  return (
    <nav className="sidenav text-sm mt-20 px-4 w-[240px]">
      <div className="sticky top-[calc(5rem+var(--top-nav-height)+4px)]">
        <ul className="space-y-1">
          {sidebarItems.map((item) => (
            <SidebarSection key={item.text} section={item} />
          ))}
        </ul>
      </div>
    </nav>
  );
};

const getSidebarItemStyles = (active?: boolean) =>
  `flex justify-between items-center cursor-pointer px-2 py-[6px] rounded ${
    active
      ? "bg-indigo-100 border-indigo-500 text-indigo-700 font-semibold"
      : "text-gray-500"
  } hover:bg-indigo-100`;

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
            <ul className="space-y-1 ml-4">
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
