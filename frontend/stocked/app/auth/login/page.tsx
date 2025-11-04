"use client";

import Link from "next/link";
import { ApiClientError } from "../../../api/hooks/useFetch";
import { SessionUser } from "../../../api/types/schema";
import { fetcher } from "../../../utils/fetcher";
import { Dispatch, SetStateAction, useState } from "react";
import { useRouter } from "next/navigation";

// export type onSubmitType = (e: React.FormEvent<HTMLFormElement>) => void;

export interface LoginPageProps {
  title?: string;
  onSubmit: (
    e: React.FormEvent<HTMLFormElement>,
    // NOTE: Maybe that should be generic.
  ) => Promise<SessionUser | ApiClientError>;
}

// NOTE: That is untested.
export const validatePasswordPolicy = (password: string): boolean => {
  const MIN_LENGTH = 8;
  const MAX_LENGTH = 128;
  const SPECIAL_CHARACTERS = "!@#$%^&*()-+";
  const REQUIRE_UPPERCASE = true;
  const REQUIRE_DIGIT = true;
  const REQUIRE_LOWERCASE = true;
  const REQUIRE_SPECIAL_CHARACTERS = true;
  const size = password.length;

  if (size < MIN_LENGTH || size > MAX_LENGTH) {
    return false;
  }

  let has_uppercase = !REQUIRE_UPPERCASE;
  let has_lowercase = !REQUIRE_LOWERCASE;
  let has_digit = !REQUIRE_DIGIT;
  let has_special = !REQUIRE_SPECIAL_CHARACTERS;

  for (const char of password) {
    has_uppercase ||= char === char.toUpperCase();
    has_lowercase ||= char === char.toLowerCase();
    has_digit ||= /\d/.test(char);
    has_special ||= SPECIAL_CHARACTERS.includes(char);
  }

  console.log({ has_uppercase, has_lowercase, has_digit, has_special });

  return has_uppercase && has_lowercase && has_digit && has_special;
};

const handleLogin: LoginPageProps["onSubmit"] = async (e) => {
  e.preventDefault();

  const formData = new FormData(e.currentTarget);

  const { email, password } = Object.fromEntries(formData) || {};

  if (!email || !password) {
    // Kind of shenanigans here.
    const error = { message: "Invalid email or password" } as ApiClientError;
    console.error("Validation error: ", error);

    return error;
  }

  try {
    // That can only return ApiFetchError so the catch error is probably of that type.
    const data = await fetcher<SessionUser>("/auth/login", {
      method: "POST",
      body: JSON.stringify({ email, password }),
    });

    return data;
  } catch (error) {
    // TODO: I don't know what happens here, that coercion is probably unsafe.
    console.error(error);

    throw error;
  }
};

function LoginFormSubmit() {
  return (
    <button
      type="submit"
      className="cursor-pointer rounded bg-black p-3 text-sm font-bold text-white"
    >
      Submit
    </button>
  );
}

export function LoginForm({
  onSubmit,
}: {
  onSubmit: LoginPageProps["onSubmit"];
}) {
  const router = useRouter();
  const [error, setError] = useState<string | null>(null);

  const handleOnSubmit = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    setError(null);

    try {
      await onSubmit(e);

      // Redirect on successful login
      router.push("/");
    } catch (error: any) {
      // Capture API or validation errors
      console.error("Login error:", error);

      setError(
        error?.message || "Unexpected error occurred. Please try again.",
      );
    }
  };

  return (
    <form
      onSubmit={handleOnSubmit}
      className="flex h-fit w-full flex-col gap-4"
    >
      {error && (
        <div className="mb-4 rounded border bg-red-100 px-4 py-3 text-red-700">
          <div className="flex items-center justify-between">
            <span>{error}</span>
            <button
              onClick={() => setError(null)}
              className="ml-2 cursor-pointer text-red-700 hover:text-red-900"
            >
              x
            </button>
          </div>
        </div>
      )}
      <label htmlFor="email">
        <input
          type="email"
          id="email"
          name="email"
          placeholder="email"
          required
          className="w-full rounded bg-gray-200 p-3 text-sm font-bold text-black"
        />
      </label>
      <label htmlFor="password">
        <input
          type="password"
          id="password"
          name="password"
          placeholder="password"
          required
          className="w-full rounded bg-gray-200 p-3 text-sm font-bold text-black"
        />
      </label>
      <LoginFormSubmit />
    </form>
  );
}

function RegisterLink({ title }: { title: LoginPageProps["title"] }) {
  return (
    title?.toLowerCase().includes("login") && (
      <div className="border-t border-gray-700 pt-4 text-center">
        <Link
          href="/auth/register"
          className="text-sm text-gray-300 transition-colors hover:text-white"
        >
          Don't have an account?{" "}
          <span className="font-bold text-blue-400 underline">Register</span>
        </Link>
      </div>
    )
  );
}

export function LoginPage({ title, onSubmit }: LoginPageProps) {
  return (
    <div className="flex h-screen w-full items-center justify-center bg-gray-100 p-4">
      <div className="flex w-full max-w-md flex-col gap-6 rounded-lg bg-gray-900 p-8 text-white shadow-lg">
        <div className="text-center">
          <h1 className="mb-2 text-2xl font-bold">{title}</h1>
          {title?.toLowerCase().includes("login") && (
            <p className="text-sm text-gray-400">Welcome back!</p>
          )}
        </div>
        <LoginForm onSubmit={onSubmit} />
        <RegisterLink title={title} />
      </div>
    </div>
  );
}

const Page = () => {
  return <LoginPage title="Login page" onSubmit={handleLogin} />;
};

export default Page;
