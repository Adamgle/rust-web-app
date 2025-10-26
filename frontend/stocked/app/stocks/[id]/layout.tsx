import React from "react";

function Layout({ children }: { children: React.ReactNode }) {
  return <div className="w-full max-w-lg p-8">{children}</div>;
}

export default Layout;
