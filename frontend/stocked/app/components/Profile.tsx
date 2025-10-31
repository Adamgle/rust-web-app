"use client";

import Link from "next/link";
import { getSessionUser } from "../../api/hooks/getAuthSessions";

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
  const { user, error } = getSessionUser();

  if (!user || error) {
    return <LoginButton />;
  }

  const { name, image, balance } = user;

  return (
    <div className="flex h-fit flex-col gap-2 text-sm">
      <div className="flex flex-row gap-4">
        <div className="rounded-sm border p-2">{name}</div>
        <div className="rounded-sm border p-2">{image}</div>
      </div>
      <button className="rounded bg-blue-500 p-1 text-sm font-bold text-white">
        Balance {balance}
      </button>
    </div>
  );
}

export function Logo() {
  return (
    <div className="flex flex-col items-center justify-center gap-2">
      <h1 className="self-start rounded bg-purple-500 px-4 py-2 font-bold text-white saturate-100">
        Stocked
      </h1>
      <button className="rounded bg-blue-500 p-3 text-sm font-bold text-white">
        Make beaucoup-Bucks
      </button>
    </div>
  );
}
