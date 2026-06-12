import { type ClassValue, clsx } from "clsx";
import { format } from "date-fns";
import { fr } from "date-fns/locale";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export function formatAccessKey(accessKey: string): string {
  // Remove any characters except alphanumeric and hyphens, then return first 15 characters
  return accessKey
    .replace(/[^a-zA-Z0-9-]/g, "")
    .slice(0, 15)
    .toUpperCase();
}

export const formatSSN = (ssn: string): string =>
  `${ssn[0]} ${ssn.slice(1, 3)} ${ssn.slice(3, 5)} ${ssn.slice(5, 7)} ${ssn.slice(7, 10)} ${ssn.slice(10, 13)} ${ssn.slice(13, 15)}`;

export const formatAddress = (address: {
  address_line_1?: string | null;
  address_zip_code?: string | null;
  address_city?: string | null;
  address_country?: string | null;
}): string =>
  [
    address.address_line_1,
    [address.address_zip_code, address.address_city].filter(Boolean).join(" "),
    address.address_country,
  ]
    .filter(Boolean)
    .join(", ");

export const getInitials = (name: string): string =>
  name
    .split(/\s+/)
    .filter(Boolean)
    .slice(0, 2)
    .map((word) => word[0].toUpperCase())
    .join("");

export const formatPrice = (cents: number) => `${(cents / 100).toFixed(2)} €`;

export const formatDate = (dateStr: string) =>
  format(new Date(dateStr), "dd/MM/yyyy", { locale: fr });
