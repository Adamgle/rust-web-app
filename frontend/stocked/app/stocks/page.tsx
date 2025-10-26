"use client";

import { useFetch } from "../../api/hooks/useFetch";
import { Stock } from "../../types/types";
import StockEntry from "../components/StockEntry";

export default function Page() {
  const { data: stocks, error, isLoading } = useFetch<Stock[]>("/stocks");

  if (isLoading && !error) return <div>Loading...</div>;

  if (error) return <div>Error loading stocks</div>;

  if (stocks === undefined || stocks.length === 0) {
    return <div>No stocks available</div>;
  }

  return (
    <div className="w-full flex flex-col gap-4 p-8">
      <h2 className="text-xl font-bold mb-4">Stocks</h2>
      <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-4">
        {stocks.map((stock) => (
          <StockEntry stock={stock} key={stock.id} />
        ))}
      </div>
    </div>
  );
}
