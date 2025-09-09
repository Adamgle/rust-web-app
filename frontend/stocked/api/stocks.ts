// Everything related to stocks API calls

import { Stock } from "../types/types";

export async function getStocks(): Promise<Stock[]> {
    return fetch("/api/v1/stocks").then((res) => res.json());
}