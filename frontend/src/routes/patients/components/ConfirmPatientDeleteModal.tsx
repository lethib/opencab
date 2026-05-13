import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { queryClient } from "@/api/api";
import { APIHooks } from "@/api/hooks";
import type { SearchPatientResponse } from "@/api/hooks/patient";
import {
  Button,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui";

interface ConfirmPatientDeleteProps {
  isOpen: boolean;
  onClose: () => void;
  patient: SearchPatientResponse;
}

export const ConfirmPatientDeleteModal = ({
  isOpen,
  onClose,
  patient,
}: ConfirmPatientDeleteProps) => {
  const { t } = useTranslation();

  const deletePatientMutation = APIHooks.patient.deletePatient.useMutation({
    patient_id: patient.id,
  });

  const deletePatient = () => {
    deletePatientMutation.mutateAsync(null).then(() => {
      queryClient.invalidateQueries({ queryKey: ["/patient/_search"] });
      onClose();
      toast.success(
        t("patients.deleteModal.successMessage", {
          firstName: patient.first_name,
          lastName: patient.last_name,
        }),
      );
    });
  };

  return (
    <Dialog open={isOpen} onOpenChange={onClose}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{t("patients.deleteModal.title")}</DialogTitle>
          <DialogDescription>
            {t("patients.deleteModal.description", {
              firstName: patient.first_name,
              lastName: patient.last_name,
            })}
          </DialogDescription>
        </DialogHeader>

        <DialogFooter>
          <Button type="button" variant="outline" onClick={onClose}>
            {t("common.close")}
          </Button>
          <Button
            type="button"
            variant="destructive"
            onClick={deletePatient}
            disabled={deletePatientMutation.isPending}
            className="w-full sm:w-auto"
          >
            {t("common.delete")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};
