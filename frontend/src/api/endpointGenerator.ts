import {
  type UseMutationOptions,
  type UseQueryOptions,
  useMutation,
  useQuery,
} from "@tanstack/react-query";
import type { AxiosError } from "axios";
import { APIClient, type APIError } from "./api";

// Base endpoint configuration type
export type EndpointConfig<P, R> = {
  type: "POST" | "GET" | "PUT" | "DELETE";
  path: string;
  params?: P;
  response?: R;
};

export type PaginationMetaData = {
  page: number;
  per_page: number;
  total_pages: number;
  has_more: boolean;
};

export type Paginated<D> = {
  paginated_data: D[];
  pagination: PaginationMetaData;
};

// Generic hook generators
function createMutation<P, R>(endpoint: EndpointConfig<P, R>) {
  return (
    pathParams?: Record<string, number>,
    options?: Omit<
      UseMutationOptions<R, AxiosError<APIError>, P>,
      "mutationFn"
    >,
  ) => {
    let finalRoute = endpoint.path;

    if (pathParams) {
      Object.entries(pathParams).forEach(([key, value]) => {
        finalRoute = finalRoute.replace(`{${key}}`, value.toString());
      });
    }

    return useMutation<R, AxiosError<APIError>, P>({
      mutationFn: async (data: P) => {
        switch (endpoint.type) {
          case "POST":
            return await APIClient.post<P, R>(finalRoute, data);
          case "PUT":
            return await APIClient.put<P, R>(finalRoute, data);
          case "DELETE":
            return await APIClient.delete<R>(finalRoute);
          default:
            throw new Error("Type not implemented");
        }
      },
      ...options,
    });
  };
}

function createQuery<P, R>(endpoint: EndpointConfig<P, R>) {
  return (
    params: P,
    options?: Omit<
      UseQueryOptions<R, AxiosError<APIError>, R>,
      "queryKey" | "queryFn"
    >,
  ) => {
    return useQuery({
      queryKey: [endpoint.path, params],
      queryFn: async () => {
        return await APIClient.get<R>(endpoint.path, { params });
      },
      ...options,
    });
  };
}

export const queryEndpoint = <P, R>(config: { type: "GET"; path: string }) => {
  return { useQuery: createQuery<P, R>(config) };
};

export const mutationEndpoint = <P, R>(config: {
  type: "POST" | "PUT" | "DELETE";
  path: string;
}) => {
  return { useMutation: createMutation<P, R>(config) };
};
