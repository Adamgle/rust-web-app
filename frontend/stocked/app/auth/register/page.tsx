"use client";

import { ApiClientError } from "../../../api/hooks/useFetch";
import {
  LoginPage,
  LoginPageProps,
  validatePasswordPolicy,
} from "../login/page";
import { fetcher } from "../../../utils/fetcher";
import { SessionUser } from "../../../api/types/schema";

const handleRegister: LoginPageProps["onSubmit"] = async (e) => {
  e.preventDefault();

  const formData = new FormData(e.currentTarget);

  const { email, password } = Object.fromEntries(formData) || {};

  if (!email || !password || !validatePasswordPolicy(password.toString())) {
    // Kind of shenanigans here.
    const error = { message: "Invalid email or password" } as ApiClientError;
    console.error("Validation error: ", error);

    return error;
  }

  try {
    // That can only return ApiFetchError so the catch error is probably of that type.
    const data = await fetcher<SessionUser>("/auth/register", {
      method: "POST",
      body: JSON.stringify({ email: email.toString().toLowerCase(), password }),
    });

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
