import { Building2, Calendar, Euro, Hash, Percent, Tag } from "lucide-react";
import { useTranslation } from "react-i18next";
import { APIHooks } from "@/api/hooks";
import type { Company } from "@/api/hooks/practitioner_company";
import { CenteredSpineer } from "@/components/ui/spinner";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { H2 } from "@/components/ui/typography/h2";
import { InterventionRow } from "./InterventionRow";

interface Props {
  company: Company;
}

export const CompanyInterventionsSection = ({ company }: Props) => {
  const { t } = useTranslation();

  const interventionsQuery = APIHooks.company.interventions
    .list(company.id)
    .useQuery(null);
  const interventions = interventionsQuery.data ?? [];

  return (
    <div className="space-y-4 px-2">
      <div className="flex justify-between items-center">
        <div>
          <H2 className="text-2xl font-bold flex items-center gap-2">
            <Building2 className="h-6 w-6" />
            {t("companies.interventions.title")}
          </H2>
          <p className="text-muted-foreground text-sm mt-1">
            {t("companies.interventions.subtitle", { name: company.name })}
          </p>
        </div>
      </div>

      <div className="rounded-lg border bg-card overflow-hidden">
        {interventionsQuery.isLoading ? (
          <CenteredSpineer />
        ) : (
          <Table>
            <TableHeader>
              <TableRow className="border-b bg-muted/50">
                <TableHead className="h-12 px-6 font-semibold text-foreground">
                  <div className="flex items-center gap-2">
                    <Calendar className="h-4 w-4" />
                    {t("companies.interventions.date")}
                  </div>
                </TableHead>
                <TableHead className="h-12 px-4 font-semibold text-foreground">
                  <div className="flex items-center gap-2">
                    <Tag className="h-4 w-4" />
                    {t("companies.interventions.description")}
                  </div>
                </TableHead>
                <TableHead className="h-12 px-4 font-semibold text-foreground text-center">
                  <div className="flex items-center justify-center gap-2">
                    <Hash className="h-4 w-4" />
                    {t("companies.interventions.quantity")}
                  </div>
                </TableHead>
                <TableHead className="h-12 px-4 font-semibold text-foreground text-center">
                  <div className="flex items-center justify-center gap-2">
                    <Euro className="h-4 w-4" />
                    {t("companies.interventions.unitPrice")}
                  </div>
                </TableHead>
                <TableHead className="h-12 px-4 font-semibold text-foreground text-center">
                  <div className="flex items-center justify-center gap-2">
                    <Percent className="h-4 w-4" />
                    {t("companies.interventions.vat")}
                  </div>
                </TableHead>
                <TableHead className="h-12 px-4 font-semibold text-foreground text-center">
                  <div className="flex items-center justify-center gap-2">
                    <Euro className="h-4 w-4" />
                    {t("companies.interventions.amount")}
                  </div>
                </TableHead>
                <TableHead className="h-12 px-4" />
              </TableRow>
            </TableHeader>

            <TableBody>
              {interventions.length === 0 ? (
                <TableRow>
                  <TableCell
                    colSpan={7}
                    className="h-32 text-center text-muted-foreground text-sm"
                  >
                    {t("companies.interventions.empty")}
                  </TableCell>
                </TableRow>
              ) : (
                interventions.map((intervention, index) => (
                  <InterventionRow
                    key={intervention.id}
                    intervention={intervention}
                    index={index}
                  />
                ))
              )}
            </TableBody>
          </Table>
        )}
      </div>
    </div>
  );
};
