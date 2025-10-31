"use client";

import { useCallback } from "react";
import useSWR from "swr";

// Error consistent with the API error responses.
// That is error that originates from the server-side.
export interface ApiFetchError {
  message: string;
  status: number;
}

// That error is highly TBD, as I am not sure how to handle errors
// I am planning to use that as a error that originated from the client side, caused by the client mistakes.
// For example, validation errors, network errors, timeouts, etc.
// TODO: We could think about making that a class extending the built-in Error class, but I do not know how errors work in
// javascript in-depth.
export interface ApiClientError {
  message: string;
}

// class ApiClientError extends Error {
//   constructor(message: string) {
//     super(message);
//     this.name = "ApiClientError";
//   }

//   static fromMessage(message: string): ApiClientError {
//     return new ApiClientError(message);
//   }
// }

/**
    General purpose hook to fetch data from the API. Prefixes the path with API version if not present.

    If custom behavior of the useSWR hook is needed, use useSWR directly.
    Thought it would be possible to just pass the configuration object here.

    NOTE: The naming is ambiguous, as this is not a general fetch hook, but an API fetch hook, though
    we are not fetching other resources and assume that this the the server API we are working with.
 */
export function useFetch<Data, Error = any>(
  endpoint: string,
  options: RequestInit = {},
) {
  const callback = useCallback(
    () => fetcher<Data>(endpoint, options),
    [endpoint, options],
  );

  return useSWR<Data, Error>(endpoint, callback);
}

// I believe it is better to move the fetcher outside of the hook to prevent
// to prevent the re-allocation on each render and call to useFetch.
export async function fetcher<Data>(
  path: string,
  options: RequestInit = {},
): Promise<Data> {
  const API_VERSION = "/api/v1";
  let endpoint = path.startsWith("/") ? path : `/${path}`;

  if (!path.startsWith(API_VERSION)) {
    endpoint = API_VERSION + endpoint;
  }

  console.log("Sending headers: ", {
    headers: {
      "Content-Type": "application/json",
      ...(options?.headers || {}),
    },
    ...options,
  });

  const res = await fetch(endpoint, {
    headers: {
      "Content-Type": "application/json",
      ...(options?.headers || {}),
    },
    ...options,
  });

  const data = await res.json();

  if (!res.ok) {
    // Tha should be of type ApiFetchError
    throw data as ApiFetchError;
  }

  return data;
}
