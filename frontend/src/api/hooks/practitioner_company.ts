import { mutationEndpoint, queryEndpoint } from "../endpointGenerator";

export type Company = {
  id: number;
  name: string;
  contact_email: string;
  address_line_1: string | null;
  address_zip_code: string | null;
  address_country: string | null;
};

type CompanyParams = {
  name: string;
  contact_email: string;
  address_line_1?: string;
  address_zip_code?: string;
};

export const companySchema = {
  list: queryEndpoint<null, Company[]>({
    type: "GET",
    path: "/companies",
  }),
  create: mutationEndpoint<CompanyParams, void>({
    type: "POST",
    path: "/companies",
  }),
};
