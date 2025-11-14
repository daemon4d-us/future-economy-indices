"""
Custom weighting algorithm for SPACEINFRA index.

Three-factor weighting model:
- 40% Space Revenue Percentage
- 30% Market Capitalization (log-transformed)
- 30% Revenue Growth Rate
"""

import pandas as pd
import numpy as np
from typing import List, Dict, Optional
from dataclasses import dataclass


@dataclass
class IndexConstituent:
    """Represents a company in the index with its metrics and weight."""
    ticker: str
    name: str
    market_cap: float
    space_revenue_pct: float
    revenue_growth_rate: float  # YoY or 3Y CAGR
    raw_score: float
    weight: float
    segments: str


class SpaceIndexWeighting:
    """Calculate custom weights for space infrastructure index."""

    def __init__(
        self,
        space_revenue_weight: float = 0.4,
        market_cap_weight: float = 0.3,
        growth_weight: float = 0.3,
        max_position_size: float = 0.15,  # 15%
        min_position_size: float = 0.01,  # 1%
    ):
        """
        Initialize weighting parameters.

        Args:
            space_revenue_weight: Weight for space revenue % factor (default 0.4)
            market_cap_weight: Weight for market cap factor (default 0.3)
            growth_weight: Weight for growth rate factor (default 0.3)
            max_position_size: Maximum weight for any single position (default 0.15)
            min_position_size: Minimum weight for any position (default 0.01)
        """
        # Validate weights sum to 1.0
        total = space_revenue_weight + market_cap_weight + growth_weight
        if not np.isclose(total, 1.0):
            raise ValueError(f"Weights must sum to 1.0, got {total}")

        self.w_space_rev = space_revenue_weight
        self.w_market_cap = market_cap_weight
        self.w_growth = growth_weight
        self.max_weight = max_position_size
        self.min_weight = min_position_size

    def normalize_market_cap(self, market_caps: pd.Series) -> pd.Series:
        """
        Normalize market cap using log transformation.

        Args:
            market_caps: Series of market cap values

        Returns:
            Normalized market cap scores (0-100)
        """
        # Log transform to dampen large-cap dominance
        log_caps = np.log10(market_caps.replace(0, np.nan))

        # Normalize to 0-100 scale
        min_val = log_caps.min()
        max_val = log_caps.max()

        if max_val == min_val:
            return pd.Series(50.0, index=market_caps.index)

        normalized = (log_caps - min_val) / (max_val - min_val) * 100
        return normalized.fillna(0)

    def normalize_growth(self, growth_rates: pd.Series) -> pd.Series:
        """
        Normalize growth rates with clipping to handle extremes.

        Args:
            growth_rates: Series of growth rate values (as percentages, e.g., 45 for 45%)

        Returns:
            Normalized growth scores (0-100)
        """
        # Clip extreme values (-50% to +200%)
        clipped = growth_rates.clip(lower=-50, upper=200)

        # Normalize to 0-100 scale
        min_val = clipped.min()
        max_val = clipped.max()

        if max_val == min_val:
            return pd.Series(50.0, index=growth_rates.index)

        normalized = (clipped - min_val) / (max_val - min_val) * 100
        return normalized

    def calculate_weights(self, companies: pd.DataFrame) -> List[IndexConstituent]:
        """
        Calculate index weights for all companies.

        Args:
            companies: DataFrame with columns:
                - ticker: Stock ticker
                - name: Company name
                - market_cap: Market capitalization
                - space_revenue_pct: Space revenue % (0-100)
                - revenue_growth_rate: Revenue growth rate % (e.g., 45 for 45%)
                - segments: Space segments (optional)

        Returns:
            List of IndexConstituent objects with calculated weights
        """
        if len(companies) == 0:
            return []

        df = companies.copy()

        # Normalize each factor
        norm_space_rev = df['space_revenue_pct']  # Already 0-100
        norm_market_cap = self.normalize_market_cap(df['market_cap'])
        norm_growth = self.normalize_growth(df['revenue_growth_rate'])

        # Calculate raw scores
        raw_scores = (
            norm_space_rev * self.w_space_rev +
            norm_market_cap * self.w_market_cap +
            norm_growth * self.w_growth
        )

        # Convert to weights (normalize to sum to 1.0)
        weights = raw_scores / raw_scores.sum()

        # Apply position size constraints
        weights = weights.clip(lower=self.min_weight, upper=self.max_weight)

        # Renormalize after clipping
        weights = weights / weights.sum()

        # Create IndexConstituent objects
        constituents = []
        for idx, row in df.iterrows():
            constituent = IndexConstituent(
                ticker=row['ticker'],
                name=row.get('name', ''),
                market_cap=row['market_cap'],
                space_revenue_pct=row['space_revenue_pct'],
                revenue_growth_rate=row['revenue_growth_rate'],
                raw_score=raw_scores.loc[idx],
                weight=weights.loc[idx],
                segments=row.get('segments', '')
            )
            constituents.append(constituent)

        # Sort by weight descending
        constituents.sort(key=lambda x: x.weight, reverse=True)

        return constituents

    def summary_stats(self, constituents: List[IndexConstituent]) -> Dict:
        """Generate summary statistics for the index."""
        if not constituents:
            return {}

        total_market_cap = sum(c.market_cap * c.weight for c in constituents)
        avg_space_rev = sum(c.space_revenue_pct * c.weight for c in constituents)
        avg_growth = sum(c.revenue_growth_rate * c.weight for c in constituents)

        return {
            'num_constituents': len(constituents),
            'total_weight': sum(c.weight for c in constituents),
            'weighted_avg_market_cap': total_market_cap,
            'weighted_avg_space_rev_pct': avg_space_rev,
            'weighted_avg_growth': avg_growth,
            'max_weight': max(c.weight for c in constituents),
            'min_weight': min(c.weight for c in constituents),
        }


def main():
    """Example usage of weighting algorithm."""
    print("SPACEINFRA Index Weighting Algorithm")
    print("="*80 + "\n")

    # Sample data (from our batch test)
    sample_companies = pd.DataFrame([
        {
            'ticker': 'ASTS',
            'name': 'AST SpaceMobile',
            'market_cap': 19.2e9,
            'space_revenue_pct': 90,
            'revenue_growth_rate': 120,  # Estimated
            'segments': 'Satellites, Ground'
        },
        {
            'ticker': 'RKLB',
            'name': 'Rocket Lab',
            'market_cap': 25.0e9,
            'space_revenue_pct': 80,
            'revenue_growth_rate': 50,
            'segments': 'Launch, Satellites'
        },
        {
            'ticker': 'SPCE',
            'name': 'Virgin Galactic',
            'market_cap': 0.2e9,
            'space_revenue_pct': 50,
            'revenue_growth_rate': -20,  # Negative growth
            'segments': 'Launch, Ground'
        },
        {
            'ticker': 'IRDM',
            'name': 'Iridium',
            'market_cap': 1.8e9,
            'space_revenue_pct': 50,
            'revenue_growth_rate': 5,
            'segments': 'Satellites, Ground'
        },
        {
            'ticker': 'GSAT',
            'name': 'Globalstar',
            'market_cap': 6.4e9,
            'space_revenue_pct': 30,
            'revenue_growth_rate': 15,
            'segments': 'Satellites, Ground'
        },
    ])

    # Initialize weighting algorithm
    weighting = SpaceIndexWeighting(
        space_revenue_weight=0.4,
        market_cap_weight=0.3,
        growth_weight=0.3,
        max_position_size=0.15,
        min_position_size=0.01
    )

    # Calculate weights
    constituents = weighting.calculate_weights(sample_companies)

    # Display results
    print("Index Composition:\n")
    print(f"{'Rank':<6} {'Ticker':<8} {'Name':<20} {'Weight':<10} {'Space%':<10} {'Growth%':<10}")
    print("-" * 80)

    for i, c in enumerate(constituents, 1):
        print(f"{i:<6} {c.ticker:<8} {c.name:<20} {c.weight*100:>6.2f}%   "
              f"{c.space_revenue_pct:>6.1f}%   {c.revenue_growth_rate:>7.1f}%")

    # Summary statistics
    print("\n" + "="*80)
    print("Index Statistics:\n")
    stats = weighting.summary_stats(constituents)

    print(f"Number of Constituents: {stats['num_constituents']}")
    print(f"Total Weight: {stats['total_weight']*100:.1f}%")
    print(f"Weighted Avg Space Revenue %: {stats['weighted_avg_space_rev_pct']:.1f}%")
    print(f"Weighted Avg Growth Rate: {stats['weighted_avg_growth']:.1f}%")
    print(f"Largest Position: {stats['max_weight']*100:.1f}%")
    print(f"Smallest Position: {stats['min_weight']*100:.1f}%")


if __name__ == "__main__":
    main()
