# Open Questions

Revisited at every phase boundary. Each has my recommendation. Resolved items move to `DECISIONS.md`.

| ID | Question | Raise at | My recommendation |
|---|---|---|---|
| ~~OQ-1~~ (resolved D-017) | **Midwest weather:** confirm "historical Chicago record drives the tank, live NWS weather is on-screen only" (D-017)? Or do you want live weather to affect the simulation? | **Now** | Historical driver + live flavor. Live-as-input breaks deterministic replay and zero-player. If you still want it, I'd log live readings as simulation inputs so replay works. |
| **OQ-2** | Which Midwest location(s)? Chicago only, or a mix (Des Moines, Minneapolis, St. Louis…) per tank? | Phase 1 | Chicago default; a per-tank station is a cheap config option later. |
| ~~OQ-3~~ (accepted) | Lookahead delay (D-012): is a 1–3 min gap between simulation and broadcast acceptable? | Phase 0 review | Yes. Viewers can't tell, and the camera gets much better. |
| OQ-4 | Hardware: macOS version on the Mac mini, and will you run benchmark/soak scripts and send results? | Phase 1 | Required to validate the memory budget. |
| OQ-5 | Default tank end: should a thriving tank ever be retired (`max_real_days`) so the Hall of Fallen Tanks fills? | Phase 2 | Off by default; revisit after seeing real lifespans. |
| OQ-6 | Observation-pod glimpses: the pod position comes from the Camera Director (downstream of the simulation). Feeding it back breaks determinism. | Phase 7 | Let the simulation's own observer place "pod visits" from in-simulation interest (deterministic), and have the Director follow the pod rather than lead it. |
| OQ-7 | Remote health access: Tailscale, a simple port-forward, or local-only? | Phase 5 | Tailscale (no open ports). |
| OQ-8 | News team names and personalities | Phase 6 | I'll draft three. |
| OQ-9 | Visual mood and palette for the aquarium | Phase 4 | I'll propose 2–3 mood boards as rendered stills. |
| OQ-10 | Content policy: how graphic can death, war and predation be? | Phase 4 | Stylized, no gore; death shown by stillness, drifting and fading color. |
| OQ-11 | Stream resolution/bitrate (1080p60 vs 1080p30) for Twitch + YouTube from one Mac mini | Phase 5 | 1080p30 for headroom; 60 if soak tests allow. |
| OQ-12 | Do we want an "inhabitant chatter" audio bed always on, or only in spotlight shots? | Phase 5 | Low ambient murmur always, louder in spotlight. |
| OQ-13 | Language split timescale: dialects by cycle 60, no separate languages yet. Speed up drift or accept? | Phase 1 review | Run 200-cycle tests first |
| OQ-14 | Early founder crash (median 126 → 52 by cycle 6): soften or keep? | Phase 1 review | Soften modestly |
| OQ-15 | Algae gardens arrive early and enable booms to ~800: add prerequisites now? | Phase 1 review | Yes, cheap |
| OQ-16 | Leaders have little effect: give leadership real coordination powers? | Phase 1 review | Yes, before Phase 2 |
| ~~OQ-17~~ (resolved D-043) | Boom-bust: nearly all tanks overshoot and crash at ~200 people; none escape; 29% extinct by cycle 60. How should escape happen? | Now | Learned restraint: minds learn "worn beds → less food" and "crowding → hunger" from experience, then spread out, rest beds or move sooner. Escape becomes a cultural achievement some tanks reach and others don't. |
