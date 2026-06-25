import { MutationCache, QueryCache, QueryClient } from "@tanstack/react-query";
import axios, {
  AxiosError,
  type AxiosInstance,
  type AxiosRequestConfig,
} from "axios";
import { t } from "i18next";
import { toast } from "sonner";
import { logout } from "@/lib/authUtils";
import { APIHooks } from "./hooks";

export type APIError = {
  code: number;
  msg: string;
};

declare module "@tanstack/react-query" {
  interface Register {
    mutationMeta: { skipGlobalErrorToast?: boolean };
    queryMeta: { skipGlobalErrorToast?: boolean };
  }
}

export function showGlobalErrorToast(error: unknown) {
  if (!axios.isAxiosError<APIError>(error)) {
    toast.error(t("errors.global"));
    return;
  }

  if (
    error.response?.status === 401 &&
    error.response.data?.msg !== "invalid_credentials"
  ) {
    return;
  }

  toast.error(t("errors.global"), {
    description: error.response?.data?.msg,
  });
}

class MyPatientsAPI {
  client: AxiosInstance;
  hooks: typeof APIHooks;

  constructor(baseURL: string) {
    this.client = axios.create({ baseURL });
    this.hooks = APIHooks;

    this.client.interceptors.request.use(
      (config) => {
        const accessToken = localStorage.getItem("accessToken");
        if (accessToken) {
          config.headers.Authorization = `Bearer ${accessToken}`;
        }
        return config;
      },
      (error: AxiosError) => Promise.reject(error),
    );

    this.client.interceptors.response.use(
      (response) => response,
      (error: AxiosError<APIError>) => {
        if (
          error.response?.status === 401 &&
          error.response.data?.msg !== "invalid_credentials"
        ) {
          logout();
        }

        return Promise.reject(error);
      },
    );
  }

  get = async <R>(path: string, config?: AxiosRequestConfig): Promise<R> => {
    return this.client.get<R>(path, config).then((res) => {
      return res.data;
    });
  };

  post = async <P, R>(
    path: string,
    data: P,
    config?: AxiosRequestConfig,
  ): Promise<R> => {
    return this.client.post<R>(path, data, config).then((res) => {
      return res.data;
    });
  };

  put = async <P, R>(
    path: string,
    data: P,
    config?: AxiosRequestConfig,
  ): Promise<R> => {
    return this.client.put<R>(path, data, config).then((res) => res.data);
  };

  delete = async <R>(path: string, config?: AxiosRequestConfig): Promise<R> => {
    return this.client.delete<R>(path, config).then((res) => res.data);
  };
}

export const APIClient = new MyPatientsAPI(import.meta.env.VITE_BASE_API_URL);

export const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      retry: 2,
      refetchOnWindowFocus: false,
    },
  },
  queryCache: new QueryCache({
    onError: (error, query) => {
      if (!query.meta?.skipGlobalErrorToast) showGlobalErrorToast(error);
    },
  }),
  mutationCache: new MutationCache({
    onError: (error, _variables, _context, mutation) => {
      if (!mutation.meta?.skipGlobalErrorToast) showGlobalErrorToast(error);
    },
  }),
});
