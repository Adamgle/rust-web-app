"use client";

import Link from "next/link";
import { getSessionUser } from "../../api/hooks/getAuthSessions";

function LoginButton() {
  return (
    <Link
      href="/login"
      className="bg-gray-700 text-white font-bold p-2 rounded h-fit text-xs self-center"
    >
      Login
    </Link>
  );
}

export function Profile() {
  const sessionUser = getSessionUser();

  if (!sessionUser) {
    return <LoginButton />;
  }

  const { name, image, balance } = sessionUser;

  return (
    <div className="flex flex-col gap-2 h-fit text-sm">
      <div className="flex flex-row gap-4">
        <div className="border rounded-sm p-2">{name}</div>
        <div className="border rounded-sm p-2">{image}</div>
      </div>
      <button className="bg-blue-500 text-white font-bold p-1 rounded text-sm">
        Balance {balance}
      </button>
    </div>
  );
}

export function Logo() {
  return (
    <div className="flex flex-col items-center justify-center gap-2">
      <h1 className="bg-purple-500 text-white font-bold px-4 py-2 rounded saturate-100 self-start">
        Stocked
      </h1>
      <button className="bg-blue-500 text-white font-bold p-3 rounded text-sm">
        Make beaucoup-Bucks
      </button>
    </div>
  );
}
