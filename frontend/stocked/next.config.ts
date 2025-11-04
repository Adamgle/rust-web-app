import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  /* config options here */

  async rewrites() {
    return [
      // {
      //   source: '/api/v1/products',
      //   destination: `http://127.0.0.1:5000/api/v1/products`,
      // },
      {
        source: "/api/v1/:path*",
        destination: `${process.env.SERVER_URL}/api/v1/:path*`, // Proxy to Backend
        // has: [
        //   {
        //     type: 'header',
        //     key: 'x-api-key',
        //     value: process.env.API_KEY
        //   }
        // ]
      },
    ];
  },
};

export default nextConfig;
