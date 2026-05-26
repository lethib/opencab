import { mutationEndpoint, queryEndpoint } from "../endpointGenerator";

export type Company = {
  id: number;
  name: string;
  contact_name: string;
  contact_email: string;
  siret: string | null;
  address_line_1: string | null;
  address_zip_code: string | null;
  address_city: string | null;
  address_country: string | null;
};

type CompanyParams = {
  name: string;
  contact_name: string;
  contact_email: string;
  siret?: string;
  address_line_1?: string;
  address_zip_code?: string;
  address_city?: string;
};

export const companySchema = {
  list: queryEndpoint<null, Company[]>({
    type: "GET",
    path: "/companies",
  }),
  get: (companyId: number) =>
    queryEndpoint<null, Company>({
      type: "GET",
      path: `/companies/${companyId}`,
    }),
  create: mutationEndpoint<CompanyParams, void>({
    type: "POST",
    path: "/companies",
  }),
  update: (companyId: number) =>
    mutationEndpoint<CompanyParams, void>({
      type: "PUT",
      path: `/companies/${companyId}`,
    }),
};
