"use client";

import Link from "next/link";
import { getSessionUser } from "../../api/hooks/getAuthSessions";
import { ApiStatusResponse } from "../../api/hooks/useFetch";
import { fetcher } from "../../utils/fetcher";
import { useSWRConfig } from "swr";

const handleLogout = async (e: React.FormEvent<HTMLFormElement>) => {
  e.preventDefault();

  try {
    await fetcher<ApiStatusResponse>("/auth/logout", { method: "POST" });
  } catch (error) {
    console.error("Logout failed:", error);

    throw error;
  }
};

export function LoginButton() {
  return (
    <Link
      href="/auth/login"
      className="h-fit self-center rounded bg-gray-700 p-2 text-xs font-bold text-white"
    >
      Login
    </Link>
  );
}

export function Profile() {
  // This causes flicker as that is client side render.
  const { data, error, mutate } = getSessionUser();

  if (!data || error) {
    return <LoginButton />;
  }

  const { email, balance } = data;

  const handleLogoutRevalidate = async (
    e: React.FormEvent<HTMLFormElement>,
  ) => {
    await handleLogout(e);

    // That mutate is pre-bound and will revalidate the getSessionUser after logging out.
    mutate();
  };

  return (
    <div className="flex h-fit flex-col gap-2 text-sm">
      <div className="flex flex-row gap-4">
        <div className="flex flex-row items-center justify-center gap-2">
          <form onSubmit={handleLogoutRevalidate}>
            <button
              className="cursor-pointer rounded-sm border p-2 font-bold"
              type="submit"
            >
              Logout
            </button>
          </form>
          <div className="rounded-sm border p-2">{email}</div>
        </div>
        {/* Future components ... */}
      </div>
      <div className="flex flex-row gap-2 rounded border-2 bg-blue-500 p-1 pl-2 text-left text-sm font-bold text-white">
        Balance{" "}
        <div className="flex flex-row items-center justify-center gap-0.5">
          <span className="font-extrabold">{balance}</span>
          <span>$</span>
        </div>
      </div>
    </div>
  );
}

export function Logo() {
  return (
    <div className="flex flex-col items-center justify-center gap-2">
      <h1 className="self-start rounded bg-purple-500 px-4 py-2 font-bold text-white saturate-100">
        Stocked
      </h1>
      <button className="bg-blue-500 p-3 text-sm font-bold text-white">
        Make beaucoup-Bucks
      </button>
    </div>
  );
}
