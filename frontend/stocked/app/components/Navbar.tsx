import Link from "next/link";

// Navbar but on the side, kind off sidebar, but not really, just an interpretation.
function Navbar() {
  return (
    <nav className="h-fit w-2xs rounded text-lg">
      <ul className="flex w-full flex-col gap-2">
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

export default Navbar;
