import * as React from "react";

export const Heading: React.FC<{
  id: string;
  level: number;
  className?: string;
  children: React.ReactNode;
}> = ({ id = "", level = 1, children, className }) => {
  return React.createElement(
    `h${level}`,
    {
      id,
      className: ["heading", className].filter(Boolean).join(" "),
    },
    children
  );
};
