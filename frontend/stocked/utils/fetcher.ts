import { ApiFetchError } from "../api/hooks/useFetch";

export async function fetcher<Data>(
  path: string,
  options: RequestInit = {},
): Promise<Data> {
  const API_VERSION = "/api/v1";
  let endpoint = path.startsWith("/") ? path : `/${path}`;

  if (!path.startsWith(API_VERSION)) {
    endpoint = API_VERSION + endpoint;
  }

  const url = new URL(endpoint, process.env.NEXT_PUBLIC_CLIENT_URL);

  console.log(`Fetching url: ${url} with options:`, {
    headers: {
      "Content-Type": "application/json",
      ...(options?.headers || {}),
    },
    ...options,
  });

  const res = await fetch(url, {
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
