import { useMutation } from "@tanstack/react-query";
import type { AxiosError } from "axios";
import { APIClient, type APIError } from "../api";
import { mutationEndpoint, queryEndpoint } from "../endpointGenerator";
import type { Profession } from "../types/profession";
import type { PractitionerOffice } from "./practitioner_office";

type SaveBusinessInformation = {
  rpps_number: string;
  siret_number: string;
  adeli_number?: string;
  profession: Profession;
};

export const userSchema = {
  saveBusinessInformation: mutationEndpoint<
    SaveBusinessInformation,
    { success: boolean }
  >({
    type: "POST",
    path: "/user/_save_business_information",
  }),
  getMyOffices: queryEndpoint<null, PractitionerOffice[]>({
    type: "GET",
    path: "/user/my_offices",
  }),
  extractMedicalAppointment: mutationEndpoint<
    { start_date: string; end_date: string },
    null
  >({
    type: "POST",
    path: "/user/_extract_medical_appointments",
  }),
  generateAccountability: mutationEndpoint<{ year: number }, null>({
    type: "POST",
    path: "/user/_generate_accountability",
  }),
  signature: {
    getURL: mutationEndpoint<null, string>({
      type: "POST",
      path: "/user/signature/_get_url",
    }),
    upload: {
      useMutation: () => {
        return useMutation<void, AxiosError<APIError>, File>({
          mutationFn: async (file: File) => {
            const formData = new FormData();
            formData.append("signature", file);

            return await APIClient.post<FormData, void>(
              "/user/signature/_upload",
              formData,
              {
                headers: {
                  "Content-Type": "multipart/form-data",
                },
              },
            );
          },
        });
      },
    },
  },
};
