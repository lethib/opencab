import { zodResolver } from "@hookform/resolvers/zod";
import { Mail, MapPin } from "lucide-react";
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

interface Props {
  open: boolean;
  setIsOpen: (open: boolean) => void;
}

const FR_ZIP_CODE_REGEX = /^(?:0[1-9]|[1-8]\d|9[0-8])\d{3}$/;

export const CompanyModal = ({ open, setIsOpen }: Props) => {
  const { t } = useTranslation();
  const createMutation = APIHooks.company.create.useMutation();

  const schema = z.object({
    name: z.string().trim().min(1, t("companies.form.validation.nameRequired")),
    contact_email: z
      .email(t("companies.form.validation.emailInvalid"))
      .trim()
      .min(1, t("companies.form.validation.emailRequired")),
    address_line_1: z.string().trim().optional(),
    address_zip_code: z
      .string()
      .trim()
      .refine((v) => !v || FR_ZIP_CODE_REGEX.test(v), {
        message: t("companies.form.validation.zipCodeInvalid"),
      })
      .optional(),
  });

  type FormValues = z.infer<typeof schema>;

  const form = useForm<FormValues>({
    resolver: zodResolver(schema),
    defaultValues: {
      name: "",
      contact_email: "",
      address_line_1: "",
      address_zip_code: "",
    },
  });

  const handleClose = () => {
    setIsOpen(false);
    form.reset();
  };

  const onSubmit = form.handleSubmit(async (values) => {
    await createMutation
      .mutateAsync({
        name: values.name,
        contact_email: values.contact_email,
        address_line_1: values.address_line_1 || undefined,
        address_zip_code: values.address_zip_code || undefined,
      })
      .then(() => {
        queryClient.invalidateQueries({ queryKey: ["/companies"] });
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

          <div className="space-y-2">
            <Label htmlFor="address_line_1">
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

          <div className="space-y-2">
            <Label htmlFor="address_zip_code">
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
