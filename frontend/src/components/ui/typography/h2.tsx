import type React from "react";

export const H2 = ({
  children,
  className,
  ...props
}: React.ComponentProps<"h2">) => (
  <h2
    className={`scroll-m-20 text-2xl font-semibold tracking-tight first:mt-0 ${className ?? ""}`}
    {...props}
  >
    {children}
  </h2>
);
