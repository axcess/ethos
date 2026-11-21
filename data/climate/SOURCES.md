# Climate data sources

`chicago_daily.csv` is built by `tools/climate/build_chicago.py` from NOAA GHCN-Daily
(Global Historical Climatology Network – Daily), a U.S. government work in the public domain.

- Primary: USC00111577, CHICAGO MIDWAY AP 3SW (1928–)
- Gap fill and wind: USW00094846, CHICAGO O'HARE INTL AP (1946–)
- Source: https://www.ncei.noaa.gov/pub/data/ghcn/daily/by_station/
- Built 2026-11-21; years 1929–2025, 365 rows per year (Feb 29 dropped)
- sha256 1195a2842fa738bbb1095aa9a7e840eb65d0cd8ef110ee18d6294b9fbd397cf0

The `filled` column flags days where values were filled from O'Hare (1), climatology (2),
precipitation fill (4) or wind climatology (8; O'Hare wind is only recorded from 1984).
