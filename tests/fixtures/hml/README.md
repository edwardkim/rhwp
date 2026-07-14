# HML equation fixture provenance

- Fixture: `exambank_math_equations_min.hml`
- Source: authorized ExamBank `tests/fixtures/serial_curated_min.hml`, which was already a minimized synthetic regression fixture
- Source SHA-256: `66998b57e70d38175e68facc3bf2fb2b7e6e0839c41c012acb47209d3071c538`
- Derived fixture SHA-256: `b51be49cde780d39b92f42cfd1cbd58474900c46c12c4df971f79bd511c7045a`
- Transformation: copied the 4,087 source bytes unchanged and appended one LF byte, producing this 4,088-byte repo fixture. No further minimization or anonymization transformation was performed.
- Scope: synthetic equation content retained for HML equation import/export regression tests. The contract test fixes the four ordered SCRIPT values and rejects source paths, URLs, email markers, and source repository identifiers.
- Authorization: the source-repository owner requested inclusion in the rhwp PR regression suite.
