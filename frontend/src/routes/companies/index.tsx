import { createFileRoute } from "@tanstack/react-router";
import { Building2, Plus } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui";
import { H2 } from "@/components/ui/typography/h2";
import { CompanyModal } from "./components/CompanyModal";
import { Companies } from "./components/companies";

export const Route = createFileRoute("/companies/")({
  component: RouteComponent,
});

function RouteComponent() {
  const { t } = useTranslation();
  const [isAddModalOpen, setIsAddModalOpen] = useState(false);

  return (
    <>
      <div className="min-h-screen bg-gradient-to-br from-background via-background to-muted/20">
        <div className="container mx-auto">
          <div className="flex flex-col md:flex-row justify-between items-start md:items-center gap-4 mb-8 px-2">
            <div>
              <H2 className="text-3xl font-bold mb-2 flex items-center gap-2">
                <Building2 className="h-8 w-8" color="var(--primary)" />
                {t("companies.title")}
              </H2>
              <p className="text-muted-foreground">{t("companies.subtitle")}</p>
            </div>
            <Button
              onClick={() => setIsAddModalOpen(true)}
              className="flex items-center gap-2"
            >
              <Plus className="h-4 w-4" />
              {t("companies.addCompany")}
            </Button>
          </div>

          <Companies onAdd={() => setIsAddModalOpen(true)} />
        </div>
      </div>

      <CompanyModal open={isAddModalOpen} setIsOpen={setIsAddModalOpen} />
    </>
  );
}
