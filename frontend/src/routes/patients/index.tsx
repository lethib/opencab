import { createFileRoute, useNavigate } from "@tanstack/react-router";
import {
  FileDown,
  LogOut,
  Plus,
  Search as SearchIcon,
  UserCog,
} from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { APIHooks } from "@/api/hooks";
import { PatientModal } from "@/components/PatientModal/PatientModal";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Input } from "@/components/ui/input";
import { useDebounce } from "@/hooks/useDebounce";
import { logout } from "@/lib/authUtils";
import { ExportAccountabilityModal } from "./components/ExportAccountabilityModal";
import { ExportAppointmentsModal } from "./components/ExportAppointmentsModal";
import { PatientsTable } from "./components/PatientsTable/PatientsTable";

export const Route = createFileRoute("/patients/")({
  component: Patients,
});

function Patients() {
  const { t } = useTranslation();
  const [isAddPatientModalOpened, setIsAddPatientModalOpened] = useState(false);
  const [isExportAppointmentsModalOpen, setIsExportAppointmentsModalOpen] =
    useState(false);
  const [isAccountabilityExportModalOpen, setIsAccountabilityExportModalOpen] =
    useState(false);
  const [searchQuery, setSearchQuery] = useState("");
  const debouncedSearchQuery = useDebounce(searchQuery, 700);
  const navigate = useNavigate();

  const addPatientMutation = APIHooks.patient.createPatient.useMutation();

  const handleOnOpenChange = (value: boolean) => {
    setIsAddPatientModalOpened(value);
  };

  return (
    <>
      <div className="min-h-screen bg-gradient-to-br from-background via-background to-muted/20">
        <div className="container mx-auto px-4 py-8">
          {/* Header */}
          <div className="flex flex-col md:flex-row justify-between items-start md:items-center gap-4 mb-8">
            <div>
              <img src="/favicon/opencab-text.png" width={200} />
              <p className="text-muted-foreground">{t("patients.subtitle")}</p>
            </div>
            <div className="flex gap-4">
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
              <DropdownMenu>
                <DropdownMenuTrigger asChild>
                  <Button variant="ghost">
                    <UserCog className="size-6 text-primary" />
                  </Button>
                </DropdownMenuTrigger>
                <DropdownMenuContent side="bottom" align="end">
                  <DropdownMenuLabel>
                    {t("navigation.account")}
                  </DropdownMenuLabel>
                  <DropdownMenuSeparator />
                  <DropdownMenuItem
                    onClick={() => navigate({ to: "/my_information" })}
                  >
                    {t("navigation.myInformation")}
                  </DropdownMenuItem>
                  <DropdownMenuItem
                    onClick={() => navigate({ to: "/offices" })}
                  >
                    {t("navigation.myOffices")}
                  </DropdownMenuItem>
                  <DropdownMenuItem onClick={logout} variant="destructive">
                    {t("auth.logout")} <LogOut />
                  </DropdownMenuItem>
                </DropdownMenuContent>
              </DropdownMenu>
            </div>
          </div>

          {/* Search Bar */}
          <div className="relative mb-8">
            <SearchIcon className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-muted-foreground" />
            <Input
              placeholder={t("patients.searchPlaceholder")}
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="pl-10 h-12 text-base"
            />
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
