"use client";

import { useCallback } from "react";
import useSWR, { SWRConfiguration, SWRHook } from "swr";
import { fetcher } from "../../utils/fetcher";

// It would be ideal if we would have some kind of mapping between the server API response
// and client return types, but not sure it is possible with the technology I am using.
// We could define some generic return types, because there could be some structure with the
// API response and use predefined types for that.
// Of course there are also the arbitrary types for each endpoint, and then
// we would use generic, but some of them are just returning something like
// { status: boolean }, or { status: boolean, message: string }, etc.
// and we should strive for that.

/// Response to indicate simple boolean status from the server.
export interface ApiStatusResponse {
  status: boolean;
}

/// Serve response with a message from the server and a status.
export interface ApiMessageResponse {
  status: boolean;
  message: string;
}

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

    Alongside with the useSWR return type, also returns the key used for fetching, if any mutations or 
    revalidation would be needed, so not to rely on remembering that information or looking it up in the code
    if the useFetch is wrapped is some other function.
    
    If custom behavior of the useSWR hook is needed, use useSWR directly.
    Thought it would be possible to just pass the configuration object here.

    NOTE: The naming is ambiguous, as this is not a general fetch hook, but an API fetch hook, though
    we are not fetching other resources and assume that this the the server API we are working with.
 */
export function useFetch<Data, Error = any>(
  endpoint: string,
  options: RequestInit = {},
  config: SWRConfiguration = {},
) {
  const callback = useCallback(
    () => fetcher<Data>(endpoint, options),
    [endpoint, options],
  );

  return {
    ...useSWR<Data, Error>(endpoint, callback, config),
    // useSWR: { key: endpoint },
  };
}
