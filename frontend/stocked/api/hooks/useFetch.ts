"use client";

import { useCallback } from "react";
import useSWR, { SWRResponse } from "swr";

/**
    General purpose hook to fetch data from the API. Prefixes the path with API version if not present.

    If custom behavior of the useSWR hook is needed, use useSWR directly.
    Thought it would be possible to just pass the configuration object here.

    NOTE: The naming is ambiguous, as this is not a general fetch hook, but an API fetch hook, though
    we are not fetching other resources and assume that this the the server API we are working with.
 */
export function useFetch<Data>(
  path: string,
  options?: RequestInit
): SWRResponse<Data> {
  const API_VERSION = "/api/v1";
  let endpoint = path.startsWith("/") ? path : `/${path}`;

  if (!path.startsWith(API_VERSION)) {
    endpoint = API_VERSION + endpoint;
  }

  const callback = useCallback(
    () => fetcher<Data>(endpoint, options),
    [endpoint, options]
  );

  return useSWR<Data>(endpoint, callback);
}

// I believe it is better to move the fetcher outside of the hook to prevent
// to prevent the re-allocation on each render and call to useFetch.
async function fetcher<Data>(
  url: string,
  options?: RequestInit
): Promise<Data> {
  return fetch(url, options).then((res) => res.json());
}
