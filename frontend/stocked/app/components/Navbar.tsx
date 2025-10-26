import Link from "next/link";

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

export default Navbar;
