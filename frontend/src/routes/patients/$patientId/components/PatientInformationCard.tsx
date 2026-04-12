import { Edit, User } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { APIHooks } from "@/api/hooks";
import { PatientModal } from "@/components/PatientModal/PatientModal";
import {
  Button,
  Card,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui";
import { CenteredSpineer } from "@/components/ui/spinner";
import { formatSSN } from "@/lib/utils";

interface Props {
  patientId: number;
}

export const PatientInformationCard = ({ patientId }: Props) => {
  const { t } = useTranslation();
  const patientQuery = APIHooks.patient.getPatient(patientId).useQuery(null);
  const updatePatientMutation = APIHooks.patient.updatePatient.useMutation({
    patient_id: patientId,
  });
  const [isEditPatientModalOpen, setIsEditPatientModalOpen] = useState(false);
  const patient = patientQuery.data;

  if (!patient) {
    return <CenteredSpineer />;
  }

  return (
    <>
      <Card>
        <CardHeader>
          <div className="flex justify-between items-start">
            <div className="flex items-center gap-4">
              <div className="flex h-16 w-16 items-center justify-center rounded-full bg-primary/10">
                <User className="h-8 w-8 text-primary" />
              </div>
              <div>
                <CardTitle className="text-2xl">
                  {patient.first_name} {patient.last_name}
                </CardTitle>
                <CardDescription className="flex flex-col gap-1 mt-2">
                  <span>
                    {t("patients.table.ssn")}:{" "}
                    {patient.ssn ? formatSSN(patient.ssn) : "-"}
                  </span>
                  <span>
                    {patient.address_line_1}, {patient.address_zip_code}{" "}
                    {patient.address_city}
                  </span>
                  {patient.email && (
                    <span>
                      Email:{" "}
                      <Button
                        variant="link"
                        className="-mt-2 -ml-1"
                        onClick={() =>
                          (window.location.href = `mailto:${patient.email}`)
                        }
                      >
                        {patient.email}
                      </Button>
                    </span>
                  )}
                </CardDescription>
              </div>
            </div>
            <Button
              variant="outline"
              onClick={() => setIsEditPatientModalOpen(true)}
              className="flex items-center gap-2"
            >
              <Edit className="h-4 w-4" />
              {t("common.edit")}
            </Button>
          </div>
        </CardHeader>
      </Card>

      <PatientModal
        open={isEditPatientModalOpen}
        asyncMutation={updatePatientMutation.mutateAsync}
        onOpenChange={() => setIsEditPatientModalOpen(false)}
        selectedPatient={patient}
      />
    </>
  );
};
