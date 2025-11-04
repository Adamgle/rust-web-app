//! Mapping of the API schema types.
//! Most of those are relying on the database schemas.
//! Each type is stripped from sensitive information like passwords, etc, each is
//! designed to be used on the client-side.

export interface SessionUser {
  id: string;
  balance: string;
  email: string;
}

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
