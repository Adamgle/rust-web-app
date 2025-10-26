"use client";

import Header from "./components/Header";
import Navbar from "./components/Navbar";
import { Stocks } from "./components/Stocks";

// TODO: This header should be in the layout, probably it would have to be inherited in each page.

export default function Page() {
  // return <h1 className="text-2xl">Stocked | Make beaucoup-Bucks</h1>;
  return (
    <div className="flex flex-col items-center justify-center gap-12 w-full">
      <Header />
      <div className="flex w-full max-w-5xl gap-8">
        <Navbar />
        <Stocks />
      </div>
    </div>
  );
}
