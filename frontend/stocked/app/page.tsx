"use client";

import Header from "./components/Header";
import Navbar from "./components/Navbar";
import { Stocks } from "./components/Stocks";

export default function Page() {
  return (
    <div className="flex w-full flex-col items-center justify-center gap-12 p-8">
      <Header />
      <div className="flex w-full max-w-5xl gap-8">
        <Navbar />
        <Stocks />
      </div>
    </div>
  );
}
