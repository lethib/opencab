import { useMutation } from "@tanstack/react-query";
import { APIClient } from "../api";
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

type GenerateCompanyInvoiceBody = {
  invoice_date: string;
  description: string;
  quantity: number;
  unit_price_ht: number;
  vat_rate: string;
  practitioner_office_id: number;
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
  generateInvoice: (companyId: number) => ({
    useMutation: () =>
      useMutation({
        mutationFn: async (data: GenerateCompanyInvoiceBody) => {
          const response = await APIClient.client.post<{
            pdf_data: string;
            filename: string;
          }>(`/companies/${companyId}/_generate_invoice`, data);

          const binaryString = atob(response.data.pdf_data);
          const bytes = new Uint8Array(binaryString.length);
          for (let i = 0; i < binaryString.length; i++) {
            bytes[i] = binaryString.charCodeAt(i);
          }
          const blob = new Blob([bytes], { type: "application/pdf" });

          return { blob, filename: response.data.filename };
        },
      }),
  }),
};
