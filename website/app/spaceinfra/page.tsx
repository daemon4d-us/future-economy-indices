'use client'

import { useEffect, useState } from 'react'

interface IndexData {
  name: string
  display_name: string
  description: string
  num_constituents: number
  total_market_cap: number
  last_rebalance: string
}

interface Constituent {
  ticker: string
  company_name: string
  weight: number
  market_cap: number
  space_revenue_pct: number
  segments: string[]
}

export default function SpaceInfra() {
  const [indexData, setIndexData] = useState<IndexData | null>(null)
  const [composition, setComposition] = useState<Constituent[]>([])
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    fetch('/api/indices/SPACEINFRA')
      .then(res => res.json())
      .then(data => setIndexData(data))
      .catch(err => console.error('Error:', err))

    fetch('/api/indices/SPACEINFRA/composition')
      .then(res => res.json())
      .then(data => setComposition(data.constituents))
      .catch(err => console.error('Error:', err))
      .finally(() => setLoading(false))
  }, [])

  if (loading) {
    return <div className="max-w-7xl mx-auto px-4 py-12"><div className="text-center">Loading...</div></div>
  }

  const marketCapB = ((indexData?.total_market_cap || 0) / 1e9).toFixed(0)

  return (
    <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
      <div className="mb-8">
        <h1 className="text-4xl font-bold text-gray-900 dark:text-white mb-2">
          {indexData?.display_name}
        </h1>
        <p className="text-lg text-gray-600 dark:text-gray-300">
          {indexData?.description}
        </p>
      </div>

      <div className="grid md:grid-cols-4 gap-6 mb-8">
        <div className="bg-white dark:bg-gray-800 p-6 rounded-lg shadow">
          <div className="text-sm text-gray-500 dark:text-gray-400 mb-1">Companies</div>
          <div className="text-3xl font-bold text-gray-900 dark:text-white">{indexData?.num_constituents}</div>
        </div>
        <div className="bg-white dark:bg-gray-800 p-6 rounded-lg shadow">
          <div className="text-sm text-gray-500 dark:text-gray-400 mb-1">Market Cap</div>
          <div className="text-3xl font-bold text-gray-900 dark:text-white">${marketCapB}B</div>
        </div>
        <div className="bg-white dark:bg-gray-800 p-6 rounded-lg shadow">
          <div className="text-sm text-gray-500 dark:text-gray-400 mb-1">1Y Return</div>
          <div className="text-3xl font-bold text-green-600">+83%</div>
        </div>
        <div className="bg-white dark:bg-gray-800 p-6 rounded-lg shadow">
          <div className="text-sm text-gray-500 dark:text-gray-400 mb-1">Sharpe Ratio</div>
          <div className="text-3xl font-bold text-gray-900 dark:text-white">1.53</div>
        </div>
      </div>

      <div className="bg-white dark:bg-gray-800 rounded-lg shadow overflow-hidden mb-8">
        <div className="px-6 py-4 border-b border-gray-200 dark:border-gray-700">
          <h2 className="text-2xl font-bold text-gray-900 dark:text-white">Holdings</h2>
        </div>
        <div className="overflow-x-auto">
          <table className="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
            <thead className="bg-gray-50 dark:bg-gray-700">
              <tr>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase">Ticker</th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase">Company</th>
                <th className="px-6 py-3 text-right text-xs font-medium text-gray-500 dark:text-gray-300 uppercase">Weight</th>
                <th className="px-6 py-3 text-right text-xs font-medium text-gray-500 dark:text-gray-300 uppercase">Space %</th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase">Segments</th>
              </tr>
            </thead>
            <tbody className="bg-white dark:bg-gray-800 divide-y divide-gray-200 dark:divide-gray-700">
              {composition.map((h) => (
                <tr key={h.ticker} className="hover:bg-gray-50 dark:hover:bg-gray-700">
                  <td className="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900 dark:text-white">{h.ticker}</td>
                  <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900 dark:text-gray-300">{h.company_name}</td>
                  <td className="px-6 py-4 whitespace-nowrap text-sm text-right text-gray-900 dark:text-white">{(h.weight * 100).toFixed(2)}%</td>
                  <td className="px-6 py-4 whitespace-nowrap text-sm text-right text-gray-900 dark:text-white">{h.space_revenue_pct}%</td>
                  <td className="px-6 py-4 text-sm text-gray-900 dark:text-gray-300">{h.segments.join(', ')}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>

      <div className="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
        <h2 className="text-2xl font-bold text-gray-900 dark:text-white mb-4">Methodology</h2>
        <div className="prose dark:prose-invert max-w-none">
          <p className="text-gray-600 dark:text-gray-300 mb-4">
            The SPACEINFRA index uses a 3-factor weighting algorithm:
          </p>
          <ul className="list-disc pl-6 text-gray-600 dark:text-gray-300 space-y-2">
            <li><strong>40% Space Revenue Percentage:</strong> Companies with higher space-focused revenue receive higher weights</li>
            <li><strong>30% Market Capitalization:</strong> Log-transformed to dampen large-cap dominance</li>
            <li><strong>30% Revenue Growth Rate:</strong> Rewards high-growth companies</li>
          </ul>
          <p className="text-gray-600 dark:text-gray-300 mt-4">
            The index rebalances quarterly and includes companies from Launch, Satellites, Ground Systems, and Components segments.
          </p>
        </div>
      </div>
    </div>
  )
}
