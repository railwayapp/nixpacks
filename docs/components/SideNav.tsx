import React from "react";
import { useRouter } from "next/router";
import Link from "next/link";
import { sidebarItems } from "../sidebar";

export const SideNav = () => {
  const router = useRouter();

  return (
    <nav className="sidenav text-sm bg-gray-50 border-r border-none sticky py-8 w-[300px]">
      {sidebarItems.map((item) => {
        const active = item.href != null && router.pathname === item.href;

        return (
          <div key={item.title} className="mb-2">
            <p className="font-semibold">
              <Link href={item.href} passHref>
                <a
                  className={`block font-semibold border-r-2 cursor-pointer px-4 py-2 ${
                    active
                      ? "bg-indigo-100 border-indigo-500 text-indigo-600"
                      : "border-transparent"
                  } hover:bg-indigo-100`}
                >
                  {item.title}
                </a>
              </Link>
            </p>

            {item.links != null && (
              <ul className="flex flex-col">
                {item.links.map((link) => {
                  const active = router.pathname === link.href;
                  return (
                    <li key={link.href}>
                      <Link {...link} passHref>
                        <a
                          className={`block text-gray-500 px-4 py-2 border-r-2 cursor-pointer ${
                            active
                              ? "bg-indigo-100 border-indigo-500 text-indigo-600"
                              : "border-transparent"
                          } hover:bg-indigo-100`}
                        >
                          {link.text}
                        </a>
                      </Link>
                    </li>
                  );
                })}
              </ul>
            )}
          </div>
        );
      })}
    </nav>
  );
};
