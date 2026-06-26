import { zodResolver } from "@hookform/resolvers/zod";
import { Navigate } from "@tanstack/react-router";
import { CreditCard, Landmark, User } from "lucide-react";
import { useEffect } from "react";
import { useForm } from "react-hook-form";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import z from "zod";
import { queryClient } from "@/api/api";
import { APIHooks } from "@/api/hooks";
import type { MeResponse } from "@/api/hooks/auth";
import { FormInput } from "@/components/form/FormInput";
import { FormProvider } from "@/components/form/FormProvider";
import {
  Button,
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
  Label,
} from "@/components/ui";

interface Props {
  currentUser: MeResponse;
}

const formatIBAN = (value: string) =>
  value
    .replace(/[^A-Z0-9]/gi, "")
    .toUpperCase()
    .match(/.{1,4}/g)
    ?.join(" ") ?? "";

const bankingInfoSchema = z.object({
  beneficiary_name: z.string().trim().min(1),
  iban: z
    .string()
    .transform((v) => v.replace(/\s/g, "").toUpperCase())
    .pipe(z.string().length(27)),
  bic: z
    .string()
    .transform((v) => v.trim().toUpperCase())
    .pipe(z.string().refine((v) => v.length === 8 || v.length === 11)),
});

export const BankingCard = ({ currentUser }: Props) => {
  const { t } = useTranslation();

  const bankingForm = useForm({
    resolver: zodResolver(bankingInfoSchema),
    defaultValues: {
      beneficiary_name: "",
      iban: "",
      bic: "",
    },
  });

  useEffect(() => {
    if (currentUser?.business_information) {
      bankingForm.reset({
        beneficiary_name:
          currentUser.business_information.beneficiary_name || "",
        iban: formatIBAN(currentUser.business_information.iban || ""),
        bic: currentUser.business_information.bic || "",
      });
    }
  }, [currentUser]);

  const saveBankingInfoMutation = APIHooks.user.saveBankingInfo.useMutation();

  if (!currentUser.business_information) {
    toast.error(t("myInformation.shouldCompleteBuinessInfoFirst"), {
      id: "signature-redirect",
    });
    return <Navigate to="/my_information" search={{ tab: "pro" }} />;
  }

  const onSubmit = bankingForm.handleSubmit(async (values) => {
    saveBankingInfoMutation.mutateAsync(values).then(() => {
      queryClient.invalidateQueries({ queryKey: ["/auth/me"] });
      toast.success("Vos informations bancaires ont bien été enregistrées.");
    });
  });

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Landmark className="h-5 w-5" />
          {t("bankingInfo.title")}
        </CardTitle>
        <CardDescription>{t("bankingInfo.subtitle")}</CardDescription>
      </CardHeader>
      <CardContent>
        <FormProvider
          methods={bankingForm}
          onSubmit={onSubmit}
          className="space-y-6"
        >
          <div className="space-y-2">
            <Label htmlFor="beneficiary" className="text-sm font-medium">
              {t("bankingInfo.beneficiary")}
            </Label>
            <FormInput
              id="beneficiary_name"
              name="beneficiary_name"
              type="text"
              placeholder={t("bankingInfo.beneficiaryPlaceholder")}
              className="pl-10 h-11"
              icon={
                <User className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
              }
            />
          </div>

          <div className="space-y-2">
            <Label htmlFor="iban" className="text-sm font-medium">
              {t("bankingInfo.iban")}
            </Label>
            <FormInput
              id="iban"
              name="iban"
              type="text"
              placeholder={t("bankingInfo.ibanPlaceholder")}
              className="pl-10 h-11 font-mono tracking-wide"
              onChange={(e) =>
                bankingForm.setValue("iban", formatIBAN(e.target.value))
              }
              icon={
                <CreditCard className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
              }
            />
          </div>

          <div className="space-y-2">
            <Label htmlFor="bic" className="text-sm font-medium">
              {t("bankingInfo.bic")}
            </Label>
            <FormInput
              id="bic"
              name="bic"
              type="text"
              placeholder={t("bankingInfo.bicPlaceholder")}
              className="pl-10 h-11 font-mono tracking-wide"
              icon={
                <Landmark className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
              }
            />
          </div>

          <Button type="submit" className="w-full">
            {t("bankingInfo.save")}
          </Button>
        </FormProvider>
      </CardContent>
    </Card>
  );
};
