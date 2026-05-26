import { Building2, MapPin, Pencil, Trash2 } from "lucide-react";
import { useTranslation } from "react-i18next";
import type { PractitionerOffice } from "@/api/hooks/practitioner_office";
import { formatAddress } from "@/lib/utils";
import {
  Button,
  Card,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui";

interface OfficeCardProps {
  office: PractitionerOffice;
  onEdit: VoidFunction;
  onDelete: VoidFunction;
}

export const OfficeCard = ({ office, onEdit, onDelete }: OfficeCardProps) => {
  const { t } = useTranslation();

  return (
    <Card className="hover:shadow-lg transition-shadow">
      <CardHeader>
        <div className="flex items-start justify-between">
          <div className="flex-1">
            <CardTitle className="flex items-center gap-2">
              <Building2 className="h-5 w-5 text-primary" />
              {office.name}
            </CardTitle>
            <CardDescription className="flex items-center gap-2 mt-2">
              <MapPin className="h-4 w-4" />
              <span>{formatAddress(office)}</span>
            </CardDescription>
          </div>
          <div className="flex gap-2">
            <Button
              variant="ghost"
              size="icon"
              onClick={onEdit}
              aria-label={t("common.edit")}
            >
              <Pencil className="h-4 w-4" />
            </Button>
            <Button
              variant="ghost_destructive"
              size="icon"
              onClick={onDelete}
              aria-label={t("common.delete")}
            >
              <Trash2 className="h-4 w-4" />
            </Button>
          </div>
        </div>
      </CardHeader>
    </Card>
  );
};
