"use client";

import useSWR from "swr";
import { getStocks } from "../../../api/stocks";
import { Stock } from "../../../types/types";

export default function ProductsPage() {
  const { data, error, isLoading } = useSWR<Stock[]>(
    "/api/v1/stocks",
    getStocks,
    {
      revalidateOnMount: true,
      revalidateIfStale: false,
      revalidateOnFocus: false,
      revalidateOnReconnect: false,
    }
  );

  console.log(data);

  if (!isLoading && data == undefined) return <div>No products found</div>;

  if (error) return <div>Error loading products</div>;

  return (
    <div>
      {data?.map((stock) => (
        <div key={stock.id}>
          {stock.abbreviation} | {stock.price}
        </div>
      ))}
    </div>
  );
}
