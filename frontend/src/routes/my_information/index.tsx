import { createFileRoute } from "@tanstack/react-router";
import { Building2, Landmark, PenTool } from "lucide-react";
import { useTranslation } from "react-i18next";
import z from "zod";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui";
import { useCurrentUser } from "@/hooks/useCurrentUser";
import { BankingCard } from "./components/BankingCard";
import { BusinessInformationCard } from "./components/BusinessInformationCard";
import { SignatureCard } from "./components/SignatureCard";

const tabSchema = z.object({
  tab: z.enum(["pro", "bank", "signature"]).default("pro"),
});

export const Route = createFileRoute("/my_information/")({
  component: MyInformation,
  validateSearch: tabSchema,
});

function MyInformation() {
  const { t } = useTranslation();
  const { tab } = Route.useSearch();
  const navigate = Route.useNavigate();
  const { currentUser } = useCurrentUser();

  const mustFillProInformation = !currentUser?.business_information;

  if (!currentUser) return;

  return (
    <div className="container mx-auto p-6 max-w-4xl">
      <div className="mb-6">
        <h1 className="text-2xl font-semibold tracking-tight">
          {t("myInformation.title")}
        </h1>
        <p className="mt-1 text-sm text-muted-foreground">
          {t("myInformation.subtitle")}
        </p>
      </div>
      <Tabs
        value={tab}
        onValueChange={(value) =>
          navigate({ search: { tab: value as typeof tab } })
        }
      >
        <TabsList className="mb-6 w-full">
          <TabsTrigger value="pro" className="flex-1 gap-2">
            <Building2 className="h-4 w-4" />
            {t("myInformation.tabs.pro")}
          </TabsTrigger>
          <TabsTrigger
            value="bank"
            className="flex-1 gap-2"
            disabled={mustFillProInformation}
          >
            <Landmark className="h-4 w-4" />
            {t("myInformation.tabs.bank")}
          </TabsTrigger>
          <TabsTrigger
            value="signature"
            className="flex-1 gap-2"
            disabled={mustFillProInformation}
          >
            <PenTool className="h-4 w-4" />
            {t("myInformation.tabs.signature")}
          </TabsTrigger>
        </TabsList>
        <TabsContent value="pro">
          <BusinessInformationCard currentUser={currentUser} />
        </TabsContent>
        <TabsContent value="bank">
          <BankingCard currentUser={currentUser} />
        </TabsContent>
        <TabsContent value="signature">
          <SignatureCard currentUser={currentUser} />
        </TabsContent>
      </Tabs>
    </div>
  );
}
