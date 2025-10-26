import { useFetch } from "../../api/hooks/useFetch";
import { Stock } from "../../types/types";
import StockEntry from "./StockEntry";

export function Stocks() {
  const { data: stocks, error, isLoading } = useFetch<Stock[]>("/stocks");

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
          <StockEntry stock={stock} key={stock.id} />
        ))}
      </div>
    </main>
  );
}
