import { zodResolver } from "@hookform/resolvers/zod";
import type { MutationFunction } from "@tanstack/react-query";
import { type ChangeEvent, useEffect } from "react";
import { useForm } from "react-hook-form";
import { useTranslation } from "react-i18next";
import z from "zod";
import { queryClient } from "@/api/api";
import type {
  SavePatientParams,
  SearchPatientResponse,
} from "@/api/hooks/patient";
import { FormProvider } from "@/components/form/FormProvider";
import {
  Button,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui";
import { PatientFormFields } from "./components/PatientFormFields";
import { SSNField } from "./components/SSNField";

interface Props {
  open: boolean;
  asyncMutation: MutationFunction<{ success: boolean }, SavePatientParams>;
  onOpenChange: (open: boolean) => void;
  selectedPatient?: SearchPatientResponse;
}

const FR_SSN_REGEX =
  /([12])([0-9]{2})(0[1-9]|1[0-2])(2[AB]|[0-9]{2})[0-9]{3}[0-9]{3}([0-9]{2})/;
const FR_ZIP_CODE_REGEX = /^(?:0[1-9]|[1-8]\d|9[0-8])\d{3}$/;

export const PatientModal = ({
  open,
  asyncMutation,
  onOpenChange,
  selectedPatient,
}: Props) => {
  const { t } = useTranslation();

  const isEditMode = !!selectedPatient;
  const currentPatient = selectedPatient;

  const addPatientFormSchema = z.object({
    first_name: z
      .string()
      .trim()
      .min(1, t("patients.form.validation.firstNameRequired")),
    last_name: z
      .string()
      .trim()
      .min(1, t("patients.form.validation.lastNameRequired")),
    email: z.email(t("patients.form.validation.emailRequired")).optional(),
    ssn: z
      .string()
      .length(15)
      .regex(FR_SSN_REGEX, {
        message: t("patients.form.validation.ssnInvalid"),
      })
      .optional(),
    address_line_1: z
      .string()
      .trim()
      .min(1, t("patients.form.validation.addressRequired")),
    address_zip_code: z
      .string()
      .trim()
      .length(5)
      .regex(FR_ZIP_CODE_REGEX, {
        message: t("patients.form.validation.zipCodeInvalid"),
      }),
    address_city: z
      .string()
      .trim()
      .min(1, t("patients.form.validation.cityRequired")),
  });

  const addPatientForm = useForm({
    resolver: zodResolver(addPatientFormSchema),
    defaultValues: {
      first_name: "",
      last_name: "",
      ssn: undefined,
      address_line_1: "",
      address_zip_code: "",
      address_city: "",
    },
  });

  const handleOnClose = () => {
    onOpenChange(false);
    addPatientForm.reset({
      first_name: "",
      last_name: "",
      ssn: undefined,
      email: undefined,
      address_line_1: "",
      address_zip_code: "",
      address_city: "",
    });
  };

  useEffect(() => {
    if (open && currentPatient) {
      addPatientForm.reset({
        first_name: currentPatient.first_name || "",
        last_name: currentPatient.last_name || "",
        ssn: currentPatient.ssn || undefined,
        email: currentPatient.email || undefined,
        address_line_1: currentPatient.address_line_1 || "",
        address_zip_code: currentPatient.address_zip_code || "",
        address_city: currentPatient.address_city || "",
      });
    }
  }, [open, currentPatient]);

  const onSubmit = addPatientForm.handleSubmit(async (values) => {
    asyncMutation({
      ...values,
    })
      .then(() => {
        queryClient.invalidateQueries({ queryKey: ["/patient/_search"] });
        queryClient.invalidateQueries({
          queryKey: [`/patient/${currentPatient?.id}`, null],
        });
        handleOnClose();
      })
      .catch((error) => alert(error.message));
  });

  const handleSSNChange = (e: ChangeEvent<HTMLInputElement>) => {
    const rawValue = e.target.value.replace(/\D/g, "");
    if (rawValue.length <= 15) {
      addPatientForm.setValue("ssn", rawValue === "" ? undefined : rawValue);
    }
  };

  return (
    <Dialog open={open} onOpenChange={handleOnClose}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>
            {t(`patients.form.title.${isEditMode ? "update" : "create"}`)}
          </DialogTitle>
          <DialogDescription>
            {t("patients.form.description")}
          </DialogDescription>
        </DialogHeader>

        <FormProvider
          methods={addPatientForm}
          onSubmit={onSubmit}
          className="space-y-4"
        >
          <SSNField onChange={handleSSNChange} />

          <PatientFormFields />

          <Button type="submit" className="w-full">
            {t(`patients.form.submit.${isEditMode ? "update" : "create"}`)}
          </Button>
        </FormProvider>
      </DialogContent>
    </Dialog>
  );
};
