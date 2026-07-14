# 2026-07-12

## HML signature detection CodeQL fix

- Root cause: the leading XML-comment expression combined a lazy comment-body repetition with an
  outer repeated-comment group. Repeated `--><!--` boundaries therefore admitted exponentially many
  backtracking paths; a 172-byte prefix already took hundreds of milliseconds.
- Decision: scan optional XML declaration and leading comments with monotonically advancing offsets
  and `indexOf`, then apply the existing root capture as a sticky expression at the resulting offset.
  This keeps XML/HML/HTML classification and the root-end offset contract while making preamble work
  linear in the bounded prefix size.
- Regression proof: the original implementation failed the malicious-prefix timing test; the scanner
  classifies the same input within the 100 ms budget and preserves XML declaration plus consecutive
  comment handling.
- Alternatives rejected: changing regex flags or lazy modifiers leaves the ambiguous nested
  repetition, and relying only on the 64 KiB prefix bound still permits expensive attacks from very
  small inputs.

## PR #2219 HML comparison-material clarification

- Authority correction: PDFs exported by Hancom Viewer on macOS are auxiliary comparison material,
  not answer keys; this project's answer-key tier requires output from the Hancom 2020/2022 editor.
- Width finding: the HML declares a `41956 HU` table between `283 HU` outer margins; together they
  exactly fill the `42522 HU` text area. Native output and the Viewer comparison both measure about
  `420 pt` for the table, so the reported half-width difference was not reproducible.
- Root cause: the HML adapter preserved `TreatAsChar=true` only in `common.treat_as_char`, while the
  legacy typesetter also reads `Table.attr` bit 0. Separately, the wide-table classifier treated the
  table as a block and lost its middle text anchor between `abc` and `efg`.
- Decision: mirror the canonical inline bit during HML adaptation, keep a wide treat-as-character
  table inline when it has a true middle anchor, and wrap that table onto its own visual line when it
  cannot fit after the leading text. HML save preflight accepts that mirrored compatibility bit while
  continuing to reject unknown or contradictory table bits. This preserves `abc` above and `efg`
  below the table without weakening the loss guard.
- Alternatives rejected: changing the source width would contradict the HML and Viewer evidence;
  treating every wide table as inline would change start- and end-anchored table behavior globally.
