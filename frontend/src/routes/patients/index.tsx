import { createFileRoute } from "@tanstack/react-router";
import { FileDown, Plus, Search as SearchIcon } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { z } from "zod";
import { APIHooks } from "@/api/hooks";
import { PatientModal } from "@/components/PatientModal/PatientModal";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Input } from "@/components/ui/input";
import { useDebounce } from "@/hooks/useDebounce";
import { ExportAccountabilityModal } from "./components/ExportAccountabilityModal";
import { ExportAppointmentsModal } from "./components/ExportAppointmentsModal";
import { PatientsTable } from "./components/PatientsTable/PatientsTable";

const patientSearchSchema = z.object({
  page: z.number().default(1),
  q: z.string().default(""),
});

export const Route = createFileRoute("/patients/")({
  component: Patients,
  validateSearch: patientSearchSchema,
});

function Patients() {
  const { t } = useTranslation();
  const { q } = Route.useSearch();
  const navigate = Route.useNavigate();
  const [isAddPatientModalOpened, setIsAddPatientModalOpened] = useState(false);
  const [isExportAppointmentsModalOpen, setIsExportAppointmentsModalOpen] =
    useState(false);
  const [isAccountabilityExportModalOpen, setIsAccountabilityExportModalOpen] =
    useState(false);
  const debouncedSearchQuery = useDebounce(q, 700);

  const addPatientMutation = APIHooks.patient.createPatient.useMutation();

  const handleOnOpenChange = (value: boolean) => {
    setIsAddPatientModalOpened(value);
  };

  const setSearchQuery = (query: string) => {
    navigate({ search: (prev) => ({ ...prev, q: query }) });
  };

  return (
    <>
      <div className="min-h-screen bg-gradient-to-br from-background via-background to-muted/20">
        <div className="container mx-auto">
          <div className="flex gap-4 mb-8">
            {/* Search Bar */}
            <div className="flex-1 relative">
              <SearchIcon className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-muted-foreground" />
              <Input
                placeholder={t("patients.searchPlaceholder")}
                value={q}
                onChange={(e) => setSearchQuery(e.target.value)}
                className="pl-10 h-12 text-base"
              />
            </div>

            {/* Buttons */}
            <div className="flex flex-col md:flex-row justify-between items-start md:items-center gap-4">
              <DropdownMenu modal={false}>
                <DropdownMenuTrigger asChild>
                  <Button variant="outline">
                    <FileDown />
                    {t("appointments.exportAppointments")}
                  </Button>
                </DropdownMenuTrigger>
                <DropdownMenuContent>
                  <DropdownMenuItem
                    onClick={() => setIsExportAppointmentsModalOpen(true)}
                  >
                    {t("appointments.exportMenu.appointments")}
                  </DropdownMenuItem>
                  <DropdownMenuItem
                    onClick={() => setIsAccountabilityExportModalOpen(true)}
                  >
                    {t("appointments.exportMenu.accountability")}
                  </DropdownMenuItem>
                </DropdownMenuContent>
              </DropdownMenu>

              <Button
                onClick={() => setIsAddPatientModalOpened(true)}
                className="flex items-center gap-2"
              >
                <Plus className="h-4 w-4" />
                {t("patients.addPatient")}
              </Button>
            </div>
          </div>

          <PatientsTable searchQuery={debouncedSearchQuery} />
        </div>
      </div>

      <PatientModal
        open={isAddPatientModalOpened}
        asyncMutation={addPatientMutation.mutateAsync}
        onOpenChange={handleOnOpenChange}
      />

      <ExportAppointmentsModal
        open={isExportAppointmentsModalOpen}
        onOpenChange={setIsExportAppointmentsModalOpen}
      />

      <ExportAccountabilityModal
        open={isAccountabilityExportModalOpen}
        onOpenChange={setIsAccountabilityExportModalOpen}
      />
    </>
  );
}
