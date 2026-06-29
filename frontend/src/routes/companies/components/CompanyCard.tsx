import { useNavigate } from "@tanstack/react-router";
import { HashIcon, Mail, MapPin, Trash2, User } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";
import type { Company } from "@/api/hooks/practitioner_company";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { formatAddress } from "@/lib/utils";
import { CompanyAvatar } from "./CompanyAvatar";
import { DeleteCompanyDialog } from "./DeleteCompanyDialog";

interface Props {
  company: Company;
}

export const CompanyCard = ({ company }: Props) => {
  const navigate = useNavigate({ from: "/companies" });
  const { t } = useTranslation();
  const address = formatAddress(company);
  const [isDeleteDialogOpen, setIsDeleteDialogOpen] = useState(false);

  return (
    <>
      <Card
        className="hover:shadow-md transition-shadow cursor-pointer"
        onClick={() =>
          navigate({
            to: "$companyId",
            params: { companyId: company.id.toString() },
          })
        }
      >
        <CardContent className="space-y-4">
          <div className="flex items-center gap-3">
            <CompanyAvatar name={company.name} />
            <div className="flex-1 min-w-0">
              <h3 className="font-semibold text-base leading-tight">
                {company.name}
              </h3>
              {company.siret && (
                <div className="flex items-center gap-1 text-sm text-muted-foreground">
                  <HashIcon className="h-3 w-3 flex-shrink-0" />
                  <span>{company.siret}</span>
                </div>
              )}
            </div>
            <Button
              variant="ghost_destructive"
              size="icon"
              aria-label={t("common.delete")}
              onClick={(e) => {
                e.stopPropagation();
                setIsDeleteDialogOpen(true);
              }}
            >
              <Trash2 className="h-4 w-4" />
            </Button>
          </div>

          <div className="space-y-1.5 text-sm text-muted-foreground">
            <div className="flex items-center gap-2">
              <User className="h-4 w-4 flex-shrink-0" />
              <span className="truncate">{company.contact_name}</span>
            </div>
            <div className="flex items-center gap-2">
              <Mail className="h-4 w-4 flex-shrink-0" />
              <span className="truncate">{company.contact_email}</span>
            </div>
            {address && (
              <div className="flex items-center gap-2">
                <MapPin className="h-4 w-4 flex-shrink-0" />
                <span>{address}</span>
              </div>
            )}
          </div>
        </CardContent>
      </Card>

      <DeleteCompanyDialog
        open={isDeleteDialogOpen}
        setIsOpen={setIsDeleteDialogOpen}
        company={company}
      />
    </>
  );
};
