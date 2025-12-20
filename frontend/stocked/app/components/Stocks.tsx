import { useEffect, useState } from "react";
import { useFetch } from "../../api/hooks/useFetch";
import { Stock } from "../../api/types/schema";
import StockEntry from "./StockEntry";
import { assert } from "node:console";

export function Stocks() {
  const { data: stocks, error, isLoading } = useFetch<Stock[]>("/stocks");
  const [price, setPrice] = useState(Array(100).map((_) => 0));

  useEffect(() => {
    const endpoint = new URL("/api/v1/sse", process.env.NEXT_PUBLIC_SERVER_URL);

    if (!endpoint) {
      return;
    }

    const source = new EventSource(endpoint);

    source.onmessage = (event) => {
      if (event.data) {
        const data: number[] = JSON.parse(event.data);

        setPrice(data);
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

  if (error) return <div>Error loading stocks</div>;

  if (isLoading) return <div>Loading...</div>;

  if (stocks === undefined || stocks.length === 0) {
    return <div>No stocks available</div>;
  }

  return (
    <main className="w-full">
      <h2 className="mb-4 text-xl font-bold">Stocks</h2>
      <div className="grid grid-cols-2 gap-4 md:grid-cols-3 lg:grid-cols-4">
        {stocks.map((stock, i) => (
          <StockEntry stock={stock} key={stock.id} price={price[i]} />
        ))}
      </div>
    </main>
  );
}
