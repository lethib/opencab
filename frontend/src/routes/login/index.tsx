import { zodResolver } from "@hookform/resolvers/zod";
import { createFileRoute, Navigate } from "@tanstack/react-router";
import { Eye, EyeOff, Lock, Mail } from "lucide-react";
import { useState } from "react";
import { useForm } from "react-hook-form";
import { useTranslation } from "react-i18next";
import * as z from "zod";
import { APIClient } from "@/api/api";
import { FormInput } from "@/components/form/FormInput";
import { FormProvider } from "@/components/form/FormProvider";
import { Button, Label } from "@/components/ui";
import { Card, CardContent, CardHeader } from "@/components/ui/card";
import { useCurrentUser } from "@/hooks/useCurrentUser";
import { login } from "@/lib/authUtils";
import { AccessKeyModal } from "./components/AccessKeyModal";
import { ForgotPasswordModal } from "./components/ForgotPasswordModal";
import { RegisterModal } from "./components/RegisterModal";

export const Route = createFileRoute("/login/")({
  component: Login,
});

function Login() {
  const { t } = useTranslation();
  const { currentUser } = useCurrentUser();
  const [showPassword, setShowPassword] = useState(false);
  const [isRegisterModalOpen, setIsRegisterModalOpen] = useState(false);
  const [isAccessKeyModalOpen, setIsAccessKeyModalOpen] = useState(false);
  const [isForgotPasswordModalOpen, setIsForgotPasswordModalOpen] =
    useState(false);
  const [userEmail, setUserEmail] = useState("");

  const loginMutation = APIClient.hooks.auth.login.useMutation();

  const loginFormSchema = z.object({
    email: z.string().email(t("auth.login.validation.invalidEmail")),
    password: z.string().min(1, t("auth.login.validation.passwordRequired")),
  });

  const loginForm = useForm({
    resolver: zodResolver(loginFormSchema),
    defaultValues: {
      email: "",
      password: "",
    },
  });

  const onSubmit = async (data: z.infer<typeof loginFormSchema>) => {
    loginMutation.mutateAsync(data, {
      onSuccess: (res) => {
        login(res.token);
      },
      onError: (error) => {
        if (error.response?.data.msg === "access_key_needs_to_be_verified") {
          setUserEmail(data.email);
          setIsAccessKeyModalOpen(true);
          return;
        } else if (error.response?.data.msg === "invalid_credentials") {
          loginForm.setError("password", { message: "invalid credentials" });
          return;
        }
      },
    });
  };

  if (currentUser) return <Navigate to="/" />;

  return (
    <>
      <div className="min-h-screen flex items-center justify-center bg-gradient-to-br from-background via-background to-muted/20 px-4">
        <Card className="w-full max-w-md shadow-lg border-0 bg-card/50 backdrop-blur-sm">
          <CardHeader className="flex flex-col items-center space-y-2 pt-4 pb-2">
            <img src="/favicon/opencab-main.png" width={300} />
          </CardHeader>

          <CardContent className="space-y-6">
            <FormProvider
              methods={loginForm}
              onSubmit={loginForm.handleSubmit((data) => onSubmit(data))}
              className="space-y-4"
            >
              <div className="space-y-2">
                <Label htmlFor="email" className="text-sm font-medium">
                  {t("auth.login.email")}
                </Label>
                <FormInput
                  name="email"
                  id="email"
                  type="email"
                  placeholder={t("auth.login.emailPlaceholder")}
                  className="pl-10 h-11"
                  icon={
                    <Mail className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                  }
                />
              </div>

              <div className="space-y-2">
                <Label htmlFor="password" className="text-sm font-medium">
                  {t("auth.login.password")}
                </Label>
                <div className="relative">
                  <FormInput
                    name="password"
                    id="password"
                    type={showPassword ? "text" : "password"}
                    placeholder={t("auth.login.passwordPlaceholder")}
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

              <Button
                type="submit"
                className="w-full h-11 text-sm font-medium"
                disabled={loginMutation.isPending}
              >
                {loginMutation.isPending ? (
                  <div className="flex items-center gap-2">
                    <div className="w-4 h-4 border-2 border-current border-t-transparent rounded-full animate-spin" />
                    {t("auth.login.signingIn")}
                  </div>
                ) : (
                  t("auth.login.signIn")
                )}
              </Button>
            </FormProvider>

            <div className="text-center space-y-2">
              <button
                className="text-sm text-primary hover:underline"
                onClick={() => setIsForgotPasswordModalOpen(true)}
              >
                {t("auth.login.forgotPassword")}
              </button>
              <p className="text-sm text-muted-foreground">
                {t("auth.login.noAccount")}{" "}
                <button
                  className="text-primary hover:underline font-medium"
                  onClick={() => setIsRegisterModalOpen(true)}
                >
                  {t("auth.login.signUp")}
                </button>
              </p>
            </div>
          </CardContent>
        </Card>
      </div>

      <RegisterModal
        open={isRegisterModalOpen}
        onOpenChange={setIsRegisterModalOpen}
      />

      <AccessKeyModal
        userEmail={userEmail}
        open={isAccessKeyModalOpen}
        onOpenChange={setIsAccessKeyModalOpen}
      />

      <ForgotPasswordModal
        open={isForgotPasswordModalOpen}
        onOpenChange={setIsForgotPasswordModalOpen}
      />
    </>
  );
}
