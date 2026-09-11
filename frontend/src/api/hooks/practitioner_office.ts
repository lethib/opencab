import { mutationEndpoint } from "../endpointGenerator";

export type PractitionerOffice = {
  id: number;
  name: string;
  address_line_1: string;
  address_zip_code: string;
  address_city: string;
  revenue_share_percentage: number;
};

export type PractitionerOfficeParams = {
  office: {
    name: string;
    address_line_1: string;
    address_zip_code: string;
    address_city: string;
  };
  revenue_share_percentage: number;
};

type DeleteOfficeParams = {
  also_delete_patients: boolean;
};

export const practitionerOfficeSchema = {
  createOffice: mutationEndpoint<
    PractitionerOfficeParams,
    { success: boolean }
  >({
    type: "POST",
    path: "/practitioner_office/create",
  }),
  updateOffice: mutationEndpoint<
    PractitionerOfficeParams,
    { success: boolean }
  >({
    type: "PUT",
    path: "/practitioner_office/{office_id}",
  }),
  deleteOffice: mutationEndpoint<DeleteOfficeParams, { success: true }>({
    type: "DELETE",
    path: "/practitioner_office/{office_id}",
  }),
};
