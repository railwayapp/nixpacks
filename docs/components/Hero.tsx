import * as React from "react";
import { Link } from "./Link";

export const Hero: React.FC = () => {
  return (
    <div className="not-prose hero bg-teal-200 flex flex-col items-center pt-40 pb-[calc(10rem + var(--top-nav-height))]">
      <img
        src="/box.svg"
        alt="Nixpacks Logo"
        className="logo w-40 h-40 mb-16"
      />

      <h1 className="mb-6 text-7xl font-bold">Nixpacks</h1>
      <h2 className="text-2xl font-semibold">
        App source + Nix packages + Docker = Image
      </h2>

      <div className="actions mt-12 flex gap-4">
        <Link href="/docs">Get Started</Link>
        <Link href="https://github.com/railwayapp/nixpacks">GitHub</Link>
      </div>

      <style jsx>
        {`
          .hero {
            padding-bottom: calc(10rem + var(--top-nav-height));
          }
        `}
      </style>
    </div>
  );
};
