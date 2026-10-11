# Actual asynchronous pipeline review

The audio/gameplay specialist independently read the proposed workflow and team contract while the producer was integrating. It resolved the three pinned Actions references in their official repositories, checked Node/pnpm/Rust versions and commands against this checkout, and found no source blocker in the read-only CI permissions, native matrix or dual-WASM package lane. Hosted CI had not been executed by that reviewer.

Two concrete wording corrections were requested and applied: Mission Control's service is localhost-bound, but its tracked ledger is part of the user's public GitHub repository; the WASM/save contract uses actual Rust serialization through SaveStore with **headless fake-indexeddb**, not a real browser storage measurement.

The producer created https://github.com/itchson/GalaxyRoyale/pull/1 and attached it to this chat. No approval from another GitHub account or live meeting is claimed.

Hosted run 38106105476 passed Linux source and dual-WASM packaging but failed five Windows artifact tests. Windows temporary roots used a short-name alias while realpath returned a long name. The producer canonicalized both sides of the containment check and added a real directory-alias regression; all 14 local store tests passed. The revised head 89452bcb42f86d4d1068d79d133a5bb7ef7ac557 passed all three jobs in run 38106597180. PR1 then merged with expected-head enforcement, producing 871cd1e6bc06ef79cfd31c9cf1964f1dde63bdea.

The later independent preview review found an actual overlooked relative-path Vite server at PID16080/port4190. Its absolute project-owned baseline output path established ownership before it was stopped. The launcher now recognizes that form and blocks unverified relative game processes visibly instead of killing them. The producer also added a before/after build identity check and record-write failure cleanup. These are actual asynchronous review exchanges; final integrated checks remain separate.
