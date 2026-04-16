import { useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { APIHooks } from "@/api/hooks";
import {
  Table,
  TableCell,
  TableFooter,
  TableHead,
  TableHeader,
  TablePagination,
  TableRow,
} from "@/components/ui/table";
import { PatientList } from "./PatientsList";

interface Props {
  searchQuery: string;
}

export const PatientsTable = ({ searchQuery }: Props) => {
  const { t } = useTranslation();
  const [page, setPage] = useState(1);
  const prevSearchQueryRef = useRef(searchQuery);

  // Reset page during render when searchQuery changes (prevents double requests)
  if (prevSearchQueryRef.current !== searchQuery) {
    prevSearchQueryRef.current = searchQuery;
    if (page !== 1) {
      setPage(1);
    }
  }

  const searchPatientsQuery = APIHooks.patient.search.useQuery({
    q: searchQuery,
    page,
  });

  const paginationData = searchPatientsQuery.data?.pagination;

  return (
    <div className="space-y-4">
      <div className="rounded-lg border bg-card overflow-hidden">
        <Table>
          <TableHeader>
            <TableRow className="border-b bg-muted/50">
              <TableHead className="h-12 px-6 font-semibold text-foreground">
                {t("patients.table.name")}
              </TableHead>
              <TableHead className="h-12 px-4 font-semibold text-foreground">
                {t("patients.table.ssn")}
              </TableHead>
              <TableHead className="h-12 px-4 font-semibold text-foreground">
                {t("patients.table.address")}
              </TableHead>
              <TableHead className="h-12 px-4 font-semibold text-foreground">
                {t("patients.table.zip_code")}
              </TableHead>
              <TableHead className="h-12 px-4 font-semibold text-foreground">
                {t("patients.table.city")}
              </TableHead>
              <TableHead className="h-12 px-4 font-semibold text-foreground">
                {t("patients.table.actions", "Actions")}
              </TableHead>
            </TableRow>
          </TableHeader>

          <PatientList
            patientsList={searchPatientsQuery.data?.paginated_data}
            isDataFetching={searchPatientsQuery.isFetching}
          />

          <TableFooter>
            <TableRow>
              <TableCell colSpan={8}>
                {paginationData && (
                  <TablePagination
                    currentPage={page}
                    setCurrentPage={setPage}
                    paginationData={paginationData}
                  />
                )}
              </TableCell>
            </TableRow>
          </TableFooter>
        </Table>
      </div>
    </div>
  );
};
