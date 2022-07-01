import * as React from "react";
import { GitHub } from "react-feather";
import { ButtonLink } from "./ButtonLink";

export const Hero: React.FC = () => {
  return (
    <div className="not-prose hero bg-fuchsia-400 flex flex-col items-center pt-36 pb-[calc(9rem + var(--top-nav-height))]">
      <img
        src="/box.svg"
        alt="Nixpacks Logo"
        className="w-40 h-40 mb-16 logo"
      />

      <h1 className="mb-6 font-bold text-7xl">Nixpacks</h1>
      <h2 className="text-2xl font-semibold">
        App source + Nix packages + Docker = Image
      </h2>

      <div className="flex gap-4 mt-12 actions">
        <ButtonLink href="/docs">Get Started</ButtonLink>
        <ButtonLink href="https://github.com/railwayapp/nixpacks">
          <div className="flex items-center space-x-2">
            <img src="/icons/github.svg" alt="" className="w-5" />{" "}
            <span>GitHub</span>
          </div>
        </ButtonLink>
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
