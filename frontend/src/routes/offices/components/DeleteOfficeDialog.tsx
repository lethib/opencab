import { useState } from "react";
import { useTranslation } from "react-i18next";
import { APIClient, queryClient } from "@/api/api";
import type { PractitionerOffice } from "@/api/hooks/practitioner_office";
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
import { Switch } from "@/components/ui/switch";

interface DeleteOfficeDialogProps {
  open: boolean;
  setIsOpen: (open: boolean) => void;
  office: PractitionerOffice;
}

export const DeleteOfficeDialog = ({
  open,
  setIsOpen,
  office,
}: DeleteOfficeDialogProps) => {
  const { t } = useTranslation();
  const [alsoDeletePatients, setAlsoDeletePatients] = useState(false);

  const deleteOfficeMutation = APIClient.hooks.office.deleteOffice.useMutation({
    office_id: office.id,
  });

  const handleDelete = () =>
    deleteOfficeMutation
      .mutateAsync({ also_delete_patients: alsoDeletePatients })
      .then(() => {
        queryClient.invalidateQueries({ queryKey: ["/user/my_offices"] });
        setIsOpen(false);
      });

  return (
    <Dialog open={open} onOpenChange={setIsOpen}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{t("offices.delete.title")}</DialogTitle>
          <DialogDescription>
            {/* TODO_TM: change the description with medical appointments once the feature is live */}
            {t("offices.delete.description")}
          </DialogDescription>
        </DialogHeader>
        <div className="flex items-center space-x-3">
          <Switch
            checked={alsoDeletePatients}
            onCheckedChange={setAlsoDeletePatients}
          />
          <Label className="cursor-pointer">
            {"Aussi supprimer tous les patients liés à ce cabinet ?"}
          </Label>
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={() => setIsOpen(false)}>
            {t("common.cancel")}
          </Button>
          <Button variant="destructive" onClick={handleDelete}>
            {t("common.delete")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};
