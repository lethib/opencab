import { useMutation } from "@tanstack/react-query";
import type { UUID } from "crypto";
import { APIClient } from "../api";
import {
  mutationEndpoint,
  type Paginated,
  queryEndpoint,
} from "../endpointGenerator";
import type { PractitionerOffice } from "./practitioner_office";

export type SavePatientParams = {
  first_name: string;
  last_name: string;
  email?: string;
  ssn: string;
  address_line_1: string;
  address_zip_code: string;
  address_city: string;
};

interface SearchPatientParams {
  q: string;
  page: number;
}

export type SearchPatientResponse = {
  id: number;
  pid: UUID;
  first_name: string;
  last_name: string;
  email: string | null;
  ssn: string;
  address_line_1: string;
  address_zip_code: string;
  address_city: string;
  address_country: string;
};

export type MedicalAppointment = {
  id: number;
  date: string;
  price_in_cents: number;
  payment_method: PaymentMethod | null;
  office: PractitionerOffice;
};

export const PAYMENT_METHODS = ["Cash", "Card", "Check", "Transfer"] as const;
export type PaymentMethod = (typeof PAYMENT_METHODS)[number];

export type MedicalAppointmentParams = {
  date: string;
  price_in_cents: number;
  practitioner_office_id: number;
  payment_method: PaymentMethod | null;
};

export const patientSchema = {
  createPatient: mutationEndpoint<SavePatientParams, { success: boolean }>({
    type: "POST",
    path: "/patient/create",
  }),
  getPatient: (patientId: number) =>
    queryEndpoint<null, SearchPatientResponse>({
      type: "GET",
      path: `/patient/${patientId}`,
    }),
  getMedicalAppointments: (patientId: number) =>
    queryEndpoint<null, MedicalAppointment[]>({
      type: "GET",
      path: `/patient/${patientId}/medical_appointments`,
    }),
  createMedicalAppointment: (patientId: number) =>
    mutationEndpoint<MedicalAppointmentParams, null>({
      type: "POST",
      path: `/patient/${patientId}/medical_appointments`,
    }),
  updateMedicalAppointment: (patientId: number, appointmentId: number) =>
    mutationEndpoint<MedicalAppointmentParams, null>({
      type: "PUT",
      path: `/patient/${patientId}/medical_appointments/${appointmentId}`,
    }),
  deleteMedicalAppointment: (patientId: number, appointmentId: number) =>
    mutationEndpoint<null, null>({
      type: "DELETE",
      path: `/patient/${patientId}/medical_appointments/${appointmentId}`,
    }),
  medicalAppointment: (patientId: number) => ({
    generateInvoice: (appointmentId: number) =>
      mutationEndpoint<null, null>({
        type: "POST",
        path: `/patient/${patientId}/medical_appointments/${appointmentId}/_generate_invoice`,
      }),
  }),
  updatePatient: mutationEndpoint<SavePatientParams, { success: boolean }>({
    type: "PUT",
    path: "/patient/{patient_id}",
  }),
  deletePatient: mutationEndpoint<null, null>({
    type: "DELETE",
    path: "/patient/{patient_id}",
  }),
  search: queryEndpoint<SearchPatientParams, Paginated<SearchPatientResponse>>({
    type: "GET",
    path: "/patient/_search",
  }),
  generateInvoice: {
    useMutation: () => {
      return useMutation({
        mutationFn: async ({
          patientId,
          amount,
          invoice_date,
          should_be_sent_by_email,
          practitioner_office_id,
          payment_method,
        }: {
          patientId: number;
          amount: number;
          invoice_date: string;
          should_be_sent_by_email: boolean;
          practitioner_office_id: number;
          payment_method: PaymentMethod | null;
        }) => {
          const response = await APIClient.client.post<{
            pdf_data: string;
            filename: string;
          }>(`/patient/${patientId}/_generate_invoice`, {
            invoice_params: {
              amount,
              date: invoice_date,
              office_id: practitioner_office_id,
            },
            should_be_sent_by_email,
            payment_method,
          });

          // Decode base64 PDF data to blob
          const pdfData = response.data.pdf_data;
          const binaryString = atob(pdfData);
          const bytes = new Uint8Array(binaryString.length);
          for (let i = 0; i < binaryString.length; i++) {
            bytes[i] = binaryString.charCodeAt(i);
          }
          const blob = new Blob([bytes], { type: "application/pdf" });

          return { blob, filename: response.data.filename };
        },
      });
    },
  },
};
