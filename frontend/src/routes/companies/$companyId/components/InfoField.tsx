export const InfoField = ({
  label,
  value,
}: {
  label: string;
  value: string | null | undefined;
}) => (
  <div className="space-y-1">
    <p className="text-xs font-medium tracking-widest text-muted-foreground uppercase">
      {label}
    </p>
    <p className="text-sm font-mono">{value ?? "-"}</p>
  </div>
);
