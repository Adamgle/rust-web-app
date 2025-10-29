// NOTE: We cannot use 'use client' as we use metadata here.
// "use client";

import type { Metadata } from "next";
import "./globals.css";

//  Geist,
import { Geist_Mono } from "next/font/google";

// const geistSans = Geist({
//   variable: "--font-geist-sans",
//   subsets: ["latin"],
// });

const geistMono = Geist_Mono({
  variable: "--font-geist-mono",
  subsets: ["latin"],
});

export const metadata: Metadata = {
  title: "Stocked",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body
        className={`bg-background max-full geist-sans antialiased ${geistMono.className}`}
      >
        {children}
      </body>
    </html>
  );
}
