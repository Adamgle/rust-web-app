"use client";

import { useParams } from "next/navigation";
import { useFetch } from "../../../api/hooks/useFetch";
import StockEntry from "../../components/StockEntry";
import { Stock } from "../../../api/types/schema";
import { useEffect, useState } from "react";

export default function Page() {
  const { id } = useParams();

  const { data: stock, error, isLoading } = useFetch<Stock>(`/stocks/${id}`);

  const [prices, setPrices] = useState(Array(100).map((_) => 0));

  // TODO: This repeats the connection which is very bad I suppose, done just for testing.
  useEffect(() => {
    const endpoint = new URL("/api/v1/sse", process.env.NEXT_PUBLIC_SERVER_URL);

    if (!endpoint) {
      return;
    }

    const source = new EventSource(endpoint);

    source.onmessage = (event) => {
      if (event.data) {
        const data: number[] = JSON.parse(event.data);

        setPrices(data);
      }
    };

    // Bullshit error handling
    source.onerror = () => {
      console.error("Error connecting to SSE server.");
      source.close();
    };

    return () => {
      source.close();
    };
  }, []);

  if (isLoading && !error) return <div>Loading...</div>;

  if (error) return <div>Error loading stock</div>;

  if (stock === undefined) {
    return <div>Stock not found</div>;
  }

  return (
    <div className="flex w-full flex-col gap-4">
      <h2 className="text-xl font-bold">{stock.abbreviation}</h2>
      {/* TODO: price here is hard-coded because I need to think about how to pass that data without creating new EventSource SSE event. */}
      {/* TODO: Shenanigans down there */}
      <StockEntry stock={stock} price={prices[parseInt(id as string)]} />
    </div>
  );
}
