"""
Polygon.io API client for fetching market data.

Handles:
- Stock tickers and company metadata
- Financial fundamentals
- Historical price data
- Rate limiting and caching
"""

import os
import time
from typing import Optional, Dict, List, Any
from datetime import datetime, timedelta
import requests
from dotenv import load_dotenv
from tenacity import retry, stop_after_attempt, wait_exponential
import json

# Load environment variables
load_dotenv()


class PolygonClient:
    """Client for interacting with Polygon.io API."""

    BASE_URL = "https://api.polygon.io"

    def __init__(self, api_key: Optional[str] = None):
        """
        Initialize Polygon client.

        Args:
            api_key: Polygon.io API key. If not provided, reads from POLYGON_API_KEY env var.
        """
        self.api_key = api_key or os.getenv("POLYGON_API_KEY")
        if not self.api_key:
            raise ValueError("POLYGON_API_KEY must be set in environment or passed to constructor")

        self.session = requests.Session()
        self.rate_limit_delay = 0.2  # 200ms between requests (conservative for free tier)

    @retry(stop=stop_after_attempt(3), wait=wait_exponential(multiplier=1, min=2, max=10))
    def _make_request(self, endpoint: str, params: Optional[Dict] = None) -> Dict[str, Any]:
        """
        Make API request with retry logic.

        Args:
            endpoint: API endpoint (e.g., "/v3/reference/tickers")
            params: Query parameters

        Returns:
            JSON response as dictionary
        """
        if params is None:
            params = {}

        params["apiKey"] = self.api_key
        url = f"{self.BASE_URL}{endpoint}"

        time.sleep(self.rate_limit_delay)  # Rate limiting

        response = self.session.get(url, params=params)
        response.raise_for_status()

        return response.json()

    def get_ticker_details(self, ticker: str) -> Dict[str, Any]:
        """
        Get detailed information about a ticker.

        Args:
            ticker: Stock ticker symbol (e.g., "AAPL")

        Returns:
            Ticker details including name, market cap, description, etc.
        """
        endpoint = f"/v3/reference/tickers/{ticker}"
        return self._make_request(endpoint)

    def search_tickers(
        self,
        market: str = "stocks",
        exchange: Optional[str] = None,
        active: bool = True,
        limit: int = 1000
    ) -> List[Dict[str, Any]]:
        """
        Search for tickers matching criteria.

        Args:
            market: Market type (stocks, crypto, fx)
            exchange: Exchange code (XNAS for Nasdaq, XNYS for NYSE, etc.)
            active: Only return active tickers
            limit: Maximum results to return

        Returns:
            List of ticker dictionaries
        """
        endpoint = "/v3/reference/tickers"
        params = {
            "market": market,
            "active": str(active).lower(),
            "limit": limit
        }

        if exchange:
            params["exchange"] = exchange

        response = self._make_request(endpoint, params)
        return response.get("results", [])

    def get_aggregates(
        self,
        ticker: str,
        multiplier: int = 1,
        timespan: str = "day",
        from_date: str = None,
        to_date: str = None,
        limit: int = 5000
    ) -> List[Dict[str, Any]]:
        """
        Get aggregate bars (OHLCV) for a ticker.

        Args:
            ticker: Stock ticker symbol
            multiplier: Size of timespan multiplier (e.g., 1 for 1 day)
            timespan: Size of time window (minute, hour, day, week, month, quarter, year)
            from_date: Start date (YYYY-MM-DD)
            to_date: End date (YYYY-MM-DD)
            limit: Maximum results

        Returns:
            List of aggregate bars with OHLCV data
        """
        if not from_date:
            from_date = (datetime.now() - timedelta(days=365)).strftime("%Y-%m-%d")
        if not to_date:
            to_date = datetime.now().strftime("%Y-%m-%d")

        endpoint = f"/v2/aggs/ticker/{ticker}/range/{multiplier}/{timespan}/{from_date}/{to_date}"
        params = {"limit": limit, "adjusted": "true"}

        response = self._make_request(endpoint, params)
        return response.get("results", [])

    def get_snapshot(self, ticker: str) -> Dict[str, Any]:
        """
        Get current snapshot of a ticker (latest price, volume, etc.).

        Args:
            ticker: Stock ticker symbol

        Returns:
            Snapshot data
        """
        endpoint = f"/v2/snapshot/locale/us/markets/stocks/tickers/{ticker}"
        return self._make_request(endpoint)

    def get_financials(
        self,
        ticker: str,
        timeframe: str = "annual",
        limit: int = 10
    ) -> List[Dict[str, Any]]:
        """
        Get financial data (income statement, balance sheet, cash flow).

        Args:
            ticker: Stock ticker symbol
            timeframe: annual or quarterly
            limit: Number of periods to retrieve

        Returns:
            List of financial statements
        """
        endpoint = f"/vX/reference/financials"
        params = {
            "ticker": ticker,
            "timeframe": timeframe,
            "limit": limit
        }

        response = self._make_request(endpoint, params)
        return response.get("results", [])

    def get_market_cap(self, ticker: str) -> Optional[float]:
        """
        Get current market capitalization for a ticker.

        Args:
            ticker: Stock ticker symbol

        Returns:
            Market cap in USD, or None if not available
        """
        try:
            details = self.get_ticker_details(ticker)
            return details.get("results", {}).get("market_cap")
        except Exception as e:
            print(f"Error fetching market cap for {ticker}: {e}")
            return None


def main():
    """Example usage of PolygonClient."""
    client = PolygonClient()

    # Example: Get ticker details
    print("Testing Polygon.io connection...")
    ticker = "ASTS"  # AST SpaceMobile - space infrastructure company

    try:
        details = client.get_ticker_details(ticker)
        print(f"\n✓ Successfully connected to Polygon.io")
        print(f"\nTicker: {ticker}")
        print(f"Name: {details.get('results', {}).get('name', 'N/A')}")
        print(f"Market Cap: ${details.get('results', {}).get('market_cap', 0):,.0f}")
        print(f"Description: {details.get('results', {}).get('description', 'N/A')[:200]}...")

        # Get recent price data
        print(f"\nFetching historical data...")
        aggregates = client.get_aggregates(ticker, from_date="2024-01-01", to_date="2024-12-31")
        print(f"✓ Retrieved {len(aggregates)} daily bars")

        if aggregates:
            latest = aggregates[-1]
            print(f"Latest close: ${latest.get('c', 0):.2f}")
            print(f"Volume: {latest.get('v', 0):,}")

    except Exception as e:
        print(f"✗ Error: {e}")
        print("\nMake sure your POLYGON_API_KEY is set in .env file")


if __name__ == "__main__":
    main()
