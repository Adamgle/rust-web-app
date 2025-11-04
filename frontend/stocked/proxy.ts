import { cookies } from "next/headers";

import { RequestCookie } from "next/dist/compiled/@edge-runtime/cookies";
import { ReadonlyRequestCookies } from "next/dist/server/web/spec-extension/adapters/request-cookies";
import type { NextRequest } from "next/server";
import { NextResponse } from "next/server";
import { fetcher } from "./utils/fetcher";

function formatCookieJar(jar: ReadonlyRequestCookies): string {
  // NOTE: There is possibly an issues, maybe that is related to the backend,
  // but the cookies from that jar are not containing things like HttpOnly or Secure
  // and expiration, maybe that is related to how they are handled on the backend, when registering.

  return jar
    .getAll()
    .map((c: RequestCookie) => `${c.name}=${c.value}`)
    .join("; ");
}

/// Requests the API to check if there is an authenticated user session and it is valid, never revalidates the request.
async function redirectAuthenticatedUser(
  to: URL | string = new URL("/", process.env.NEXT_PUBLIC_CLIENT_URL) || "/",
) {
  if (!process.env.NEXT_PUBLIC_CLIENT_URL) {
    throw new Error(
      "NEXT_PUBLIC_CLIENT_URL env is not defined, cannot cast param to URL using string.",
    );
  }

  console.log("Checking for authenticated user session for redirect...");

  const jar = await cookies();

  const response = await fetcher("/auth/session", {
    headers: {
      cookie: formatCookieJar(jar),
    },
  })
    // If there is no error, the user is authenticated and should be redirected.
    .then(() => {
      return NextResponse.redirect(to);
    })
    // On error, do nothing, we could log that to database thought for analysis.
    .catch((_) => null);

  return response || NextResponse.next();
}

// This function can be marked `async` if using `await` inside
export async function proxy(request: NextRequest) {
  const { pathname } = request.nextUrl;

  console.log(`Proxy: [${request.method}] ${pathname}`);

  try {
    switch (pathname) {
      case "/auth/login":
      case "/auth/register":
        switch (request.method) {
          // POST does not make any sense here,
          // when someone is doing POST on that route, we are handling that on the server,
          // that is strictly for the client-side to not ship the page to authenticated users.
          case "GET":
            return await redirectAuthenticatedUser();
        }
      // This proxy is I presume called on the edge, meaning we would have to host that to vercel, we cannot achieve
      // that logic with our server as that is the separate layer, actually the logic could be achieved easily but not this
      // good, as this redirect before even loading the page is better for UX and engines.

      default:
        const next = NextResponse.next();
        console.log("No proxy match, continuing request normally: ", next);

        return next;
    }
  } catch (error) {
    console.error("Error in proxy middleware:", error);

    return NextResponse.next();
  }
}

// See "Matching Paths" below to learn more
export const config = {
  matcher: ["/auth/login", "/auth/register"],
};
