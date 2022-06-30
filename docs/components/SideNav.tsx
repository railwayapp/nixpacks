import React from "react";
import { useRouter } from "next/router";
import Link from "next/link";
import { sidebarItems } from "../sidebar";

export const SideNav = () => {
  const router = useRouter();

  return (
    <nav className="sidenav bg-amber-50 border-r border-amber-100 sticky py-8 w-[300px]">
      {sidebarItems.map((item) => {
        const active = item.href != null && router.pathname === item.href;

        return (
          <div key={item.title} className="mb-3">
            <p className="font-semibold">
              <Link href={item.href} passHref>
                <a
                  className={`block font-semibold border-r-2 cursor-pointer px-4 py-2 ${
                    active
                      ? "bg-amber-100 border-amber-500"
                      : "border-transparent"
                  } hover:bg-amber-200`}
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
                          className={`block px-4 py-2 border-r-2 cursor-pointer ${
                            active
                              ? "bg-amber-100 border-amber-500"
                              : "border-transparent"
                          } hover:bg-amber-200`}
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
