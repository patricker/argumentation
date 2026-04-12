# Real ICCMA 2019 benchmark fixtures

**Source instances:** http://argumentationcompetition.org/2019/iccma-instances.tar.gz
**Source expected outputs:** http://argumentationcompetition.org/2019/reference-results.tar.gz
**Retrieved:** 2026-04-12
**License:** The ICCMA 2019 competition page does not state an explicit license
for the bundled benchmark archives. They are published by the organizers for
benchmarking solvers. These files are distributed here for testing purposes
only; if your downstream use requires a specific license, verify with the
upstream source.

## Selected instances

Five small instances from the "Small-result-*" family of the ICCMA 2019
instances archive were chosen for diversity of structure. All contain ≤ 15
arguments.

| File | Args | Attacks | Notes |
|------|------|---------|-------|
| `Small-result-b2.apx`  | 5 | 8  | two mutual attacks (a0↔a1, a3↔a4) plus isolated unattacked `a2_0`; exercises preferred/stable split around an uncontested defender |
| `Small-result-b41.apx` | 6 | 9  | three unattacked source arguments (a0_0, a1_0, a3_0) plus a rebut cycle on a2_2/a4_2/a5_2 — multiple stable extensions, empty grounded |
| `Small-result-b8.apx`  | 8 | 11 | one "central" argument (a1_0) attacking many peripheral args that each partially attack back — nested defense |
| `Small-result-b57.apx` | 8 | 12 | a1_0 in mutual attack with many defenders; similar to b8 but structurally denser |
| `Small-result-b35.apx` | 7 | 22 | dense framework combining mutual attacks, odd cycles, and nested defenses — 6 complete extensions, semi-stable ⊊ complete |

## Expected outputs

Expected extensions were taken from the ICCMA 2019 `reference-results.tar.gz`
archive (`iccma-2019-<stem>.apx-{SE-GR,SE-ID,EE-CO,EE-PR,EE-ST,EE-SST}.out`
files) and transcribed into this directory's `.txt` files under
`expected/`. The ICCMA native format is `[[a,b],[c,d]]` per file plus a
`N solution(s) found` footer; we convert to one-argument-per-line with
`---` separators between extensions for easier diffing.

Mapping:

| ICCMA track | File suffix used here |
|-------------|-----------------------|
| `SE-GR` (grounded, single extension) | `.grounded.txt` |
| `SE-ID` (ideal, single extension) | `.ideal.txt` |
| `EE-CO` (all complete extensions) | `.complete.txt` |
| `EE-PR` (all preferred extensions) | `.preferred.txt` |
| `EE-ST` (all stable extensions) | `.stable.txt` |
| `EE-SST` (all semi-stable extensions) | `.semi-stable.txt` |

An empty extension (e.g. grounded = ∅) is encoded as a file containing only
comment lines; `tests/ground_truth_iccma.rs`'s `parse_expected` always emits a
trailing extension, so this parses to a singleton set containing the empty
set — matching the SE-GR convention.
