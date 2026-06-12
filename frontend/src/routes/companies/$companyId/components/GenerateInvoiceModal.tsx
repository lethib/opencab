import { zodResolver } from "@hookform/resolvers/zod";
import { useQueryClient } from "@tanstack/react-query";
import { format } from "date-fns";
import { t } from "i18next";
import { FileText } from "lucide-react";
import { useEffect } from "react";
import { useForm, useWatch } from "react-hook-form";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import z from "zod";
import { APIHooks } from "@/api/hooks";
import type { Company } from "@/api/hooks/practitioner_company";
import { FormDatePicker } from "@/components/form/FormDatePicker";
import { FormInput } from "@/components/form/FormInput";
import { FormProvider } from "@/components/form/FormProvider";
import { FormSelect } from "@/components/form/FormSelect";
import {
  Button,
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  Label,
} from "@/components/ui";
import { DialogDescription } from "@/components/ui/dialog";

const VAT_OPTIONS = [
  { value: "0", label: "0 %" },
  { value: "5.5", label: "5,5 %" },
  { value: "10", label: "10 %" },
  { value: "20", label: "20 %" },
];

const schema = z.object({
  invoice_date: z.date(),
  description: z.string().trim().min(1, "L'objet est requis"),
  quantity: z.coerce
    .number<number>()
    .min(1, "La quantité doit être au moins 1"),
  unit_price_ht: z.coerce.number<number>().min(0, "Le prix doit être positif"),
  vat_rate: z.enum(["0", "5.5", "10", "20"]),
  practitioner_office_id: z
    .string()
    .min(1, t("invoice.errors.officeMustBeSelected")),
});

export type GenerateInvoiceFormValues = z.infer<typeof schema>;

const defaultValues = {
  invoice_date: new Date(),
  description: "",
  quantity: 1,
  unit_price_ht: 0,
  vat_rate: "0" as const,
};

interface Props {
  open: boolean;
  setIsOpen: (open: boolean) => void;
  company: Company;
}

export const GenerateInvoiceModal = ({ open, setIsOpen, company }: Props) => {
  const { t } = useTranslation();
  const queryClient = useQueryClient();

  const form = useForm<GenerateInvoiceFormValues>({
    resolver: zodResolver(schema),
    defaultValues,
  });

  const myOfficesQuery = APIHooks.user.getMyOffices.useQuery(null, {
    enabled: open,
  });
  const generateInvoice = APIHooks.company
    .generateInvoice(company.id)
    .useMutation();

  const quantity = useWatch({ control: form.control, name: "quantity" });
  const unitPriceHt = useWatch({
    control: form.control,
    name: "unit_price_ht",
  });
  const vatRate = useWatch({ control: form.control, name: "vat_rate" });

  const totalHt = Number(quantity || 0) * Number(unitPriceHt || 0);
  const vatRateNum = parseFloat(vatRate || "0");
  const tvaAmount = totalHt * (vatRateNum / 100);
  const totalTtc = totalHt + tvaAmount;

  const fmt = (n: number) =>
    n.toLocaleString("fr-FR", {
      minimumFractionDigits: 2,
      maximumFractionDigits: 2,
    });

  useEffect(() => {
    if (myOfficesQuery.data?.length === 1) {
      form.setValue(
        "practitioner_office_id",
        myOfficesQuery.data[0].id.toString(),
      );
    }
  }, [myOfficesQuery.data, form]);

  const handleClose = () => {
    form.reset(defaultValues);
    setIsOpen(false);
  };

  const onSubmit = form.handleSubmit((values) => {
    generateInvoice.mutate(
      {
        invoice_date: format(values.invoice_date, "yyyy-MM-dd"),
        description: values.description,
        quantity: values.quantity,
        unit_price_ht: values.unit_price_ht,
        vat_rate: values.vat_rate,
        practitioner_office_id: +values.practitioner_office_id,
      },
      {
        onSuccess: (blob) => {
          blob.download();
          queryClient.invalidateQueries({
            queryKey: [`/companies/${company.id}/interventions`],
          });
          toast.success(t("companies.invoice.success"));
          handleClose();
        },
        onError: (error) => {
          toast.error(error.message);
        },
      },
    );
  });

  return (
    <Dialog open={open} onOpenChange={handleClose}>
      <DialogContent className="sm:max-w-lg">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <FileText className="h-5 w-5" />
            {t("companies.invoice.title")}
          </DialogTitle>
          <DialogDescription>
            {t("companies.invoice.for", { name: company.name })}
          </DialogDescription>
        </DialogHeader>

        <FormProvider methods={form} onSubmit={onSubmit} className="space-y-5">
          <FormDatePicker
            name="invoice_date"
            label={t("companies.invoice.date")}
          />

          <div className="space-y-2">
            <p className="text-sm font-medium">
              {t("companies.invoice.billingLine")}
            </p>
            <div className="grid grid-cols-[1fr_56px_88px_76px] gap-2">
              <span className="text-xs font-medium uppercase tracking-wide text-muted-foreground">
                {t("companies.invoice.object")}
              </span>
              <span className="text-xs font-medium uppercase tracking-wide text-muted-foreground">
                {t("companies.invoice.quantity")}
              </span>
              <span className="text-xs font-medium uppercase tracking-wide text-muted-foreground">
                {t("companies.invoice.unitPriceHt")}
              </span>
              <span className="text-xs font-medium uppercase tracking-wide text-muted-foreground">
                {t("companies.invoice.vat")}
              </span>
            </div>
            <div className="grid grid-cols-[1fr_56px_88px_76px] items-start gap-2">
              <FormInput
                name="description"
                type="text"
                placeholder={t("companies.invoice.objectPlaceholder")}
                className="h-9"
              />
              <FormInput
                name="quantity"
                type="number"
                min="1"
                className="h-9"
              />
              <FormInput
                name="unit_price_ht"
                type="number"
                min="0"
                step="0.01"
                placeholder="0"
                className="h-9"
              />
              <FormSelect name="vat_rate" options={VAT_OPTIONS} />
            </div>
          </div>

          <div className="space-y-2">
            <Label htmlFor="office" className="text-sm font-medium">
              {t("invoice.modal.office")}
            </Label>
            <FormSelect
              name="practitioner_office_id"
              placeholder={t("patients.form.officePlaceholder")}
              options={
                myOfficesQuery.data?.map((office) => ({
                  value: office.id.toString(),
                  label: office.name,
                })) || []
              }
            />
          </div>

          <div className="space-y-2 rounded-lg bg-muted/50 p-4 text-sm">
            <div className="flex justify-between text-muted-foreground">
              <span>{t("companies.invoice.totalHt")}</span>
              <span>€ {fmt(totalHt)}</span>
            </div>
            <div className="flex justify-between text-muted-foreground">
              <span>{t("companies.invoice.vat")}</span>
              <span>
                {vatRateNum === 0
                  ? t("companies.invoice.vatNotApplicable")
                  : `€ ${fmt(tvaAmount)}`}
              </span>
            </div>
            <div className="flex justify-between border-t pt-2 text-base font-semibold">
              <span>{t("companies.invoice.totalTtc")}</span>
              <span>€ {fmt(totalTtc)}</span>
            </div>
          </div>

          <DialogFooter>
            <Button type="button" variant="outline" onClick={handleClose}>
              {t("common.cancel")}
            </Button>
            <Button type="submit" disabled={generateInvoice.isPending}>
              <FileText className="h-4 w-4" />
              {generateInvoice.isPending
                ? t("common.loading")
                : t("companies.invoice.generate")}
            </Button>
          </DialogFooter>
        </FormProvider>
      </DialogContent>
    </Dialog>
  );
};
