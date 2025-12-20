"use client";

import { useEffect, useState } from "react";
import { useFetch } from "../../api/hooks/useFetch";
import { Stock } from "../../api/types/schema";
import StockEntry from "../components/StockEntry";

export default function Page() {
  const { data: stocks, error, isLoading } = useFetch<Stock[]>("/stocks");

  if (isLoading && !error) return <div>Loading...</div>;

  if (error) return <div>Error loading stocks</div>;

  if (stocks === undefined || stocks.length === 0) {
    return <div>No stocks available</div>;
  }

  return (
    <div className="flex w-full flex-col gap-4 p-8">
      <h2 className="mb-4 text-xl font-bold">Stocks</h2>
      <div className="grid grid-cols-2 gap-4 md:grid-cols-3 lg:grid-cols-4">
        {stocks.map((stock) => (
          <StockEntry stock={stock} key={stock.id} price={1} />
        ))}
      </div>
    </div>
  );
}
