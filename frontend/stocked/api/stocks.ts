'use client'

// // Everything related to stocks API calls

// import { Stock } from "../types/types";

// // No idea how error handling will look like, the idea is that
// // we will iterate to provide some functionality and then
// // improve it.
// // The important idea is that we need to figure out the framework
// // of doing thing to easily layer down the error handling on top of we have.


// export async function getStocks(): Promise<Stock[]> {
//     return fetch("/stocks").then((res) => res.json()).catch((error) => {
//         console.error("Error fetching stocks:", error);

//         return undefined;
//     });
// }

// export async function getStock(id: number): Promise<Stock | undefined> {
//     return fetch(`stocks/${id}`).then((res) => res.json()).catch((error) => {
//         console.error("Error fetching stock:", error);

//         return undefined;
//     });
// }