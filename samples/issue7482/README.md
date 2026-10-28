# PR #7482 paragraph/table row regression inputs

These public inputs exercise reflow without saved LineSegs. The HWPX variants
are manually constructed diagnostics, not evidence for accepting saved layout
metadata. `provenance.json` pins every input and its independently generated
Hancom 2020 PDF under `pdf/issue7482/` (one page each).

## Input generation and independent expectations

- `tac-{one-row,two-rows,explicit-break}.hwpx`: start with the first paragraph of
  `samples/issue6601/36331407_side_by_side_tac_tables.hwpx`, retain resources,
  remove its saved LineSegs and later paragraphs, and add the ordinary follower
  `TAC 대조 문단`. A 700px body fits both tables and their separator; a 500px
  body wraps the second table. The explicit-break variant uses an authored
  `hp:lineBreak` inside `hp:t` and wraps even at 700px.
- `tac-mixed-{one-row,explicit-break}.hwpx`: insert `B ` before the first TAC,
  ` A ` between them and ` C` after the second. Increase body width by
  15000 HWPUNIT to 900px. The second variant has a break after A. The PDFs
  establish both table row ownership and text order, including C outside
  the second table.
- `tac-word-after-pair.hwpx`: append ` validation` to the 700px fitting pair.
  Hancom keeps both TACs on the first row and the complete word on the next.
- `original-no-ls-overlay-tac.hwp`: the unchanged public #7480 reproduction
  from the contributor's sample repository. Its PDF places the overlay on the
  spacer row after the TAC. Equal-style empty, space-only and InFront picture
  host paragraphs each reserve their own line.
- `margin{284,1034}.hwpx`: export that public HWP as HWPX; change only the root
  TAC bottom outer margin from 284 to 1034 HWPUNIT in the latter. Independent
  PDFs show a 10px follower advance at 96dpi with the table bbox unchanged.
- `long-boundary-8.hwpx`: derive from the #7481 title/Square host input, repeat
  its host text `문단 배치 검증 ` eight times and replace the following large
  table with `후속 문단 경계`. Hancom produces three visible host rows, restores
  full width below the Square table, and places the follower after all text.

The test also reuses the existing
`samples/issue7481/synth_square_host_full_width_table_no_ls.hwp`: independent
Hancom output and a separately saved HWPX establish that empty decoration
hosts still occupy a line and Square outer margins separate the host and
following TAC. It is not duplicated here.

## Validation scope

At production code `af6daab01fb8739a8af9a2d377b91fdf2b72936e`, all twelve
Native and fresh WASM comparisons (these inputs plus the existing title and
full-width controls) had a minimum 2px-neighbour silhouette agreement of
99.35552%. All review PNGs and representative standalone overlays were read
directly for row membership, order, table boundaries and follower placement.
The output-relationship diagnostic passed 12/12; exact contributor code
`9bc8478d5ddf43dc87c06278b8c0deb8c7640959` passed the fitting control and failed
the eleven defect checks for their placement causes.

This evidence permits adding the regression source; it does not substitute
for the final head's required lint, corpus and complete regression checks.
The local verification report links SHA-pinned execution evidence under
`output/pr-review/davindev-20261003/candidate-final6-*`. Private fonts and
embedded SVGs are excluded from these public assets. Residual Paper-anchored
header differences are separate from the tested body-flow relationships.
The 50% page acceptance condition is unresolved and not asserted here.

## 후속 일반 문단의 Square 배제 영역 대조군

`tac-after-plain-paragraphs.hwpx`는 공개 #7481 제목/Square 입력의 NO_LS
HWPX에 일반 문단 `공간 확인` 22개를 넣고 뒤의 큰 TAC를 유지한 수동 진단이다.
용지 크기는 그대로이며 표의 쪽 경계 설정은 NONE이다. 독립 한컴 2020 PDF는
`pdf/issue7482/tac-after-plain-paragraphs-hwpx-2020.pdf`이고 2쪽이다.
첫 후속 문단은 Square의 왼쪽 가용 공간에서 시작하며 나머지 문단의 순서를
보존한다. 큰 TAC는 다음 쪽에 통째로 있다. 줄 흐름 끝과 표의 점유 끝을
같은 값으로 처리하는 가정을 검증한다. 저장 LineSeg 수용이나 50% 정책의
정당성을 입증하는 입력으로 쓰지 않는다. 최신 보정과 시각 검증은 진행 중이다.
