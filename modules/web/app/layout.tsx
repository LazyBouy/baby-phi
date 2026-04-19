import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "baby-phi",
  description: "Agent management platform",
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en">
      <body className="min-h-screen">{children}</body>
    </html>
  );
}
