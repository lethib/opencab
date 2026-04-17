import { createFileRoute } from "@tanstack/react-router";
import { Building2, Plus } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { APIHooks } from "@/api/hooks";
import type { PractitionerOffice } from "@/api/hooks/practitioner_office";
import { Button } from "@/components/ui/button";
import { CenteredSpineer } from "@/components/ui/spinner";
import { H2 } from "@/components/ui/typography/h2";
import { DeleteOfficeDialog } from "./components/DeleteOfficeDialog";
import { OfficeCard } from "./components/OfficeCard";
import { OfficeModal } from "./components/OfficeModal";

export const Route = createFileRoute("/offices/")({
  component: Offices,
});

function Offices() {
  const { t } = useTranslation();
  const [isAddOfficeModalOpened, setIsAddOfficeModalOpened] = useState(false);
  const [officeToEdit, setOfficeToEdit] = useState<PractitionerOffice | null>(
    null,
  );
  const [officeToDelete, setOfficeToDelete] =
    useState<PractitionerOffice | null>(null);

  const officesQuery = APIHooks.user.getMyOffices.useQuery(null);
  const createOfficeMutation = APIHooks.office.createOffice.useMutation();
  const updateOfficeMutation = APIHooks.office.updateOffice.useMutation(
    officeToEdit
      ? {
          office_id: officeToEdit.id,
        }
      : undefined,
  );

  const handleOnEdit = (office: PractitionerOffice) => {
    setOfficeToEdit(office);
    setIsAddOfficeModalOpened(true);
  };

  return (
    <>
      <div className="min-h-screen bg-gradient-to-br from-background via-background to-muted/20">
        <div className="container mx-auto">
          {/* Header */}
          <div className="flex flex-col md:flex-row justify-between items-start md:items-center gap-4 mb-8 px-2">
            <div>
              <H2 className="text-3xl font-bold mb-2 flex items-center gap-2">
                <Building2 className="h-8 w-8" />
                {t("offices.title")}
              </H2>
              <p className="text-muted-foreground">{t("offices.subtitle")}</p>
            </div>
            <Button
              onClick={() => setIsAddOfficeModalOpened(true)}
              className="flex items-center gap-2"
            >
              <Plus className="h-4 w-4" />
              {t("offices.addOffice")}
            </Button>
          </div>

          {officesQuery.isFetching && <CenteredSpineer />}

          {/* Empty State */}
          {!officesQuery.isLoading &&
            !officesQuery.isError &&
            officesQuery.data?.length === 0 && (
              <div className="text-center py-12">
                <Building2 className="h-12 w-12 mx-auto mb-4 text-muted-foreground" />
                <p className="text-muted-foreground mb-4">
                  {t("offices.noOffices")}
                </p>
                <Button onClick={() => setIsAddOfficeModalOpened(true)}>
                  <Plus className="h-4 w-4 mr-2" />
                  {t("offices.addFirstOffice")}
                </Button>
              </div>
            )}

          {/* Offices List */}
          {officesQuery.data && officesQuery.data.length > 0 && (
            <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
              {officesQuery.data.map((office) => (
                <OfficeCard
                  key={office.id}
                  office={office}
                  onEdit={() => handleOnEdit(office)}
                  onDelete={() => setOfficeToDelete(office)}
                />
              ))}
            </div>
          )}
        </div>
      </div>

      {/* Add/Edit Modal */}
      <OfficeModal
        open={isAddOfficeModalOpened}
        setIsOpen={setIsAddOfficeModalOpened}
        asyncMutation={
          officeToEdit
            ? updateOfficeMutation.mutateAsync
            : createOfficeMutation.mutateAsync
        }
        office={officeToEdit}
      />

      {officeToDelete && (
        <DeleteOfficeDialog
          open={!!officeToDelete}
          setIsOpen={(open) => !open && setOfficeToDelete(null)}
          office={officeToDelete}
        />
      )}
    </>
  );
}
