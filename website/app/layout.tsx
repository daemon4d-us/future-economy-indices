import type { Metadata } from 'next'
import './globals.css'

export const metadata: Metadata = {
  title: 'Future Economy Indices',
  description: 'Track the future economy with SPACEINFRA and AIINFRA indices',
}

export default function RootLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return (
    <html lang="en">
      <body className="min-h-screen bg-gray-50 dark:bg-gray-900">
        <nav className="bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700">
          <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
            <div className="flex justify-between h-16">
              <div className="flex items-center">
                <a href="/" className="text-xl font-bold text-gray-900 dark:text-white">
                  Future Economy Indices
                </a>
              </div>
              <div className="flex items-center space-x-4">
                <a
                  href="/spaceinfra"
                  className="text-gray-700 dark:text-gray-300 hover:text-gray-900 dark:hover:text-white px-3 py-2 rounded-md text-sm font-medium"
                >
                  SPACEINFRA
                </a>
                <a
                  href="/aiinfra"
                  className="text-gray-700 dark:text-gray-300 hover:text-gray-900 dark:hover:text-white px-3 py-2 rounded-md text-sm font-medium"
                >
                  AIINFRA
                </a>
                <a
                  href="/pricing"
                  className="bg-blue-600 text-white px-4 py-2 rounded-md text-sm font-medium hover:bg-blue-700"
                >
                  Subscribe
                </a>
              </div>
            </div>
          </div>
        </nav>
        <main>{children}</main>
      </body>
    </html>
  )
}
