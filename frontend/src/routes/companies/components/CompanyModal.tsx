import { zodResolver } from "@hookform/resolvers/zod";
import { Contact, HashIcon, Mail, MapPin } from "lucide-react";
import { useForm } from "react-hook-form";
import { useTranslation } from "react-i18next";
import z from "zod";
import { queryClient } from "@/api/api";
import { APIHooks } from "@/api/hooks";
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
}

const schema = z.object({
  name: z
    .string()
    .trim()
    .min(1, i18n.t("companies.form.validation.nameRequired")),
  contact_name: z.string().trim().min(1, "Le nom du contacy est requis."),
  contact_email: z
    .email(i18n.t("companies.form.validation.emailInvalid"))
    .trim()
    .min(1, i18n.t("companies.form.validation.emailRequired")),
  siret: z.string().trim().optional(),
  address_line_1: z.string().trim().optional(),
  address_zip_code: z.string().trim().optional(),
  address_city: z.string().trim().optional(),
});

export const CompanyModal = ({ open, setIsOpen }: Props) => {
  const { t } = useTranslation();
  const createMutation = APIHooks.company.create.useMutation();

  const form = useForm({
    resolver: zodResolver(schema),
    defaultValues: {
      name: "",
      contact_email: "",
    },
  });

  const handleClose = () => {
    setIsOpen(false);
    form.reset();
  };

  const onSubmit = form.handleSubmit(async (values) => {
    await createMutation
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
          <DialogTitle>{t("companies.form.addTitle")}</DialogTitle>
          <DialogDescription>
            {t("companies.form.addDescription")}
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
              <Label htmlFor="contact_name">Contact de facturation</Label>
              <FormInput
                id="contact_name"
                name="contact_name"
                type="text"
                placeholder="Jean Dupont"
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
              SIRET
            </Label>
            <FormInput
              id="siret"
              name="siret"
              type="text"
              placeholder="345 130 488 00017"
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
                Ville
              </Label>
              <FormInput
                id="address_city"
                name="address_city"
                type="text"
                placeholder="Paris"
                className="h-11"
              />
            </div>
          </div>

          <DialogFooter>
            <Button type="button" variant="outline" onClick={handleClose}>
              {t("common.cancel")}
            </Button>
            <Button type="submit" disabled={createMutation.isPending}>
              {t("companies.form.add")}
            </Button>
          </DialogFooter>
        </FormProvider>
      </DialogContent>
    </Dialog>
  );
};
