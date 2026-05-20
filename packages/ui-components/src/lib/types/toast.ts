// Define the Toast interface
export interface ToastProps {
  message: string;
  detail?: string;
  type: "success" | "error" | "warning" | "info";
  color: "green" | "red" | "yellow" | "blue";
  links?: ToastLink[];
}

export type ToastLink = {
  link: string;
  label: string;
};
