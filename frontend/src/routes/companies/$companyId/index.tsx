import {
  createFileRoute,
  useCanGoBack,
  useNavigate,
  useRouter,
} from "@tanstack/react-router";
import { ArrowLeft, FileText, Pencil } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { APIHooks } from "@/api/hooks";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { CenteredSpineer } from "@/components/ui/spinner";
import { formatAddress } from "@/lib/utils";
import { CompanyAvatar } from "../components/CompanyAvatar";
import { CompanyModal } from "../components/CompanyModal";
import { CompanyInterventionsSection } from "./components/CompanyInterventionsSection";
import { GenerateInvoiceModal } from "./components/GenerateInvoiceModal";
import { InfoField } from "./components/InfoField";

export const Route = createFileRoute("/companies/$companyId/")({
  component: CompanyPage,
});

function CompanyPage() {
  const { t } = useTranslation();
  const router = useRouter();
  const canGoBack = useCanGoBack();
  const navigate = useNavigate();
  const { companyId } = Route.useParams();

  const [isEditModalOpen, setIsEditModalOpen] = useState(false);
  const [isInvoiceModalOpen, setIsInvoiceModalOpen] = useState(false);

  const companyQuery = APIHooks.company.get(+companyId).useQuery(null);
  const company = companyQuery.data;

  const handleBackNavigation = () => {
    if (canGoBack) {
      router.history.back();
    } else {
      navigate({ to: "/companies" });
    }
  };

  if (companyQuery.isLoading) {
    return <CenteredSpineer />;
  }

  if (!company) {
    return (
      <div className="container mx-auto py-8">
        <Button
          variant="link"
          onClick={handleBackNavigation}
          className="flex items-center gap-2 mb-6 px-0"
        >
          <ArrowLeft className="h-4 w-4" />
          {t("common.backToCompanies")}
        </Button>
        <p className="text-muted-foreground">
          {t("companies.form.show.notFound")}
        </p>
      </div>
    );
  }

  const formattedAddress = formatAddress(company);

  return (
    <>
      <div className="min-h-screen bg-linear-to-br from-background via-background to-muted/20">
        <div className="container mx-auto space-y-6">
          <Button
            variant="link"
            onClick={handleBackNavigation}
            className="flex items-center gap-2 px-0"
          >
            <ArrowLeft className="h-4 w-4" />
            {t("common.backToCompanies")}
          </Button>

          <Card>
            <CardContent className="pt-4 space-y-8">
              <div className="flex items-center justify-between gap-4">
                <div className="flex items-center gap-4">
                  <CompanyAvatar name={company.name} size="lg" />
                  <div>
                    <h1 className="text-2xl font-bold leading-tight">
                      {company.name}
                    </h1>
                  </div>
                </div>

                <div className="flex items-center gap-3 flex-shrink-0">
                  <Button
                    variant="outline"
                    onClick={() => setIsEditModalOpen(true)}
                    className="flex items-center gap-2"
                  >
                    <Pencil className="h-4 w-4" />
                    {t("companies.form.show.edit")}
                  </Button>
                  <Button
                    onClick={() => setIsInvoiceModalOpen(true)}
                    className="flex items-center gap-2"
                  >
                    <FileText className="h-4 w-4" />
                    {t("companies.form.show.generateInvoice")}
                  </Button>
                </div>
              </div>

              <div className="grid grid-cols-1 md:grid-cols-2 gap-x-16 gap-y-6 border-t pt-6">
                <InfoField
                  label={t("companies.form.show.billingContact")}
                  value={company.contact_name}
                />
                <InfoField
                  label={t("companies.form.show.siret")}
                  value={company.siret}
                />
                <InfoField
                  label={t("companies.form.show.billingEmail")}
                  value={company.contact_email}
                />
                <InfoField
                  label={t("companies.form.show.billingAddress")}
                  value={formattedAddress || null}
                />
              </div>
            </CardContent>
          </Card>

          <CompanyInterventionsSection company={company} />
        </div>
      </div>

      <CompanyModal
        open={isEditModalOpen}
        setIsOpen={setIsEditModalOpen}
        company={company}
      />

      <GenerateInvoiceModal
        open={isInvoiceModalOpen}
        setIsOpen={setIsInvoiceModalOpen}
        company={company}
      />
    </>
  );
}
