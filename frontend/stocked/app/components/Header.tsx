import HeaderProfile from "./HeaderProfile";

function Header() {
  return (
    <header className="flex flex-row w-full justify-center">
      <div className="flex flex-col gap-8 w-full max-w-5xl">
        <HeaderProfile />
      </div>
    </header>
  );
}

export default Header;
