import { QueryClient } from "@tanstack/react-query";
import axios, {
  AxiosError,
  type AxiosInstance,
  type AxiosRequestConfig,
} from "axios";
import { logout } from "@/lib/authUtils";
import { APIHooks } from "./hooks";
import { toast } from "sonner";
import { t } from "i18next";

export type APIError = {
  code: number;
  msg: string;
};

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
        if (error.response?.status === 401) {
          if (error.response.data.msg !== "invalid_credentials") {
            logout();
            return;
          }
        }

        toast.error(t('errors.global'), { description: error.response?.data.msg });

        throw error;
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
});
