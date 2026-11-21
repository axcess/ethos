#!/usr/bin/env python3
"""Build the bundled Chicago daily climate dataset from NOAA GHCN-Daily station files.

Dev-time only. The simulation never touches the network (DESIGN §4.1).

Usage:
    python3 -I tools/climate/build_chicago.py <dir-with-station-csv.gz> data/climate/chicago_daily.csv

Inputs (download from https://www.ncei.noaa.gov/pub/data/ghcn/daily/by_station/):
    USC00111577.csv.gz  Chicago Midway (primary, 1928-)
    USW00094846.csv.gz  Chicago O'Hare (gap fill + wind, 1946-)

Output columns (365 rows per year; Feb 29 dropped):
    year,doy,tmax_c,tmin_c,prcp_mm,snwd_mm,wind_ms,filled
`filled` is a bitmask: 1=temp from O'Hare, 2=temp from climatology, 4=prcp filled, 8=wind climatology.
"""
import csv, datetime, gzip, os, sys
from collections import defaultdict

PRIMARY, SECONDARY = "USC00111577", "USW00094846"
FIRST_YEAR, LAST_YEAR = 1929, 2025


def load(path):
    data = defaultdict(dict)  # date -> elem -> value
    with gzip.open(path, "rt") as f:
        for row in csv.reader(f):
            sid, date, elem, val, mflag, qflag = row[:6]
            if qflag.strip():  # failed a NOAA quality check
                continue
            if elem in ("TMAX", "TMIN", "PRCP", "SNWD", "AWND"):
                data[date][elem] = int(val)
    return data


def main(src, out):
    p = load(os.path.join(src, PRIMARY + ".csv.gz"))
    s = load(os.path.join(src, SECONDARY + ".csv.gz"))

    # Day-of-year climatology from the primary station (and secondary for wind).
    clim = defaultdict(lambda: defaultdict(list))
    for d, e in list(p.items()) + [(d, {"AWND": v["AWND"]}) for d, v in s.items() if "AWND" in v]:
        dt = datetime.date(int(d[:4]), int(d[4:6]), int(d[6:]))
        if dt.month == 2 and dt.day == 29:
            continue
        doy = (dt - datetime.date(dt.year, 1, 1)).days
        if dt.month > 2 and dt.year % 4 == 0 and (dt.year % 100 != 0 or dt.year % 400 == 0):
            doy -= 1
        for k, v in e.items():
            clim[doy][k].append(v)
    cmean = {doy: {k: sum(v) / len(v) for k, v in ks.items()} for doy, ks in clim.items()}

    rows, stats = [], defaultdict(int)
    for y in range(FIRST_YEAR, LAST_YEAR + 1):
        dt, doy = datetime.date(y, 1, 1), 0
        while dt.year == y:
            if not (dt.month == 2 and dt.day == 29):
                key, flag = dt.strftime("%Y%m%d"), 0
                pe, se = p.get(key, {}), s.get(key, {})
                if "TMAX" in pe and "TMIN" in pe:
                    tx, tn = pe["TMAX"], pe["TMIN"]
                elif "TMAX" in se and "TMIN" in se:
                    tx, tn, flag = se["TMAX"], se["TMIN"], flag | 1
                else:
                    tx, tn, flag = cmean[doy]["TMAX"], cmean[doy]["TMIN"], flag | 2
                if "PRCP" in pe:
                    pr = pe["PRCP"]
                elif "PRCP" in se:
                    pr, flag = se["PRCP"], flag | 4
                else:
                    pr, flag = 0, flag | 4
                sn = pe.get("SNWD", se.get("SNWD", 0))
                if "AWND" in se:
                    wd = se["AWND"]
                else:
                    wd, flag = cmean[doy].get("AWND", 45), flag | 8
                stats[flag] += 1
                rows.append((y, doy, tx / 10, tn / 10, pr / 10, sn, wd / 10, flag))
                doy += 1
            dt += datetime.timedelta(days=1)

    with open(out, "w", newline="") as f:
        w = csv.writer(f, lineterminator="\n")
        w.writerow(["year", "doy", "tmax_c", "tmin_c", "prcp_mm", "snwd_mm", "wind_ms", "filled"])
        for r in rows:
            w.writerow([r[0], r[1], f"{r[2]:.1f}", f"{r[3]:.1f}", f"{r[4]:.1f}", r[5], f"{r[6]:.1f}", r[7]])
    print(f"wrote {len(rows)} rows ({FIRST_YEAR}-{LAST_YEAR}); fill flags: {dict(stats)}")


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2])
