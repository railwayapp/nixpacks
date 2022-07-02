import React from "react";

export const Section: React.FC<{ children?: React.ReactNode }> = ({
  children,
}) => {
  return <div className="section">{children}</div>;
};
