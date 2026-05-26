import { getInitials } from "@/lib/utils";

interface Props {
  name: string;
  size?: "sm" | "lg";
}

const sizeClasses = {
  sm: { wrapper: "w-10 h-10 rounded-lg", text: "text-sm font-semibold" },
  lg: { wrapper: "w-16 h-16 rounded-xl", text: "text-xl font-bold" },
};

export const CompanyAvatar = ({ name, size = "sm" }: Props) => {
  const { wrapper, text } = sizeClasses[size];
  return (
    <div
      className={`flex-shrink-0 ${wrapper} bg-primary/15 flex items-center justify-center`}
    >
      <span className={`${text} text-primary`}>{getInitials(name)}</span>
    </div>
  );
};
