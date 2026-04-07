import { APIHooks } from "@/api/hooks";
import type { MedicalAppointment } from "@/api/hooks/patient";
import { CenteredSpineer } from "@/components/ui/spinner";
import { TableBody, TableCell, TableRow } from "@/components/ui/table";
import { AppointmentRow } from "./AppointmentRow";

interface Props {
  patientId: number;
  onClickEditAppointment: (appointment: MedicalAppointment) => void;
  onClickDeleteAppointment: (appointment: MedicalAppointment) => void;
  onClickGenerateInvoice: (appointment: MedicalAppointment) => Promise<null>;
}

export const AppointmentsList = ({
  patientId,
  onClickEditAppointment,
  onClickDeleteAppointment,
  onClickGenerateInvoice,
}: Props) => {
  const medicalAppointmentsQuery = APIHooks.patient
    .getMedicalAppointments(patientId)
    .useQuery(null);

  if (medicalAppointmentsQuery.isFetching) {
    return (
      <TableBody>
        <TableRow>
          <TableCell colSpan={7} className="h-32 text-center">
            <CenteredSpineer />
          </TableCell>
        </TableRow>
      </TableBody>
    );
  }

  return (
    <TableBody>
      {medicalAppointmentsQuery.data?.map((appointment, index) => (
        <AppointmentRow
          key={index}
          appointment={appointment}
          index={index}
          onClickEdit={onClickEditAppointment}
          onClickDelete={onClickDeleteAppointment}
          onClickGenerateInvoice={onClickGenerateInvoice}
        />
      ))}
    </TableBody>
  );
};
