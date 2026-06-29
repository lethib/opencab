import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { queryClient } from "@/api/api";
import { APIHooks } from "@/api/hooks";
import type { Company } from "@/api/hooks/practitioner_company";
import {
  Button,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui";

interface DeleteCompanyDialogProps {
  open: boolean;
  setIsOpen: (open: boolean) => void;
  company: Company;
}

export const DeleteCompanyDialog = ({
  open,
  setIsOpen,
  company,
}: DeleteCompanyDialogProps) => {
  const { t } = useTranslation();

  const deleteCompanyMutation = APIHooks.company
    .delete(company.id)
    .useMutation();

  const handleDelete = () =>
    deleteCompanyMutation.mutateAsync(null).then(() => {
      queryClient.invalidateQueries({ queryKey: ["/companies"] });
      toast.success(t("companies.delete.success"));
      setIsOpen(false);
    });

  return (
    <Dialog open={open} onOpenChange={setIsOpen}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{t("companies.delete.title")}</DialogTitle>
          <DialogDescription>
            {t("companies.delete.description")}
          </DialogDescription>
        </DialogHeader>
        <DialogFooter>
          <Button variant="outline" onClick={() => setIsOpen(false)}>
            {t("common.cancel")}
          </Button>
          <Button
            variant="destructive"
            onClick={handleDelete}
            disabled={deleteCompanyMutation.isPending}
          >
            {t("common.delete")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};
