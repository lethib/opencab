import { Trash2 } from "lucide-react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { queryClient } from "@/api/api";
import { APIHooks } from "@/api/hooks";
import type { CompanyIntervention } from "@/api/hooks/company_interventions";
import { Button } from "@/components/ui";
import { TableCell, TableRow } from "@/components/ui/table";
import { formatDate, formatPrice } from "@/lib/utils";

interface Props {
  intervention: CompanyIntervention;
  index: number;
}

export const InterventionRow = ({ intervention, index }: Props) => {
  const { t } = useTranslation();
  const deleteInterventionMutation = APIHooks.company.interventions
    .delete(intervention.company_id, intervention.id)
    .useMutation();

  const handleDelete = () => {
    deleteInterventionMutation.mutateAsync(null).then(() => {
      queryClient.invalidateQueries({
        queryKey: [`/companies/${intervention.company_id}/interventions`, null],
      });
      toast.success(t("companies.interventions.interventionDeleted"));
    });
  };

  return (
    <TableRow
      key={intervention.id}
      className={`transition-colors hover:bg-muted/30 ${
        index % 2 === 0 ? "bg-background" : "bg-muted/10"
      }`}
    >
      <TableCell className="px-6 py-4">
        <span className="font-medium">
          {formatDate(intervention.issue_date)}
        </span>
      </TableCell>
      <TableCell className="px-4 py-4 text-muted-foreground">
        {intervention.object}
      </TableCell>
      <TableCell className="px-4 py-4 text-muted-foreground text-center">
        {intervention.quantity}
      </TableCell>
      <TableCell className="px-4 py-4 text-muted-foreground font-mono text-center">
        {formatPrice(intervention.unit_price_in_cents)}
      </TableCell>
      <TableCell className="px-4 py-4 text-muted-foreground text-center">
        {intervention.vat_rate_in_percent} %
      </TableCell>
      <TableCell className="px-4 py-4 text-center">
        <span className="font-mono font-medium">
          {formatPrice(
            intervention.quantity * intervention.unit_price_in_cents,
          )}
        </span>
      </TableCell>
      <TableCell align="right" className="px-4 py-4">
        <Button
          variant="ghost_destructive"
          size="sm"
          className="h-8 w-8 p-0"
          onClick={(e) => {
            e.stopPropagation();
            handleDelete();
          }}
        >
          <Trash2 className="h-4 w-4" />
        </Button>
      </TableCell>
    </TableRow>
  );
};
