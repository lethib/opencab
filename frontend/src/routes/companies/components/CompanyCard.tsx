import { useNavigate } from "@tanstack/react-router";
import { HashIcon, Mail, MapPin, User } from "lucide-react";
import type { Company } from "@/api/hooks/practitioner_company";
import { Card, CardContent } from "@/components/ui/card";
import { formatAddress } from "@/lib/utils";
import { CompanyAvatar } from "./CompanyAvatar";

interface Props {
  company: Company;
}

export const CompanyCard = ({ company }: Props) => {
  const navigate = useNavigate({ from: "/companies" });
  const address = formatAddress(company);

  return (
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
          <div>
            <h3 className="font-semibold text-base leading-tight flex-1 min-w-0">
              {company.name}
            </h3>
            {company.siret && (
              <div className="flex items-center gap-1 text-sm text-muted-foreground">
                <HashIcon className="h-3 w-3 flex-shrink-0" />
                <span>{company.siret}</span>
              </div>
            )}
          </div>
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
  );
};
