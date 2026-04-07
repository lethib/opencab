import { Edit, FileText, Trash2 } from "lucide-react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import type { MedicalAppointment } from "@/api/hooks/patient";
import { Button } from "@/components/ui";
import { TableCell, TableRow } from "@/components/ui/table";

interface Props {
  appointment: MedicalAppointment;
  index: number;
  onClickGenerateInvoice: (appointment: MedicalAppointment) => Promise<null>;
  onClickEdit: (appointment: MedicalAppointment) => void;
  onClickDelete: (appointment: MedicalAppointment) => void;
}

export const AppointmentRow = ({
  appointment,
  index,
  onClickDelete,
  onClickEdit,
  onClickGenerateInvoice,
}: Props) => {
  const { t } = useTranslation();
  return (
    <TableRow
      key={appointment.id}
      className={`transition-colors hover:bg-muted/30 ${
        index % 2 === 0 ? "bg-background" : "bg-muted/10"
      }`}
    >
      <TableCell className="px-6 py-4">
        <span className="font-medium">
          {new Date(appointment.date).toLocaleDateString()}
        </span>
      </TableCell>
      <TableCell className="px-4 py-4 text-muted-foreground">
        {appointment.office.name}
      </TableCell>
      <TableCell className="px-4 py-4">
        <span className="font-mono font-medium">
          {(appointment.price_in_cents / 100).toFixed(2)} €
        </span>
      </TableCell>
      <TableCell align="center" className="px-4 py-4">
        <span className="font-medium">
          {appointment.payment_method
            ? t(`paymentMethods.${appointment.payment_method}`)
            : t("common.unspecified")}
        </span>
      </TableCell>
      <TableCell align="right" className="space-x-1">
        <Button
          variant="outline"
          size="sm"
          onClick={(e) => {
            e.stopPropagation();
            toast.promise(onClickGenerateInvoice(appointment), {
              loading: t("invoice.modal.sendingInvoice"),
              success: t("invoice.modal.invoiceSent"),
            });
          }}
          className="h-8 w-8 p-0"
          title={t("invoice.modal.generate")}
        >
          <FileText className="h-4 w-4" />
        </Button>
        <Button
          variant="outline"
          size="sm"
          className="h-8 w-8 p-0"
          onClick={(e) => {
            e.stopPropagation();
            onClickEdit(appointment);
          }}
        >
          <Edit className="h-4 w-4" />
        </Button>
        <Button
          variant="ghost_destructive"
          size="sm"
          className="h-8 w-8 p-0"
          onClick={(e) => {
            e.stopPropagation();
            onClickDelete(appointment);
          }}
        >
          <Trash2 />
        </Button>
      </TableCell>
    </TableRow>
  );
};
