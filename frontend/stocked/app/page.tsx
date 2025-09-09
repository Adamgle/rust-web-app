"use client";

import Link from "next/link";
import useSWR from "swr";
import { Stock } from "../types/types";
import { getStocks } from "../api/stocks";

function Logo() {
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

function Profile() {
  return (
    <div className="flex flex-col gap-2 h-fit text-sm">
      <div className="flex flex-row gap-4">
        <div className="border rounded-sm p-2">Name</div>
        <div className="border rounded-sm p-2">Image</div>
      </div>
      <button className="bg-blue-500 text-white font-bold p-1 rounded text-sm">
        Balance $0.0
      </button>
    </div>
  );
}

// Navbar but on the side, kind off sidebar, but not really, just an interpretation.
function Navbar() {
  // Installation Directory: C:\Program Files\PostgreSQL\17
  // Server Installation Directory: C:\Program Files\PostgreSQL\17
  // Data Directory: C:\Program Files\PostgreSQL\17\data
  // Database Port: 5432
  // Database Superuser: postgres
  // Operating System Account: NT AUTHORITY\NetworkService
  // Database Service: postgresql-x64-17
  // Command Line Tools Installation Directory: C:\Program Files\PostgreSQL\17
  // pgAdmin4 Installation Directory: C:\Program Files\PostgreSQL\17\pgAdmin 4
  // Stack Builder Installation Directory: C:\Program Files\PostgreSQL\17
  // Installation Log: C:\Users\adamb\AppData\Local\Temp\install-postgresql.log

  return (
    <nav className="w-2xs h-fit rounded text-lg">
      <ul className="flex flex-col gap-2 w-full">
        <Link href="me/stocks" className="font-bold">
          Your Stocks
        </Link>
        <Link href="trade" className="font-bold">
          Trade
        </Link>
        <Link href="me/actions" className="font-bold">
          Trading Actions
        </Link>
        <Link href="market" className="font-bold">
          Market
        </Link>
        <Link href="watchlist" className="font-bold">
          Watchlist
        </Link>
        <Link href="account/settings" className="font-bold">
          Settings
        </Link>
      </ul>
    </nav>
  );
}

function Stocks() {
  const {
    data: stocks,
    error,
    isLoading,
  } = useSWR<Stock[]>("/api/v1/stocks", getStocks);

  if (error) return <div>Error loading stocks</div>;

  if (isLoading) return <div>Loading...</div>;

  if (stocks === undefined || stocks.length === 0) {
    return <div>No stocks available</div>;
  }

  return (
    <main className="w-full">
      <h2 className="text-xl font-bold mb-4">Stocks</h2>

      <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-4">
        {stocks.map((stock) => (
          <div
            key={stock.id}
            className="rounded-2xl bg-gray-800 text-white p-4 shadow-md hover:scale-105 transition"
          >
            <h3 className="text-lg font-semibold">{stock.abbreviation}</h3>
            <p className="text-sm">Price: ${stock.price}</p>
            <p className={`text-sm font-bold ${stock.delta}`}>
              {stock.company}
            </p>
          </div>
        ))}
      </div>
    </main>
  );
}
// TODO: This header should be in the layout, probably it would have to be inherited in each page.
function Header() {
  return (
    <header className="flex flex-row w-full justify-center">
      <div className="flex flex-col gap-8 w-full max-w-5xl">
        <div className="flex flex-row justify-between">
          <Logo />
          <Profile />
        </div>
      </div>
    </header>
  );
}

export default function Home() {
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
