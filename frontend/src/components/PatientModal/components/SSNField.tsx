import { IdCard } from "lucide-react";
import type { ChangeEvent } from "react";
import { useFormContext } from "react-hook-form";
import { useTranslation } from "react-i18next";
import { FormInput } from "@/components/form/FormInput";
import { Label } from "@/components/ui";

interface SSNFieldProps {
  onChange: (e: ChangeEvent<HTMLInputElement>) => void;
}

const formatSSN = (value: string) => {
  const digits = value.replace(/\D/g, "");
  if (digits.length <= 1) return digits;
  if (digits.length <= 3) return `${digits[0]} ${digits.slice(1)}`;
  if (digits.length <= 5)
    return `${digits[0]} ${digits.slice(1, 3)} ${digits.slice(3)}`;
  if (digits.length <= 7)
    return `${digits[0]} ${digits.slice(1, 3)} ${digits.slice(3, 5)} ${digits.slice(5)}`;
  if (digits.length <= 10)
    return `${digits[0]} ${digits.slice(1, 3)} ${digits.slice(3, 5)} ${digits.slice(5, 7)} ${digits.slice(7)}`;
  if (digits.length <= 13)
    return `${digits[0]} ${digits.slice(1, 3)} ${digits.slice(3, 5)} ${digits.slice(5, 7)} ${digits.slice(7, 10)} ${digits.slice(10)}`;

  return `${digits[0]} ${digits.slice(1, 3)} ${digits.slice(3, 5)} ${digits.slice(5, 7)} ${digits.slice(7, 10)} ${digits.slice(10, 13)} ${digits.slice(13, 15)}`;
};

export const SSNField = ({ onChange }: SSNFieldProps) => {
  const { t } = useTranslation();
  const { watch } = useFormContext();

  const ssnValue = watch("ssn") || "";

  return (
    <div className="space-y-2">
      <Label htmlFor="ssn" className="text-sm font-medium" optional>
        {t("patients.form.ssn")}
      </Label>
      <FormInput
        id="ssn"
        name="ssn"
        type="text"
        onChange={onChange}
        value={formatSSN(ssnValue)}
        placeholder={t("patients.form.ssnPlaceholder")}
        className="pl-10 h-11"
        icon={
          <IdCard className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
        }
      />
    </div>
  );
};
