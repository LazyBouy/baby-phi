/** @type {import('next').NextConfig} */
const nextConfig = {
  reactStrictMode: true,
  output: "standalone",
  async rewrites() {
    const api = process.env.BABY_PHI_API_URL || "http://127.0.0.1:8080";
    return [{ source: "/api/v0/:path*", destination: `${api}/api/v0/:path*` }];
  },
};

export default nextConfig;
