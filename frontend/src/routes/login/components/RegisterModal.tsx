import { zodResolver } from "@hookform/resolvers/zod";
import { Eye, EyeOff, Lock, Mail, User } from "lucide-react";
import { useState } from "react";
import { useForm } from "react-hook-form";
import { useTranslation } from "react-i18next";
import z from "zod";
import { APIClient } from "@/api/api";
import { FormInput } from "@/components/form/FormInput";
import { FormPhoneInput } from "@/components/form/FormPhoneInput";
import { FormProvider } from "@/components/form/FormProvider";
import { Button, Label } from "@/components/ui";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";

interface RegisterModalProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

export function RegisterModal({ open, onOpenChange }: RegisterModalProps) {
  const { t } = useTranslation();
  const [showPassword, setShowPassword] = useState(false);
  const [showConfirmPassword, setShowConfirmPassword] = useState(false);

  const registerMutation = APIClient.hooks.auth.register.useMutation();

  const registerFormSchema = z
    .object({
      firstName: z
        .string()
        .trim()
        .min(1, t("auth.register.validation.firstNameRequired")),
      lastName: z
        .string()
        .trim()
        .min(1, t("auth.register.validation.lastNameRequired")),
      email: z
        .string()
        .email({ message: t("auth.register.validation.invalidEmail") }),
      password: z
        .string()
        .min(6, t("auth.register.validation.passwordMinLength")),
      confirmPassword: z.string(),
      phoneNumber: z.string().length(12),
    })
    .refine((data) => data.password === data.confirmPassword, {
      message: t("auth.register.validation.passwordsDontMatch"),
      path: ["confirmPassword"],
    });

  const registerForm = useForm({
    resolver: zodResolver(registerFormSchema),
    defaultValues: {
      firstName: "",
      lastName: "",
      email: "",
      password: "",
      confirmPassword: "",
      phoneNumber: "",
    },
  });

  const onSubmit = registerForm.handleSubmit(async (data) => {
    registerMutation.mutateAsync(
      {
        email: data.email,
        password: data.password,
        first_name: data.firstName,
        last_name: data.lastName,
        phone_number: data.phoneNumber || "",
      },
      {
        onSuccess: () => {
          alert(t("auth.register.successMessage"));
          onOpenChange(false);
          registerForm.reset();
        },
        onError: (error) => {
          alert(`${t("auth.register.error")}: ${error.message}`);
        },
      },
    );
  });

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-md shadow-lg border-0 backdrop-blur-sm">
        <DialogHeader className="space-y-2 text-center pb-4">
          <DialogTitle className="text-2xl font-bold tracking-tight text-foreground">
            {t("auth.register.title")}
          </DialogTitle>
          <DialogDescription className="text-muted-foreground">
            {t("auth.register.description")}
          </DialogDescription>
        </DialogHeader>

        <FormProvider
          methods={registerForm}
          onSubmit={onSubmit}
          className="space-y-4"
        >
          <div className="grid grid-cols-2 gap-4">
            <div className="space-y-2">
              <Label htmlFor="firstName" className="text-sm font-medium">
                {t("auth.register.firstName")}
              </Label>
              <FormInput
                id="firstName"
                name="firstName"
                type="text"
                placeholder={t("auth.register.firstNamePlaceholder")}
                className="pl-10 h-11"
                icon={
                  <User className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                }
              />
            </div>

            <div className="space-y-2">
              <Label htmlFor="lastName" className="text-sm font-medium">
                {t("auth.register.lastName")}
              </Label>
              <FormInput
                id="lastName"
                name="lastName"
                type="text"
                placeholder={t("auth.register.lastNamePlaceholder")}
                className="pl-10 h-11"
                icon={
                  <User className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                }
              />
            </div>
          </div>

          <div className="space-y-2">
            <Label htmlFor="email" className="text-sm font-medium">
              {t("auth.register.email")}
            </Label>
            <FormInput
              id="email"
              name="email"
              type="email"
              placeholder={t("auth.register.emailPlaceholder")}
              className="pl-10 h-11"
              icon={
                <Mail className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
              }
            />
          </div>

          <div className="space-y-2">
            <Label htmlFor="password" className="text-sm font-medium">
              {t("auth.register.password")}
            </Label>
            <div className="relative">
              <FormInput
                id="password"
                name="password"
                type={showPassword ? "text" : "password"}
                placeholder={t("auth.register.passwordPlaceholder")}
                className="pl-10 pr-10 h-11"
                icon={
                  <Lock className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                }
              />
              <button
                type="button"
                onClick={() => setShowPassword(!showPassword)}
                className="absolute right-3 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground transition-colors"
              >
                {showPassword ? (
                  <EyeOff className="h-4 w-4" />
                ) : (
                  <Eye className="h-4 w-4" />
                )}
              </button>
            </div>
          </div>

          <div className="space-y-2">
            <Label htmlFor="confirmPassword" className="text-sm font-medium">
              {t("auth.register.confirmPassword")}
            </Label>
            <div className="relative">
              <FormInput
                id="confirmPassword"
                name="confirmPassword"
                type={showConfirmPassword ? "text" : "password"}
                placeholder={t("auth.register.confirmPasswordPlaceholder")}
                className="pl-10 pr-10 h-11"
                icon={
                  <Lock className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                }
              />
              <button
                type="button"
                onClick={() => setShowConfirmPassword(!showConfirmPassword)}
                className="absolute right-3 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground transition-colors"
              >
                {showConfirmPassword ? (
                  <EyeOff className="h-4 w-4" />
                ) : (
                  <Eye className="h-4 w-4" />
                )}
              </button>
            </div>
          </div>

          <div className="space-y-2">
            <Label htmlFor="phoneNumber" className="text-sm font-medium">
              {t("auth.register.phoneNumber")}
            </Label>
            <FormPhoneInput name="phoneNumber" />
          </div>

          <div className="flex gap-3 pt-4">
            <Button
              type="button"
              variant="outline"
              className="flex-1 h-11"
              onClick={() => onOpenChange(false)}
            >
              {t("auth.register.cancel")}
            </Button>
            <Button
              type="submit"
              className="flex-1 h-11 text-sm font-medium"
              disabled={registerForm.formState.isLoading}
            >
              {registerForm.formState.isLoading ? (
                <div className="flex items-center gap-2">
                  <div className="w-4 h-4 border-2 border-current border-t-transparent rounded-full animate-spin" />
                  {t("auth.register.creating")}
                </div>
              ) : (
                t("auth.register.createAccount")
              )}
            </Button>
          </div>
        </FormProvider>

        <div className="text-center pt-4 border-t">
          <p className="text-sm text-muted-foreground">
            {t("auth.register.hasAccount")}{" "}
            <button
              className="text-primary hover:underline font-medium"
              onClick={() => onOpenChange(false)}
            >
              {t("auth.register.signInInstead")}
            </button>
          </p>
        </div>
      </DialogContent>
    </Dialog>
  );
}
