export default function Home() {
  return (
    <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
      {/* Hero Section */}
      <div className="text-center mb-16">
        <h1 className="text-5xl font-bold text-gray-900 dark:text-white mb-4">
          Track the Future Economy
        </h1>
        <p className="text-xl text-gray-600 dark:text-gray-300 mb-8">
          Data-driven indices for Space Infrastructure and AI Infrastructure
        </p>
        <a
          href="/pricing"
          className="inline-block bg-blue-600 text-white px-8 py-3 rounded-lg text-lg font-medium hover:bg-blue-700"
        >
          Subscribe for $99/year
        </a>
      </div>

      {/* Index Cards */}
      <div className="grid md:grid-cols-2 gap-8 mb-16">
        {/* SPACEINFRA Card */}
        <a
          href="/spaceinfra"
          className="block p-8 bg-white dark:bg-gray-800 rounded-lg shadow-lg hover:shadow-xl transition-shadow"
        >
          <h2 className="text-3xl font-bold text-gray-900 dark:text-white mb-4">
            SPACEINFRA
          </h2>
          <p className="text-gray-600 dark:text-gray-300 mb-4">
            Space Infrastructure Index
          </p>
          <div className="space-y-2 text-sm">
            <div className="flex justify-between">
              <span className="text-gray-500">Companies:</span>
              <span className="font-semibold">20</span>
            </div>
            <div className="flex justify-between">
              <span className="text-gray-500">1Y Return:</span>
              <span className="font-semibold text-green-600">+83%</span>
            </div>
            <div className="flex justify-between">
              <span className="text-gray-500">Sharpe Ratio:</span>
              <span className="font-semibold">1.53</span>
            </div>
          </div>
        </a>

        {/* AIINFRA Card */}
        <div className="p-8 bg-white dark:bg-gray-800 rounded-lg shadow-lg opacity-60">
          <h2 className="text-3xl font-bold text-gray-900 dark:text-white mb-4">
            AIINFRA
          </h2>
          <p className="text-gray-600 dark:text-gray-300 mb-4">
            AI Infrastructure Index
          </p>
          <div className="text-sm text-gray-500">
            Coming Q1 2025
          </div>
        </div>
      </div>

      {/* Features */}
      <div className="grid md:grid-cols-3 gap-8">
        <div className="text-center">
          <div className="text-4xl mb-4">📊</div>
          <h3 className="text-xl font-bold text-gray-900 dark:text-white mb-2">
            AI-Powered Selection
          </h3>
          <p className="text-gray-600 dark:text-gray-300">
            Companies classified using advanced AI models
          </p>
        </div>
        <div className="text-center">
          <div className="text-4xl mb-4">🎯</div>
          <h3 className="text-xl font-bold text-gray-900 dark:text-white mb-2">
            3-Factor Weighting
          </h3>
          <p className="text-gray-600 dark:text-gray-300">
            Revenue %, market cap, and growth rate optimization
          </p>
        </div>
        <div className="text-center">
          <div className="text-4xl mb-4">📈</div>
          <h3 className="text-xl font-bold text-gray-900 dark:text-white mb-2">
            Quarterly Rebalancing
          </h3>
          <p className="text-gray-600 dark:text-gray-300">
            Systematic updates every quarter
          </p>
        </div>
      </div>
    </div>
  )
}
