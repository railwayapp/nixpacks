import Prism from "prismjs";
import React from "react";

export const CodeBlock: React.FC<{
  language?: string;
  children?: React.ReactNode;
}> = ({ children, language }) => {
  const ref = React.useRef(null);

  React.useEffect(() => {
    if (ref.current) Prism.highlightElement(ref.current, false);
  }, [children]);

  return (
    <div className="relative rounded code" aria-live="polite">
      <pre ref={ref} className={`language-${language}`}>
        {children}
      </pre>
    </div>
  );
};
