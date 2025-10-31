"use client";

import { ApiFetchError, useFetch } from "./useFetch";

export interface SessionUser {
  name: string;
  image?: string;
  balance: string;
}

// interface SessionUserError extends FetchError {}

// Client side function to authorize the access to protected routes and resources and validate
// that the user is logged in and is valid.
//
// Requests the server endpoint that would validate the session stored in the cookies, or
// in the case of the JWT, I guess, from the "Authorization" header.
export function getSessionUser(): {
  user?: SessionUser;
  error?: ApiFetchError;
} {
  // We have two options:
  // 1. We would expose endpoint that would validate the object stored in the cookies to be valid and exists in the database
  // by using some kind of ID, checking if that exists in the database
  // That way there is really not need to check if that is valid, there is not even such criteria.
  // That would put huge load on the server, as every request would need to go to the database.
  // 2. We would need to store some kind of JWT token in the cookies that would be validated on every request.
  // I do not know how to work with JWTs.
  // Authorization: bearer <TOKEN>

  // useEffect(() => {
  //   // If we would do the JWTs that would send the token in the Authorization header.
  //   // Server would verify it that it is correctly signed and not expired.
  //   // JWT would be stored in the cookies, probably HttpOnly cookies to prevent XSS attacks.
  //   // JWT would be created when the user logs in, and then store that in cookies.
  //   // I do not know how would the expiration look like, something would have to check if the cookies is not expired,
  //   // and given that the JWT is somehow encrypted that would be probably the server,
  //   // so the request when we would verify the session would determine that the token is expired
  //   // and probably generate the new one, but actually no, it would have to log out the user,
  //   // and prompt the login again, and then generate the new token if valid credentials.

  //   const { data, error, isLoading } = useFetch("auth/session");
  //   console.log("Auth session data:", data, error, isLoading);

  //   const intervalSubscriber = setInterval(() => {});

  //   return () => {
  //     clearInterval(intervalSubscriber);
  //   };
  // }, []);

  const {
    data: user,
    error,
    isLoading,
  } = useFetch<SessionUser, ApiFetchError>("auth/session");

  // TODO: Think about how to handle errors on the client side.
  // I would also imagine that the errors from the client side would be persisted somewhere
  // for later analysis, but then that would have to call some endpoint logging the error into the database.
  // I believe Postgresql has a JSON support from know on, and that could be handy
  // for logging unstructured errors from the client side.

  if (error) {
    // NOTE: Maybe that should log the error to database or something for later analysis.
    // But that is not really an error, it's just the user is not logged in.

    return { user: undefined, error };
  }

  // NOTE: It is possible that an infinite recursion will happen on that throw, as useFetch will retry infinitely,
  // when that condition will repeatedly comply
  if (!isLoading && !user) throw new Error("Failed to fetch auth session");

  // TODO: user needs to be validated against the database.
  // UPDATE: no

  return { user, error: undefined };
}
