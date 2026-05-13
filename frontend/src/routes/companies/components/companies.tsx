import { Building2, Layers, Plus, Send } from "lucide-react";
import { useTranslation } from "react-i18next";
import { APIHooks } from "@/api/hooks";
import { Button } from "@/components/ui/button";
import { CenteredSpineer } from "@/components/ui/spinner";
import { CompanyCard } from "./CompanyCard";

const EmptyState = ({ onAdd }: { onAdd: VoidFunction }) => {
  const { t } = useTranslation();

  const features = [
    {
      icon: Layers,
      title: t("companies.noCompanies.features.multipleLines.title"),
      description: t(
        "companies.noCompanies.features.multipleLines.description",
      ),
    },
    {
      icon: Send,
      title: t("companies.noCompanies.features.directSend.title"),
      description: t("companies.noCompanies.features.directSend.description"),
    },
  ];

  return (
    <div className="border rounded-xl px-8 py-16 flex flex-col items-center text-center">
      <div className="w-20 h-20 rounded-2xl bg-primary/10 flex items-center justify-center mb-6">
        <Building2 className="h-9 w-9 text-primary" />
      </div>

      <h3 className="text-xl font-semibold mb-3">
        {t("companies.noCompanies.title")}
      </h3>
      <p className="text-muted-foreground max-w-md mb-10">
        {t("companies.noCompanies.description")}
      </p>

      <div className="grid grid-cols-1 sm:grid-cols-2 gap-4 w-full max-w-xl mb-10">
        {features.map(({ icon: Icon, title, description }) => (
          <div key={title} className="border rounded-xl p-4 text-left bg-card">
            <div className="w-8 h-8 rounded-lg bg-primary/10 flex items-center justify-center mb-3">
              <Icon className="h-4 w-4 text-primary" />
            </div>
            <p className="font-semibold text-sm mb-1">{title}</p>
            <p className="text-muted-foreground text-sm">{description}</p>
          </div>
        ))}
      </div>

      <Button onClick={onAdd} className="flex items-center gap-2">
        <Plus className="h-4 w-4" />
        {t("companies.noCompanies.cta")}
      </Button>
    </div>
  );
};

export const Companies = ({ onAdd }: { onAdd: VoidFunction }) => {
  const companiesQuery = APIHooks.company.list.useQuery(null);

  if (companiesQuery.isLoading) return <CenteredSpineer />;

  if (!companiesQuery.isError && companiesQuery.data?.length === 0) {
    return <EmptyState onAdd={onAdd} />;
  }

  return (
    <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
      {companiesQuery.data?.map((company) => (
        <CompanyCard key={company.id} company={company} />
      ))}
    </div>
  );
};
