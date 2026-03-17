import { zodResolver } from "@hookform/resolvers/zod";
import { useState } from "react";
import { useForm } from "react-hook-form";
import { useTranslation } from "react-i18next";
import z from "zod";
import { APIHooks } from "@/api/hooks";
import { FormProvider } from "@/components/form/FormProvider";
import { FormSelect } from "@/components/form/FormSelect";
import {
  Button,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui";

interface Props {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

export const ExportAccountabilityModal = ({ open, onOpenChange }: Props) => {
  const { t } = useTranslation();
  const [showSuccessDialog, setShowSuccessDialog] = useState(false);
  const generateAccountabilityMutation =
    APIHooks.user.generateAccountability.useMutation();

  const exportAccountabilitySchema = z.object({
    year: z.coerce.number<number>().min(2025).max(2026),
  });

  const accountabilityExportForm = useForm({
    resolver: zodResolver(exportAccountabilitySchema),
    defaultValues: {
      year: 2026,
    },
  });

  const onSubmit = accountabilityExportForm.handleSubmit((values) => {
    generateAccountabilityMutation.mutateAsync(values).then(() => {
      onOpenChange(false);
      setShowSuccessDialog(true);
      accountabilityExportForm.reset();
    });
  });

  return (
    <>
      <Dialog open={open} onOpenChange={onOpenChange}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t("appointments.export.accountability.title")}</DialogTitle>
            <DialogDescription>
              {t("appointments.export.accountability.description")}
            </DialogDescription>
          </DialogHeader>

          <FormProvider
            methods={accountabilityExportForm}
            onSubmit={onSubmit}
            className="space-y-4"
          >
            <div className="flex gap-4">
              <FormSelect
                name="year"
                placeholder={t("appointments.export.accountability.yearPlaceholder")}
                options={[2025, 2026].map((year) => ({
                  label: year,
                  value: year,
                }))}
              />
            </div>

            <Button
              type="submit"
              disabled={accountabilityExportForm.formState.isSubmitting}
              className="w-full"
            >
              {t("appointments.export.submit")}
            </Button>
          </FormProvider>
        </DialogContent>
      </Dialog>

      <Dialog open={showSuccessDialog} onOpenChange={setShowSuccessDialog}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t("appointments.export.success.title")}</DialogTitle>
            <DialogDescription>
              {t("appointments.export.success.description")}
            </DialogDescription>
          </DialogHeader>

          <Button
            onClick={() => setShowSuccessDialog(false)}
            className="w-full"
          >
            {t("common.close")}
          </Button>
        </DialogContent>
      </Dialog>
    </>
  );
};
