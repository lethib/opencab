import { mutationEndpoint, queryEndpoint } from "../endpointGenerator";

export type CompanyIntervention = {
  id: number;
  company_id: number;
  practitioner_id: number;
  quantity: number;
  unit_price_in_cents: number;
  vat_rate_in_percent: string;
  issue_date: string;
  object: string;
  created_at: string;
  updated_at: string;
};

export const companyInterventionsSchema = {
  list: (companyId: number) =>
    queryEndpoint<null, CompanyIntervention[]>({
      type: "GET",
      path: `/companies/${companyId}/interventions`,
    }),
  delete: (companyId: number, interventionId: number) =>
    mutationEndpoint<null, null>({
      type: "DELETE",
      path: `/companies/${companyId}/interventions/${interventionId}`,
    }),
};
