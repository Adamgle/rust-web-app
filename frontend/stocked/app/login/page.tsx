"use client";

export function LoginForm() {
  return (
    <form onSubmit={() => void 0} className="flex h-fit flex-col gap-4">
      <label htmlFor="email">
        <input
          type="email"
          id="email"
          name="email"
          placeholder="email"
          required
          className="rounded bg-gray-200 p-2 text-sm font-bold text-black"
        />
      </label>
      <label htmlFor="password">
        <input
          type="password"
          id="password"
          name="password"
          placeholder="password"
          required
          className="rounded bg-gray-200 p-2 text-sm font-bold text-black"
        />
      </label>
    </form>
  );
}

const Page = () => {
  return (
    // flex h-screen flex-col items-center justify-center gap-2 w-full max-w-sm p-4 bg-gray-950
    <div className="flex h-screen w-full items-center justify-center">
      <div className="flex h-full w-full max-w-sm flex-col items-center justify-center gap-4 bg-gray-900 p-4 text-white">
        <h1 className="text-lg font-bold">Login Page</h1>
        <LoginForm />
        {/* TODO: DO the registration form, register button below. */}
      </div>
    </div>
  );
};

export default Page;
