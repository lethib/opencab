import { PenTool, Upload } from "lucide-react";
import { useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { queryClient } from "@/api/api";
import { APIHooks } from "@/api/hooks";
import {
  Button,
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
  Label,
} from "@/components/ui";
import { useCurrentUser } from "@/hooks/useCurrentUser";

export const SignatureCard = () => {
  const { t } = useTranslation();
  const { currentUser } = useCurrentUser();
  const fileInputRef = useRef<HTMLInputElement>(null);
  const [selectedFile, setSelectedFile] = useState<File | null>(null);
  const [uploadStatus, setUploadStatus] = useState<
    "idle" | "success" | "error"
  >("idle");

  const getSignatureURLMutation = APIHooks.user.signature.getURL.useMutation();
  const uploadSignatureMutation = APIHooks.user.signature.upload.useMutation();

  const handleFileSelect = (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (file) {
      const validTypes = ["image/png", "image/jpeg", "image/jpg"];
      if (!validTypes.includes(file.type)) {
        alert(t("signature.invalidFileType"));
        return;
      }

      // Validate file size (max 200KB)
      const maxSize = 200 * 1024;
      if (file.size > maxSize) {
        alert(t("signature.fileTooLarge"));
        return;
      }

      setSelectedFile(file);
      setUploadStatus("idle");
    }
  };

  const handleUploadSignature = async () => {
    if (!selectedFile) return;

    uploadSignatureMutation
      .mutateAsync(selectedFile)
      .then(() => {
        queryClient.invalidateQueries({ queryKey: ["/auth/me"] });
        setUploadStatus("success");
        setSelectedFile(null);
        if (fileInputRef.current) {
          fileInputRef.current.value = "";
        }
      })
      .catch(() => {
        setUploadStatus("error");
      });
  };

  const displaySignatureInNewTab = () => {
    getSignatureURLMutation.mutateAsync(null).then((url) => {
      window.open(url, "_blank");
    });
  };

  return (
    <Card className="mt-6">
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <PenTool className="h-5 w-5" />
          {t("signature.title")}
        </CardTitle>
        <CardDescription>{t("signature.subtitle")}</CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        {currentUser?.business_information?.signature_filename && (
          <div
            className="rounded-md border border-border bg-muted p-3 hover:cursor-pointer hover:bg-accent transition-colors"
            onClick={displaySignatureInNewTab}
          >
            <Label className="text-sm font-medium hover:cursor-pointer">
              {t("signature.currentFile")}
            </Label>
            <p className="mt-1 text-sm text-muted-foreground">
              {currentUser.business_information.signature_filename}
            </p>
          </div>
        )}

        <div className="space-y-2">
          <Label htmlFor="signature" className="text-sm font-medium">
            {t("signature.selectFile")}
          </Label>
          <input
            ref={fileInputRef}
            id="signature"
            type="file"
            accept="image/png,image/jpeg,image/jpg"
            onChange={handleFileSelect}
            className="hidden"
          />
          <div className="flex items-center gap-3">
            <Button
              type="button"
              variant="outline"
              onClick={() => fileInputRef.current?.click()}
              className="flex-1"
            >
              <Upload className="mr-2 h-4 w-4" />
              {selectedFile ? selectedFile.name : t("signature.chooseFile")}
            </Button>
            {selectedFile && (
              <Button
                type="button"
                onClick={handleUploadSignature}
                disabled={uploadSignatureMutation.isPending}
                className="px-8"
              >
                {uploadSignatureMutation.isPending
                  ? t("signature.uploading")
                  : t("signature.upload")}
              </Button>
            )}
          </div>
          <p className="text-xs text-muted-foreground">
            {t("signature.fileRequirements")}
          </p>
        </div>

        {uploadStatus === "success" && (
          <div className="rounded-md bg-green-500/10 p-3 text-sm text-green-700 dark:text-green-400">
            {t("signature.uploadSuccess")}
          </div>
        )}

        {uploadStatus === "error" && (
          <div className="rounded-md bg-destructive/10 p-3 text-sm text-destructive">
            {t("signature.uploadError")}
          </div>
        )}
      </CardContent>
    </Card>
  );
};
