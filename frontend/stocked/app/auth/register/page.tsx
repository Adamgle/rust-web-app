"use client";

import { ApiError } from "next/dist/server/api-utils";
import {
  ApiClientError,
  ApiFetchError,
  fetcher,
} from "../../../api/hooks/useFetch";
import {
  LoginPage,
  LoginPageProps,
  validatePasswordPolicy,
} from "../login/page";
import { SessionUser } from "../../../api/hooks/getAuthSessions";

const handleRegister: LoginPageProps["onSubmit"] = async (e) => {
  console.log("Register form submitted");
  e.preventDefault();

  const formData = new FormData(e.currentTarget);

  const { email, password } = Object.fromEntries(formData) || {};

  console.log({ email, password });

  if (!email || !password || !validatePasswordPolicy(password.toString())) {
    // Kind of shenanigans here.
    const error = { message: "Invalid email or password" } as ApiClientError;
    console.log("Validation error:", error);

    return error;
  }

  try {
    // That can only return ApiFetchError so the catch error is probably of that type.
    const data = await fetcher<SessionUser>("/auth/register", {
      method: "POST",
      body: JSON.stringify({ email, password }),
    });

    console.log("Registration successful:", data);
    return data;
  } catch (error) {
    // TODO: I don't know what happens here, that coercion is probably unsafe.
    console.error(error);
    throw error;
  }
};

export default function Page() {
  return <LoginPage title={"Register Form"} onSubmit={handleRegister} />;
}
