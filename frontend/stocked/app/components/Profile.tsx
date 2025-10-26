"use client";



export function Profile() {
  return (
    <div className="flex flex-col gap-2 h-fit text-sm">
      <div className="flex flex-row gap-4">
        <div className="border rounded-sm p-2">Name</div>
        <div className="border rounded-sm p-2">Image</div>
      </div>
      <button className="bg-blue-500 text-white font-bold p-1 rounded text-sm">
        Balance $0.0
      </button>
    </div>
  );
}

export function Logo() {
  return (
    <div className="flex flex-col items-center justify-center gap-2">
      <h1 className="bg-purple-500 text-white font-bold px-4 py-2 rounded saturate-100 self-start">
        Stocked
      </h1>
      <button className="bg-blue-500 text-white font-bold p-3 rounded text-sm">
        Make beaucoup-Bucks
      </button>
    </div>
  );
}
