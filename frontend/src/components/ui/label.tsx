import * as LabelPrimitive from "@radix-ui/react-label";
import * as React from "react";
import { useTranslation } from "react-i18next";

import { cn } from "@/lib/utils";

function Label({
  className,
  optional,
  children,
  ...props
}: React.ComponentProps<typeof LabelPrimitive.Root> & { optional?: boolean }) {
  const { t } = useTranslation();

  return (
    <LabelPrimitive.Root
      data-slot="label"
      className={cn(
        "flex items-center gap-2 text-sm leading-none font-medium select-none group-data-[disabled=true]:pointer-events-none group-data-[disabled=true]:opacity-50 peer-disabled:cursor-not-allowed peer-disabled:opacity-50",
        className,
      )}
      {...props}
    >
      {children}
      {optional && (
        <span className="text-gray-400">({t("common.optional")})</span>
      )}
    </LabelPrimitive.Root>
  );
}

export { Label };
