import * as React from "react";
import { ButtonLink } from "./ButtonLink";
import { GitHub } from "./icons/GitHub";

export const Hero: React.FC = () => {
  return (
    <div
      className="not-prose hero bg-fuchsia-400 flex flex-col items-center pt-36 pb-[calc(9rem + var(--top-nav-height))]"
      style={{
        paddingBottom: "calc(10rem + var(--top-nav-height))",
      }}
    >
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
            <GitHub className="w-5 h-5" />
            <span>GitHub</span>
          </div>
        </ButtonLink>
      </div>
    </div>
  );
};
