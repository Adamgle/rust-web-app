// NOTE: Currently that would the whole file, when the type would get extensive we would split those.

export interface Stock {
    id: number;
    abbreviation: string;
    company: string;
    since: Date; // Date in ISO format
    price: number;
    delta: number; // Percent change in price since the last data revalidation(ideally 1 minute)
    last_update: Date; // Date in ISO format
    created_at: Date; // Date in ISO format
}
