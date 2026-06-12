import { zodResolver } from "@hookform/resolvers/zod";
import { useNavigate } from "@tanstack/react-router";
import { CircleAlert, FileText, HandCoins, Loader2 } from "lucide-react";
import { useEffect } from "react";
import { useForm } from "react-hook-form";
import { useTranslation } from "react-i18next";
import z from "zod";
import { APIHooks } from "@/api/hooks";
import {
  PAYMENT_METHODS,
  type SearchPatientResponse,
} from "@/api/hooks/patient";
import { FormDatePicker } from "@/components/form/FormDatePicker";
import { FormInput } from "@/components/form/FormInput";
import { FormProvider } from "@/components/form/FormProvider";
import { FormSelect } from "@/components/form/FormSelect";
import { FormSwitch } from "@/components/form/FormSwitch";
import {
  Button,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  Label,
} from "@/components/ui";
import { useCurrentUser } from "@/hooks/useCurrentUser";
import type { DownloadableBlob } from "@/lib/DownloadableBlob";

interface GenerateInvoiceContent {
  patient: SearchPatientResponse;
  onClose: VoidFunction;
  setGeneratedInvoice: (invoice: DownloadableBlob) => void;
  setIsEmailSent: (val: boolean) => void;
}

export const GenerateInvoiceContent = ({
  patient,
  onClose,
  setGeneratedInvoice,
  setIsEmailSent,
}: GenerateInvoiceContent) => {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const { currentUser } = useCurrentUser();

  const myOfficesQuery = APIHooks.user.getMyOffices.useQuery(null);
  const generateInvoiceMutation =
    APIHooks.patient.generateInvoice.useMutation();

  const invoiceFormSchema = z.object({
    amount: z.coerce
      .number<number>()
      .min(1, t("invoice.errors.invalidAmount"))
      .refine(
        (val) => {
          // Check if the number has at most 2 decimal places
          // by multiplying by 100 and checking if it's an integer
          return Math.round(val * 100) === val * 100;
        },
        { message: t("invoice.errors.invalidAmount") },
      ),
    date: z.date(),
    shouldSendInvoiceByEmail: z.boolean(),
    practitionerOfficeId: z
      .string()
      .min(1, t("invoice.errors.officeMustBeSelected")),
    paymentMethod: z.enum(PAYMENT_METHODS).optional(),
  });

  type InvoiceFormData = z.infer<typeof invoiceFormSchema>;

  const invoiceForm = useForm<InvoiceFormData>({
    resolver: zodResolver(invoiceFormSchema),
    defaultValues: {
      amount: undefined,
      date: new Date(),
      shouldSendInvoiceByEmail: false,
      practitionerOfficeId: "",
    },
  });

  useEffect(() => {
    if (myOfficesQuery.data?.length === 1) {
      invoiceForm.setValue(
        "practitionerOfficeId",
        myOfficesQuery.data[0].id.toString(),
      );
    }
  }, [myOfficesQuery.data, invoiceForm]);

  const handleOnClose = () => {
    if (!generateInvoiceMutation.isPending) {
      invoiceForm.reset();
      onClose();
    }
  };

  const onSubmit = invoiceForm.handleSubmit(async (data) => {
    // Format date as YYYY-MM-DD using local timezone (not UTC)
    const year = data.date.getFullYear();
    const month = String(data.date.getMonth() + 1).padStart(2, "0");
    const day = String(data.date.getDate()).padStart(2, "0");

    generateInvoiceMutation
      .mutateAsync({
        patientId: patient.id,
        amount: data.amount,
        invoice_date: `${year}-${month}-${day}`,
        should_be_sent_by_email: data.shouldSendInvoiceByEmail,
        practitioner_office_id: +data.practitionerOfficeId,
        payment_method: data.paymentMethod ?? null,
      })
      .then((blob) => {
        setGeneratedInvoice(blob);
        setIsEmailSent(data.shouldSendInvoiceByEmail);
      });
  });

  return (
    <DialogContent
      className="sm:max-w-md"
      onInteractOutside={(e) => e.preventDefault()}
    >
      <DialogHeader>
        <DialogTitle className="flex items-center gap-2">
          <FileText className="h-5 w-5" />
          {t("invoice.modal.title")}
        </DialogTitle>
        <DialogDescription>{t("invoice.modal.description")}</DialogDescription>
      </DialogHeader>

      <FormProvider
        methods={invoiceForm}
        onSubmit={onSubmit}
        className="space-y-4"
      >
        <div className="rounded-lg border bg-muted/50 p-3">
          <p className="text-sm font-medium text-foreground">
            {t("invoice.modal.patient")}: {patient.first_name}{" "}
            {patient.last_name}
          </p>
          <p className="text-xs text-muted-foreground">
            {t("invoice.modal.id")}: {patient.id}
          </p>
        </div>

        {!currentUser?.business_information && (
          <div className="rounded-lg border-destructive border-2 bg-muted/50 p-3">
            <div className="flex gap-2">
              <CircleAlert className="text-destructive size-8" />
              <p className="text-sm font-medium text-foreground">
                {t("invoice.modal.missingInformationAlert")}
              </p>
            </div>
          </div>
        )}

        <div className="space-y-2">
          <Label htmlFor="amount">{t("invoice.modal.amount")} (€)</Label>
          <div className="relative">
            <FormInput
              id="amount"
              name="amount"
              type="number"
              step="0.01"
              min="0"
              placeholder="60.00"
              disabled={generateInvoiceMutation.isPending}
              className="pr-8"
            />
            <div className="absolute inset-y-0 right-0 flex items-center pr-3 pointer-events-none">
              <span className="text-muted-foreground text-sm">€</span>
            </div>
          </div>
        </div>

        <FormDatePicker
          name="date"
          label={t("invoice.modal.date")}
          disabled={generateInvoiceMutation.isPending}
        />

        <div className="space-y-2">
          <Label htmlFor="office" className="text-sm font-medium">
            {t("invoice.modal.office")}
          </Label>
          <FormSelect
            name="practitionerOfficeId"
            placeholder={t("patients.form.officePlaceholder")}
            options={
              myOfficesQuery.data?.map((office) => ({
                value: office.id.toString(),
                label: office.name,
              })) || []
            }
          />
        </div>

        <div className="space-y-2">
          <Label htmlFor="paymentMethod">
            <div className="flex flex-col gap-1">
              <div className="flex items-center gap-2">
                <HandCoins className="h-4 w-4" />
                {t("appointments.form.paymentMethod")}
              </div>
              <small className="text-xs italic text-muted-foreground">
                {t("invoice.modal.paymentMethodHint")}
              </small>
            </div>
          </Label>
          <FormSelect
            name="paymentMethod"
            placeholder={t("appointments.form.selectPaymentMethod")}
            options={PAYMENT_METHODS.map((method) => ({
              value: method,
              label: t(`paymentMethods.${method}`),
            }))}
          />
        </div>

        <div className="py-2 space-y-2">
          {!patient.email && (
            <div className="rounded-lg border-destructive border-2 bg-muted/50 p-3">
              <div className="flex gap-2">
                <CircleAlert className="text-destructive size-8" />
                <p className="text-sm font-medium text-foreground">
                  {t("invoice.modal.missingPatientAddressMail")}
                </p>
              </div>
            </div>
          )}
          <FormSwitch
            id="shouldSendInvoiceByEmail"
            name="shouldSendInvoiceByEmail"
            label={t("invoice.modal.sendInvoiceByEmail")}
            size="lg"
            className="cursor-pointer"
            disabled={!patient.email}
          />
        </div>

        <DialogFooter className="flex-col-reverse sm:flex-row gap-2">
          <Button
            type="button"
            variant="outline"
            onClick={handleOnClose}
            disabled={generateInvoiceMutation.isPending}
          >
            {t("common.cancel")}
          </Button>
          {currentUser?.business_information ? (
            <Button
              type="submit"
              disabled={generateInvoiceMutation.isPending}
              className="w-full sm:w-auto"
            >
              {generateInvoiceMutation.isPending ? (
                <>
                  <Loader2 className="h-4 w-4 animate-spin" />
                  {t("invoice.modal.generating")}
                </>
              ) : (
                <>
                  <FileText className="h-4 w-4" />
                  {t("invoice.modal.generate")}
                </>
              )}
            </Button>
          ) : (
            <Button
              type="button"
              onClick={() => navigate({ to: "/my_information" })}
            >
              {t("invoice.modal.completeInformation")}
            </Button>
          )}
        </DialogFooter>
      </FormProvider>
    </DialogContent>
  );
};
