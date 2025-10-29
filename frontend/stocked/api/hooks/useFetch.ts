"use client";

import { useCallback } from "react";
import useSWR, { SWRResponse } from "swr";

// #[derive(serde::Serialize)]
// pub struct ErrorResponse<'a> {
//     // TODO: Define the appropriate fields for the error response
//     // it will be serialized into JSON and pushed to the client.
//     // I think it will be error-agnostic, meaning each variant will
//     // produce the client error of the same structure.
//     pub message: Cow<'a, str>,
//     #[serde(with = "serde_status_code")]
//     pub status: axum::http::StatusCode,
// }

// Error consistent with the API error responses.
export interface ApiFetchError {
  message: string;
  status: number;
}

/**
    General purpose hook to fetch data from the API. Prefixes the path with API version if not present.

    If custom behavior of the useSWR hook is needed, use useSWR directly.
    Thought it would be possible to just pass the configuration object here.

    NOTE: The naming is ambiguous, as this is not a general fetch hook, but an API fetch hook, though
    we are not fetching other resources and assume that this the the server API we are working with.
 */
export function useFetch<Data, Error = any>(
  path: string,
  options?: RequestInit
) {
  const API_VERSION = "/api/v1";
  let endpoint = path.startsWith("/") ? path : `/${path}`;

  if (!path.startsWith(API_VERSION)) {
    endpoint = API_VERSION + endpoint;
  }

  const callback = useCallback(
    () => fetcher<Data>(endpoint, options),
    [endpoint, options]
  );

  return useSWR<Data, Error>(endpoint, callback);
}

// I believe it is better to move the fetcher outside of the hook to prevent
// to prevent the re-allocation on each render and call to useFetch.
async function fetcher<Data>(
  url: string,
  options?: RequestInit
): Promise<Data> {
  const res = await fetch(url, options);

  if (!res.ok) {
    throw await res.json();
  }

  return res.json();
}
