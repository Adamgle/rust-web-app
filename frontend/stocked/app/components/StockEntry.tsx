import Link from "next/link";
import { Stock } from "../../types/types";

export function StockEntry({ stock }: { stock: Stock }) {
  return (
    <Link
      href={`/stocks/${stock.id}`}
      key={stock.id}
      className="rounded-2xl bg-gray-800 text-white p-4 shadow-md hover:scale-105 transition"
    >
      <h3 className="text-lg font-semibold">{stock.abbreviation}</h3>
      <p className="text-sm">Price: ${stock.price}</p>
      <p className={`text-sm font-bold ${stock.delta}`}>{stock.company}</p>
    </Link>
  );
}

export default StockEntry;
