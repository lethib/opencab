import { Calendar, Euro, HandCoins, MapPin } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { queryClient } from "@/api/api";
import { APIHooks } from "@/api/hooks";
import type { MedicalAppointment } from "@/api/hooks/patient";
import { Table, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { AppointmentModal } from "../AppointmentModal";
import { AppointmentsList } from "./AppointmentsList";

interface Props {
  patientId: number;
}

export const AppointmentsTable = ({ patientId }: Props) => {
  const { t } = useTranslation();
  const [isAppointmentModalOpen, setIsAppointmentModalOpen] = useState(false);
  const [selectedAppointment, setSelectedAppointment] = useState<
    MedicalAppointment | undefined
  >(undefined);

  const updateAppointmentMutation = APIHooks.patient
    .updateMedicalAppointment(patientId, selectedAppointment?.id ?? 0)
    .useMutation();

  const deleteAppointmentMutation = APIHooks.patient
    .deleteMedicalAppointment(patientId, selectedAppointment?.id ?? 0)
    .useMutation();

  const generateInvoiceMutation = APIHooks.patient
    .medicalAppointment(patientId)
    .generateInvoice(selectedAppointment?.id ?? 0)
    .useMutation();

  const handleOnClickEdit = (appointment: MedicalAppointment) => {
    setSelectedAppointment(appointment);
    setIsAppointmentModalOpen(true);
  };

  const handleOnClickDelete = (appointment: MedicalAppointment) => {
    setSelectedAppointment(appointment);
    deleteAppointmentMutation.mutateAsync(null).then(() => {
      queryClient.invalidateQueries({
        queryKey: [`/patient/${patientId}/medical_appointments`, null],
      });
    });
  };

  const handleOnClickGenerateInvoice = (appointment: MedicalAppointment) => {
    setSelectedAppointment(appointment);
    return generateInvoiceMutation.mutateAsync(null);
  };

  return (
    <>
      <div className="rounded-lg border bg-card">
        <Table>
          <TableHeader>
            <TableRow className="border-b bg-muted/50">
              <TableHead className="h-12 px-6 font-semibold text-foreground">
                <div className="flex items-center gap-2">
                  <Calendar className="h-4 w-4" />
                  {t("appointments.table.date")}
                </div>
              </TableHead>
              <TableHead className="h-12 px-4 font-semibold text-foreground">
                <div className="flex items-center gap-2">
                  <MapPin className="h-4 w-4" />
                  {t("appointments.table.office")}
                </div>
              </TableHead>
              <TableHead className="h-12 px-4 font-semibold text-foreground">
                <div className="flex items-center gap-2">
                  <Euro className="h-4 w-4" />
                  {t("appointments.table.price")}
                </div>
              </TableHead>
              <TableHead className="h-12 px-4 font-semibold text-foreground text-center">
                <div className="flex items-center justify-center gap-2">
                  <HandCoins className="h-4 w-4" />
                  {t("appointments.table.paymentMethod")}
                </div>
              </TableHead>
              <TableHead className="h-12 px-4 font-semibold text-foreground"></TableHead>
            </TableRow>
          </TableHeader>

          <AppointmentsList
            patientId={patientId}
            onClickEditAppointment={handleOnClickEdit}
            onClickDeleteAppointment={handleOnClickDelete}
            onClickGenerateInvoice={handleOnClickGenerateInvoice}
          />
        </Table>
      </div>

      <AppointmentModal
        open={isAppointmentModalOpen}
        asyncMutation={updateAppointmentMutation.mutateAsync}
        selectedAppointment={selectedAppointment}
        onOpenChange={(open) => {
          setIsAppointmentModalOpen(open);
          if (!open) {
            setSelectedAppointment(undefined);
          }
        }}
        patientId={patientId}
      />
    </>
  );
};
