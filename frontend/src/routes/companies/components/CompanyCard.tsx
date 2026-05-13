import { HashIcon, Mail, MapPin, User } from "lucide-react";
import type { Company } from "@/api/hooks/practitioner_company";
import { Card, CardContent } from "@/components/ui/card";

interface Props {
  company: Company;
}

const getInitials = (name: string) =>
  name
    .split(/\s+/)
    .filter(Boolean)
    .slice(0, 2)
    .map((word) => word[0].toUpperCase())
    .join("");

export const CompanyCard = ({ company }: Props) => {
  return (
    <Card className="hover:shadow-md transition-shadow cursor-pointer">
      <CardContent className="space-y-4">
        <div className="flex items-center gap-3">
          <div className="flex-shrink-0 w-10 h-10 rounded-lg bg-primary/15 flex items-center justify-center">
            <span className="text-sm font-semibold text-primary">
              {getInitials(company.name)}
            </span>
          </div>
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
          {company.address_line_1 && (
            <div className="flex items-center gap-2">
              <MapPin className="h-4 w-4 flex-shrink-0" />
              <span>
                {company.address_line_1}
                {company.address_zip_code && `, ${company.address_zip_code}`}
                {company.address_city && `, ${company.address_city}`}
              </span>
            </div>
          )}
        </div>
      </CardContent>
    </Card>
  );
};
