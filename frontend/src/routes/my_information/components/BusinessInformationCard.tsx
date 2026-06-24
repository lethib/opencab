import { zodResolver } from "@hookform/resolvers/zod";
import { useNavigate } from "@tanstack/react-router";
import { Building2, FileText, Users } from "lucide-react";
import { useEffect } from "react";
import { useForm } from "react-hook-form";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import z from "zod";
import { queryClient } from "@/api/api";
import { APIHooks } from "@/api/hooks";
import { PROFESSIONS } from "@/api/types/profession";
import { FormInput } from "@/components/form/FormInput";
import { FormProvider } from "@/components/form/FormProvider";
import { FormSelect } from "@/components/form/FormSelect";
import {
  Button,
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
  Label,
} from "@/components/ui";
import { useCurrentUser } from "@/hooks/useCurrentUser";

export const BusinessInformationCard = () => {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const { currentUser } = useCurrentUser();

  const saveBusinessInformationMutation =
    APIHooks.user.saveBusinessInformation.useMutation();

  const businessInfoSchema = z.object({
    rpps_number: z.string().trim().length(11),
    siret_number: z.string().trim().length(14),
    adeli_number: z.string().trim().optional(),
    profession: z.enum(PROFESSIONS),
  });

  const businessForm = useForm({
    resolver: zodResolver(businessInfoSchema),
    defaultValues: {
      rpps_number: "",
      siret_number: "",
      adeli_number: "",
      profession: undefined,
    },
  });

  useEffect(() => {
    if (currentUser?.business_information) {
      businessForm.reset({
        rpps_number: currentUser.business_information.rpps_number || "",
        siret_number: currentUser.business_information.siret_number || "",
        adeli_number: currentUser.business_information.adeli_number || "",
        profession: currentUser.business_information.profession,
      });
    }
  }, [currentUser]);

  const onSubmit = businessForm.handleSubmit(async (values) => {
    saveBusinessInformationMutation.mutateAsync(values).then(() => {
      queryClient.invalidateQueries({ queryKey: ["/auth/me"] });
      toast.success(t("businessInfo.successMessage"));
      navigate({ to: "/patients" });
    });
  });
  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Building2 className="h-5 w-5" />
          {t("businessInfo.title")}
        </CardTitle>
        <CardDescription>{t("businessInfo.subtitle")}</CardDescription>
      </CardHeader>
      <CardContent>
        <FormProvider
          methods={businessForm}
          onSubmit={onSubmit}
          className="space-y-6"
        >
          <div className="space-y-2">
            <Label htmlFor="rpps_number" className="text-sm font-medium">
              {t("businessInfo.rppsNumber")} *
            </Label>
            <FormInput
              id="rpps_number"
              name="rpps_number"
              type="text"
              placeholder={t("businessInfo.rppsPlaceholder")}
              className="pl-10 h-11"
              icon={
                <Users className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
              }
            />
          </div>

          <div className="space-y-2">
            <Label htmlFor="siret_number" className="text-sm font-medium">
              {t("businessInfo.siretNumber")} *
            </Label>
            <FormInput
              id="siret_number"
              name="siret_number"
              type="text"
              placeholder={t("businessInfo.siretPlaceholder")}
              className="pl-10 h-11"
              icon={
                <Building2 className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
              }
            />
          </div>

          <div className="space-y-2">
            <Label htmlFor="adeli_number" className="text-sm font-medium">
              {t("businessInfo.adeliNumber")}
            </Label>
            <FormInput
              id="adeli_number"
              name="adeli_number"
              type="text"
              placeholder={t("businessInfo.adeliPlaceholder")}
              className="pl-10 h-11"
              icon={
                <FileText className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
              }
            />
          </div>

          <div className="space-y-2">
            <Label htmlFor="profession" className="text-sm font-medium">
              {t("businessInfo.profession")}
            </Label>
            <FormSelect
              name="profession"
              placeholder={t("businessInfo.professionPlaceholder")}
              options={
                PROFESSIONS.map((profession) => ({
                  value: profession,
                  label: t(`businessInfo.professionOptions.${profession}`),
                })) || []
              }
            />
          </div>

          <Button type="submit" className="w-full">
            {t("businessInfo.save")}
          </Button>
        </FormProvider>
      </CardContent>
    </Card>
  );
};
