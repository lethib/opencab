import { useNavigate } from "@tanstack/react-router";
import { FileText, Trash2, User } from "lucide-react";
import type { SearchPatientResponse } from "@/api/hooks/patient";
import { Button } from "@/components/ui/button";
import { TableCell, TableRow } from "@/components/ui/table";
import { formatSSN } from "@/lib/utils";

interface Props {
  patient: SearchPatientResponse;
  index: number;
  onGenerateInvoice: (patient: SearchPatientResponse) => void;
  onDeletePatient: (patient: SearchPatientResponse) => void;
}

export const PatientRow = ({
  patient,
  index,
  onGenerateInvoice,
  onDeletePatient,
}: Props) => {
  const navigate = useNavigate();

  return (
    <TableRow
      className={`cursor-pointer transition-colors hover:bg-muted/30 ${
        index % 2 === 0 ? "bg-background" : "bg-muted/10"
      }`}
      onClick={() =>
        navigate({
          to: "/patients/$patientId",
          params: { patientId: patient.id.toString() },
        })
      }
    >
      <TableCell className="px-6 py-4">
        <div className="flex items-center gap-3">
          <div className="flex h-10 w-10 items-center justify-center rounded-full bg-primary/10">
            <User className="h-5 w-5 text-primary" />
          </div>
          <div className="flex flex-col">
            <span className="font-bold text-foreground">
              {patient.first_name} {patient.last_name}
            </span>
            <span className="text-xs text-muted-foreground">
              ID: {patient.id}
            </span>
          </div>
        </div>
      </TableCell>
      <TableCell className="px-4 py-4">
        <span className="font-mono text-sm font-medium">
          {patient.ssn ? formatSSN(patient.ssn) : "-"}
        </span>
      </TableCell>
      <TableCell className="px-4 py-4 text-muted-foreground">
        {patient.address_line_1}
      </TableCell>
      <TableCell className="px-4 py-4 text-muted-foreground">
        <span className="font-mono text-sm">{patient.address_zip_code}</span>
      </TableCell>
      <TableCell className="px-4 py-4 text-muted-foreground">
        <span className="text-sm">{patient.address_city}</span>
      </TableCell>
      <TableCell className="space-x-1">
        <Button
          variant="outline"
          size="sm"
          onClick={(e) => {
            e.stopPropagation();
            onGenerateInvoice(patient);
          }}
          className="h-8 w-8 p-0"
          title="Generate Invoice"
        >
          <FileText className="h-4 w-4" />
        </Button>
        <Button
          variant="ghost_destructive"
          size="sm"
          className="h-8 w-8 p-0"
          onClick={(e) => {
            e.stopPropagation();
            onDeletePatient(patient);
          }}
        >
          <Trash2 />
        </Button>
      </TableCell>
    </TableRow>
  );
};
