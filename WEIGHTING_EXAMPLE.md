# SPACEINFRA Weighting Algorithm Example

## Sample Companies (Hypothetical)

| Company | Market Cap ($B) | Space Rev % | Revenue Growth (YoY) |
|---------|----------------|-------------|----------------------|
| PureSpace Inc | 5 | 100% | 45% |
| SatCom Corp | 25 | 80% | 25% |
| Mega Defense Co | 150 | 20% | 10% |
| Rocket Startup | 2 | 95% | 120% |
| Space Components | 8 | 60% | 35% |

## Step 1: Normalize Each Factor

### Market Cap Normalization (using log transform)
```
log_market_cap = log10(market_cap_billions)

PureSpace:          log10(5) = 0.70
SatCom:            log10(25) = 1.40
Mega Defense:     log10(150) = 2.18
Rocket Startup:     log10(2) = 0.30
Space Components:   log10(8) = 0.90

# Normalize to 0-100 scale
max_log = 2.18, min_log = 0.30

PureSpace:         (0.70 - 0.30) / (2.18 - 0.30) * 100 = 21.3
SatCom:            (1.40 - 0.30) / (2.18 - 0.30) * 100 = 58.5
Mega Defense:      (2.18 - 0.30) / (2.18 - 0.30) * 100 = 100.0
Rocket Startup:    (0.30 - 0.30) / (2.18 - 0.30) * 100 = 0.0
Space Components:  (0.90 - 0.30) / (2.18 - 0.30) * 100 = 31.9
```

### Space Revenue % (already 0-100)
```
PureSpace:         100
SatCom:            80
Mega Defense:      20
Rocket Startup:    95
Space Components:  60
```

### Growth Normalization (clip to -50% to +200%, then normalize)
```
# Clipped growth (120% stays, others as-is)
growth_clipped = max(-50, min(200, growth))

PureSpace:         45
SatCom:            25
Mega Defense:      10
Rocket Startup:    120
Space Components:  35

# Normalize to 0-100 (relative to this set)
max = 120, min = 10

PureSpace:         (45 - 10) / (120 - 10) * 100 = 31.8
SatCom:            (25 - 10) / (120 - 10) * 100 = 13.6
Mega Defense:      (10 - 10) / (120 - 10) * 100 = 0.0
Rocket Startup:    (120 - 10) / (120 - 10) * 100 = 100.0
Space Components:  (35 - 10) / (120 - 10) * 100 = 22.7
```

## Step 2: Calculate Raw Scores (w1=0.4, w2=0.3, w3=0.3)

```
raw_score = (space_rev_pct * 0.4) + (norm_market_cap * 0.3) + (norm_growth * 0.3)

PureSpace:         (100 * 0.4) + (21.3 * 0.3) + (31.8 * 0.3) = 55.9
SatCom:            (80 * 0.4) + (58.5 * 0.3) + (13.6 * 0.3) = 53.6
Mega Defense:      (20 * 0.4) + (100.0 * 0.3) + (0.0 * 0.3) = 38.0
Rocket Startup:    (95 * 0.4) + (0.0 * 0.3) + (100.0 * 0.3) = 68.0
Space Components:  (60 * 0.4) + (31.9 * 0.3) + (22.7 * 0.3) = 40.4

Total raw scores = 255.9
```

## Step 3: Calculate Weights (before constraints)

```
weight = raw_score / total_raw_scores

PureSpace:         55.9 / 255.9 = 21.8%
SatCom:            53.6 / 255.9 = 20.9%
Mega Defense:      38.0 / 255.9 = 14.8%
Rocket Startup:    68.0 / 255.9 = 26.6%
Space Components:  40.4 / 255.9 = 15.8%
```

## Step 4: Apply Constraints (max 15%, min 1%)

```
PureSpace:         21.8% → capped to 15%
SatCom:            20.9% → capped to 15%
Mega Defense:      14.8% → stays
Rocket Startup:    26.6% → capped to 15%
Space Components:  15.8% → capped to 15%

After capping, need to renormalize to sum to 100%
Total after caps = 74.8%

Final weights (renormalized):
PureSpace:         15% / 74.8% * 100% = 20.1%
SatCom:            15% / 74.8% * 100% = 20.1%
Mega Defense:      14.8% / 74.8% * 100% = 19.8%
Rocket Startup:    15% / 74.8% * 100% = 20.1%
Space Components:  15% / 74.8% * 100% = 20.1%
```

## Key Insights from Example

1. **Rocket Startup** gets highest raw score despite smallest market cap due to:
   - Very high space revenue % (95%)
   - Exceptional growth (120%)

2. **Mega Defense** ranks lower despite huge market cap because:
   - Only 20% space revenue (conglomerate issue)
   - Low growth (10%)

3. **Position size constraints** prevent over-concentration
   - Multiple companies hit 15% cap
   - Results in more balanced portfolio

## Refinements to Consider

1. **Should growth be capped lower** to prevent over-weighting early-stage high-growth small caps?
2. **Alternative market cap transform**: Try sqrt instead of log if we want more mid-cap exposure
3. **Pure-play bonus**: Add +10 to space_rev_pct if it's >90% to reward pure plays
4. **Minimum absolute revenue**: Screen out companies with < $100M revenue even if growth is high
