import { zodResolver } from "@hookform/resolvers/zod";
import { createFileRoute, Navigate, useNavigate } from "@tanstack/react-router";
import { Eye, EyeOff, Lock } from "lucide-react";
import { useState } from "react";
import { useForm } from "react-hook-form";
import { useTranslation } from "react-i18next";
import * as z from "zod";
import { APIClient } from "@/api/api";
import { FormInput } from "@/components/form/FormInput";
import { FormProvider } from "@/components/form/FormProvider";
import { Button, Label } from "@/components/ui";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";

type ResetPasswordSearch = {
  access_token: string;
};

export const Route = createFileRoute("/reset_password")({
  component: ResetPassword,
  validateSearch: (search: Record<string, unknown>): ResetPasswordSearch => {
    return {
      access_token: (search.access_token as string) || "",
    };
  },
});

function ResetPassword() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const { access_token } = Route.useSearch();
  const [showPassword, setShowPassword] = useState(false);
  const [showConfirmPassword, setShowConfirmPassword] = useState(false);
  const [resetSuccess, setResetSuccess] = useState(false);

  const resetMutation = APIClient.hooks.auth.reset.useMutation();

  const resetFormSchema = z
    .object({
      password: z
        .string()
        .min(6, t("auth.resetPassword.validation.passwordMinLength")),
      confirmPassword: z.string(),
    })
    .refine((data) => data.password === data.confirmPassword, {
      message: t("auth.resetPassword.validation.passwordsDontMatch"),
      path: ["confirmPassword"],
    });

  const resetForm = useForm({
    resolver: zodResolver(resetFormSchema),
    defaultValues: {
      password: "",
      confirmPassword: "",
    },
  });

  const onSubmit = async (data: z.infer<typeof resetFormSchema>) => {
    resetMutation.mutateAsync(
      {
        token: access_token,
        password: data.password,
      },
      {
        onSuccess: () => {
          setResetSuccess(true);
        },
        onError: (error) => {
          if (error.response?.status === 401) {
            resetForm.setError("password", {
              message: t("auth.resetPassword.invalidToken"),
            });
          }
        },
      },
    );
  };

  if (!access_token) {
    return <Navigate to="/login" />;
  }

  if (resetSuccess) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-gradient-to-br from-background via-background to-muted/20 px-4">
        <Card className="w-full max-w-md shadow-lg border-0 bg-card/50 backdrop-blur-sm">
          <CardHeader className="flex flex-col items-center space-y-2 pb-6">
            <img
              src="/favicon/favicon.svg"
              height={100}
              width={100}
              className="mb-4"
            />
            <CardTitle className="text-3xl font-bold tracking-tight text-center">
              {t("auth.resetPassword.success")}
            </CardTitle>
          </CardHeader>

          <CardContent className="space-y-6">
            <Button
              onClick={() => navigate({ to: "/login" })}
              className="w-full h-11 text-sm font-medium"
            >
              {t("auth.resetPassword.backToLogin")}
            </Button>
          </CardContent>
        </Card>
      </div>
    );
  }

  return (
    <div className="min-h-screen flex items-center justify-center bg-gradient-to-br from-background via-background to-muted/20 px-4">
      <Card className="w-full max-w-md shadow-lg border-0 bg-card/50 backdrop-blur-sm">
        <CardHeader className="flex flex-col items-center space-y-2 pb-6">
          <img
            src="/favicon/favicon.svg"
            height={100}
            width={100}
            className="mb-4"
          />
          <CardTitle className="text-3xl font-bold tracking-tight">
            {t("auth.resetPassword.title")}
          </CardTitle>
          <CardDescription className="text-muted-foreground">
            {t("auth.resetPassword.description")}
          </CardDescription>
        </CardHeader>

        <CardContent className="space-y-6">
          <FormProvider
            methods={resetForm}
            onSubmit={resetForm.handleSubmit((data) => onSubmit(data))}
            className="space-y-4"
          >
            <div className="space-y-2">
              <Label htmlFor="password" className="text-sm font-medium">
                {t("auth.resetPassword.newPassword")}
              </Label>
              <div className="relative">
                <FormInput
                  name="password"
                  id="password"
                  type={showPassword ? "text" : "password"}
                  placeholder={t("auth.resetPassword.newPasswordPlaceholder")}
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
                {t("auth.resetPassword.confirmPassword")}
              </Label>
              <div className="relative">
                <FormInput
                  name="confirmPassword"
                  id="confirmPassword"
                  type={showConfirmPassword ? "text" : "password"}
                  placeholder={t(
                    "auth.resetPassword.confirmPasswordPlaceholder",
                  )}
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

            <Button
              type="submit"
              className="w-full h-11 text-sm font-medium"
              disabled={resetMutation.isPending}
            >
              {resetMutation.isPending ? (
                <div className="flex items-center gap-2">
                  <div className="w-4 h-4 border-2 border-current border-t-transparent rounded-full animate-spin" />
                  {t("auth.resetPassword.resetting")}
                </div>
              ) : (
                t("auth.resetPassword.reset")
              )}
            </Button>
          </FormProvider>

          <div className="text-center">
            <button
              onClick={() => navigate({ to: "/login" })}
              className="text-sm text-primary hover:underline"
            >
              {t("auth.resetPassword.backToLogin")}
            </button>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
