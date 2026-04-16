import { zodResolver } from "@hookform/resolvers/zod";
import { Mail } from "lucide-react";
import { useState } from "react";
import { useForm } from "react-hook-form";
import { useTranslation } from "react-i18next";
import z from "zod";
import { APIClient } from "@/api/api";
import { FormInput } from "@/components/form/FormInput";
import { FormProvider } from "@/components/form/FormProvider";
import { Button, Label } from "@/components/ui";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";

interface ForgotPasswordModalProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

export const ForgotPasswordModal = ({
  open,
  onOpenChange,
}: ForgotPasswordModalProps) => {
  const { t } = useTranslation();
  const [emailSent, setEmailSent] = useState(false);

  const forgotPasswordMutation = APIClient.hooks.auth.forgot.useMutation();

  const forgotPasswordSchema = z.object({
    email: z.string().email(t("auth.forgotPassword.validation.invalidEmail")),
  });

  const forgotPasswordForm = useForm({
    resolver: zodResolver(forgotPasswordSchema),
    defaultValues: {
      email: "",
    },
  });

  const onSubmit = forgotPasswordForm.handleSubmit(async (data) => {
    forgotPasswordMutation.mutateAsync(data, {
      onSuccess: () => {
        setEmailSent(true);
      },
      onError: (error) => {
        alert(`Error: ${error.message}`);
      },
    });
  });

  const handleClose = () => {
    setEmailSent(false);
    forgotPasswordForm.reset();
    onOpenChange(false);
  };

  if (emailSent) {
    return (
      <Dialog open={open} onOpenChange={handleClose}>
        <DialogContent className="sm:max-w-md shadow-lg border-0 backdrop-blur-sm">
          <DialogHeader className="space-y-2 text-center pb-4">
            <DialogTitle className="text-2xl font-bold tracking-tight text-foreground">
              {t("auth.forgotPassword.success")}
            </DialogTitle>
          </DialogHeader>

          <div className="space-y-4">
            <Button
              onClick={handleClose}
              className="w-full h-11 text-sm font-medium"
            >
              {t("auth.forgotPassword.backToLogin")}
            </Button>
          </div>
        </DialogContent>
      </Dialog>
    );
  }

  return (
    <Dialog open={open} onOpenChange={handleClose}>
      <DialogContent className="sm:max-w-md shadow-lg border-0 backdrop-blur-sm">
        <DialogHeader className="space-y-2 text-center pb-4">
          <DialogTitle className="text-2xl font-bold tracking-tight text-foreground">
            {t("auth.forgotPassword.title")}
          </DialogTitle>
          <DialogDescription className="text-muted-foreground">
            {t("auth.forgotPassword.description")}
          </DialogDescription>
        </DialogHeader>

        <FormProvider
          methods={forgotPasswordForm}
          onSubmit={onSubmit}
          className="space-y-4"
        >
          <div className="space-y-2">
            <Label htmlFor="email" className="text-sm font-medium">
              {t("auth.forgotPassword.email")}
            </Label>
            <FormInput
              name="email"
              id="email"
              type="email"
              placeholder={t("auth.forgotPassword.emailPlaceholder")}
              className="pl-10 h-11"
              icon={
                <Mail className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
              }
            />
          </div>

          <div className="flex gap-3 pt-4">
            <Button
              type="button"
              variant="outline"
              className="flex-1 h-11"
              onClick={handleClose}
            >
              {t("auth.forgotPassword.cancel")}
            </Button>
            <Button
              type="submit"
              className="flex-1 h-11 text-sm font-medium"
              disabled={forgotPasswordForm.formState.isSubmitting}
            >
              {forgotPasswordForm.formState.isSubmitting ? (
                <div className="flex items-center gap-2">
                  <div className="w-4 h-4 border-2 border-current border-t-transparent rounded-full animate-spin" />
                  {t("auth.forgotPassword.sending")}
                </div>
              ) : (
                t("auth.forgotPassword.send")
              )}
            </Button>
          </div>
        </FormProvider>
      </DialogContent>
    </Dialog>
  );
};
