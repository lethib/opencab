import { zodResolver } from "@hookform/resolvers/zod";
import { Contact, HashIcon, Mail, MapPin } from "lucide-react";
import { useForm } from "react-hook-form";
import { useTranslation } from "react-i18next";
import z from "zod";
import { queryClient } from "@/api/api";
import { APIHooks } from "@/api/hooks";
import type { Company } from "@/api/hooks/practitioner_company";
import { FormInput } from "@/components/form/FormInput";
import { FormProvider } from "@/components/form/FormProvider";
import {
  Button,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  Label,
} from "@/components/ui";
import i18n from "@/i18n";

interface Props {
  open: boolean;
  setIsOpen: (open: boolean) => void;
  company?: Company;
}

const schema = z.object({
  name: z
    .string()
    .trim()
    .min(1, i18n.t("companies.form.validation.nameRequired")),
  contact_name: z
    .string()
    .trim()
    .min(1, i18n.t("companies.form.validation.contactNameRequired")),
  contact_email: z
    .email(i18n.t("companies.form.validation.emailInvalid"))
    .trim()
    .min(1, i18n.t("companies.form.validation.emailRequired")),
  siret: z.string().trim().optional(),
  address_line_1: z.string().trim().optional(),
  address_zip_code: z.string().trim().optional(),
  address_city: z.string().trim().optional(),
});

export const CompanyModal = ({ open, setIsOpen, company }: Props) => {
  const { t } = useTranslation();
  const isEditing = !!company;
  const createMutation = APIHooks.company.create.useMutation();
  const updateMutation = APIHooks.company
    .update(company?.id ?? 0)
    .useMutation();

  const form = useForm({
    resolver: zodResolver(schema),
    defaultValues: {
      name: company?.name ?? "",
      contact_name: company?.contact_name ?? "",
      contact_email: company?.contact_email ?? "",
      siret: company?.siret ?? "",
      address_line_1: company?.address_line_1 ?? "",
      address_zip_code: company?.address_zip_code ?? "",
      address_city: company?.address_city ?? "",
    },
  });

  const handleClose = () => {
    setIsOpen(false);
    form.reset();
  };

  const onSubmit = form.handleSubmit(async (values) => {
    const mutation = isEditing ? updateMutation : createMutation;
    await mutation
      .mutateAsync(values)
      .then(async () => {
        await queryClient.invalidateQueries({ queryKey: ["/companies"] });
        handleClose();
      })
      .catch((error) => alert((error as Error).message));
  });

  return (
    <Dialog open={open} onOpenChange={handleClose}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>
            {isEditing
              ? t("companies.form.editTitle")
              : t("companies.form.addTitle")}
          </DialogTitle>
          <DialogDescription>
            {isEditing
              ? t("companies.form.editDescription")
              : t("companies.form.addDescription")}
          </DialogDescription>
        </DialogHeader>

        <FormProvider methods={form} onSubmit={onSubmit} className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="name">{t("companies.form.name")}</Label>
            <FormInput
              id="name"
              name="name"
              type="text"
              placeholder={t("companies.form.namePlaceholder")}
              className="h-11"
            />
          </div>

          <div className="grid grid-cols-2 gap-4">
            <div className="space-y-2">
              <Label htmlFor="contact_name">
                {t("companies.form.contactName")}
              </Label>
              <FormInput
                id="contact_name"
                name="contact_name"
                type="text"
                placeholder={t("companies.form.contactNamePlaceholder")}
                className="pl-10 h-11"
                icon={
                  <Contact className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                }
              />
            </div>

            <div className="space-y-2">
              <Label htmlFor="contact_email">{t("companies.form.email")}</Label>
              <FormInput
                id="contact_email"
                name="contact_email"
                type="email"
                placeholder={t("companies.form.emailPlaceholder")}
                className="pl-10 h-11"
                icon={
                  <Mail className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                }
              />
            </div>
          </div>

          <div className="space-y-2">
            <Label optional htmlFor="siret">
              {t("companies.form.siret")}
            </Label>
            <FormInput
              id="siret"
              name="siret"
              type="text"
              placeholder={t("companies.form.siretPlaceholder")}
              className="pl-10 h-11"
              icon={
                <HashIcon className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
              }
            />
          </div>

          <div className="space-y-2">
            <Label optional htmlFor="address_line_1">
              {t("companies.form.address")}
            </Label>
            <FormInput
              id="address_line_1"
              name="address_line_1"
              type="text"
              placeholder={t("companies.form.addressPlaceholder")}
              className="pl-10 h-11"
              icon={
                <MapPin className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
              }
            />
          </div>

          <div className="grid grid-cols-2 gap-4">
            <div className="space-y-2">
              <Label optional htmlFor="address_zip_code">
                {t("companies.form.zipCode")}
              </Label>
              <FormInput
                id="address_zip_code"
                name="address_zip_code"
                type="text"
                placeholder={t("companies.form.zipCodePlaceholder")}
                className="h-11"
              />
            </div>

            <div className="space-y-2">
              <Label optional htmlFor="address_city">
                {t("companies.form.city")}
              </Label>
              <FormInput
                id="address_city"
                name="address_city"
                type="text"
                placeholder={t("companies.form.cityPlaceholder")}
                className="h-11"
              />
            </div>
          </div>

          <DialogFooter>
            <Button type="button" variant="outline" onClick={handleClose}>
              {t("common.cancel")}
            </Button>
            <Button
              type="submit"
              disabled={createMutation.isPending || updateMutation.isPending}
            >
              {isEditing ? t("companies.form.save") : t("companies.form.add")}
            </Button>
          </DialogFooter>
        </FormProvider>
      </DialogContent>
    </Dialog>
  );
};
