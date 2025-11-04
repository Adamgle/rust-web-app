"use client";

import { useParams } from "next/navigation";
import { useFetch } from "../../../api/hooks/useFetch";
import StockEntry from "../../components/StockEntry";
import { Stock } from "../../../api/types/schema";

export default function Page() {
  const { id } = useParams();

  const { data: stock, error, isLoading } = useFetch<Stock>(`/stocks/${id}`);

  console.log(stock, error, isLoading);

  if (isLoading && !error) return <div>Loading...</div>;

  if (error) return <div>Error loading stock</div>;

  if (stock === undefined) {
    return <div>Stock not found</div>;
  }

  return (
    <div className="flex w-full flex-col gap-4">
      <h2 className="text-xl font-bold">{stock.abbreviation}</h2>
      <StockEntry stock={stock} />
    </div>
  );
}
