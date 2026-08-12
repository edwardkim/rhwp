# Task M100 / #3820 Stage 174 - saved RowBreak first-fragment physical slack

## 결론

`76076_regulatory_analysis.hwp`의 p4 -> p5 경계에서 native HWP5
`TopAndBottom + RowBreak` 표(`para=15`, `control=0`)의 첫 fragment가 header만
남고 body가 통째로 다음 쪽으로 밀리던 결함을 수정했다. 이 한 조기 이월이 이후
문서 전체의 physical owner를 한 쪽씩 밀어 p33--p36 비교를 무의미하게 만들고,
SVG/render tree 83쪽을 기준 PDF 82쪽과 다르게 만들었다.

수정 후 기준 PDF, rhwp SVG, rhwp render tree는 모두 82쪽이며, p4 -> p5와
p33 -> p36 범위의 text owner/page-boundary 후보는 0건이다.

## 원인

저장된 `common.height` 929px은 이 2행 표 전체의 논리 높이가 아니라 p4에서
paint하는 **첫 fragment 물리 프레임**이었다. `scan_block_table_split_rows`가
header 뒤 row 1의 내부 cut을 고를 때 measured painted height가 row-area 예산보다
약 3.5px 컸다. 기존 0.1px 허용치는 이 cut을 거절해 p4에는 header만 남겼다.

소스 프레임의 실제 하단은 body bottom보다 약 12px 위에 있었다. 즉 측정값은
사용 가능한 저장 물리 프레임 안에 있었지만, 스캐너가 선언 프레임 하단의 여유를
알지 못했다.

## 구현

[`src/renderer/typeset.rs`](../../src/renderer/typeset.rs)에
`source_first_fragment_overflow_allowance`를 추가했다.

- native HWP5, 비-TAC, `TopAndBottom`, `RowBreak`, 2행 이상, 각주 없음,
  빈 host, 첫 fragment에만 적용한다.
- synthetic이 아닌 첫 `LineSeg`의 **unshifted** 저장 anchor가 현재 flow cursor와
  0.5px 이내로 일치해야 한다. p168의 `vertical_offset` paint inset과 flow anchor를
  혼동하지 않기 위한 조건이다.
- source frame의 physical bottom이 body bottom 안에 있을 때만, 그 bottom 아래의
  남은 실제 px을 row cut 후보의 허용치로 사용한다.
- terminal-tail, continuation, nested-table 정책에는 영향을 주지 않는다. 각주
  reservation으로 재스캔할 때도 같은 fragment-scoped allowance를 전달한다.

따라서 이 변경은 선언 높이로 table 전체를 fit시키지 않고, 소스가 명시한 첫
물리 fragment 안에서만 row 1의 컷을 허용한다.

## 회귀 검사

[`tests/issue_3820_rowbreak_rowspan_band.rs`](../../tests/issue_3820_rowbreak_rowspan_band.rs)에
p4 소유자 검사를 추가했다. p4의 `para=15/control=0` 표는 header뿐 아니라
`제32조(보호구의 지급 등)` 및 `이륜자동차`가 든 첫 body fragment를 가져야 한다.
문자열은 renderer의 `TextRun` 경계와 무관하게 표 전체 텍스트로 확인한다.

다음 검사는 release-test target에서 통과했다.

- `issue_3820_rowbreak_rowspan_band`: 4 passed
- `issue_3820_body_top_table_border_clip`: 2 passed
- `issue_4490_4491_anchor_flow`: 2 passed
- `issue_4090_hwpx_tail_page_break`: 1 passed

## 시각/owner 증거

명령:

```sh
RHWP_BIN=target/task-3820-stage168/release-test/rhwp \
  venv/bin/python tools/fidelity_compare/fidelity_compare.py 0 81 \
  --source samples/76076_regulatory_analysis.hwp \
  --reference-pdf samples/issue1891/76076_regulatory_analysis-2024.pdf \
  --label 76076-stage174-full-owner \
  --reference-grade '한컴 2024 기준 PDF' \
  --text-only --export-all-svg --layout-ledger \
  --out-dir output/task-3820-stage174-76076-full-owner
```

- `page-count-ledger.tsv`: reference PDF 82, rhwp SVG 82, rhwp render tree 82.
- p4 -> p5의 과거 `table_fragment_text_owner_drift (rhwp_later_than_reference)`는
  사라졌다.
- p33 -> p36의 `page-boundary-fidelity-candidates.tsv`,
  `text-owner-shift-candidates.tsv`, `text-owner-sequence-candidates.tsv`는 모두
  header만 남았다.

Raster 확인은 다음 output에 남겼다.

- `output/task-3820-stage174-76076-p4-p5`: p4 14.82%, p5 9.08%.
- `output/task-3820-stage174-76076-p33-p36`: p33 19.93%, p34 14.75%,
  p35 18.60%, p36 14.93%.

이 raster 잔차는 table owner/page boundary drift가 아니다. 특히 p33/p35에서
reference의 제목 글자가 rhwp에 tofu(`□`)로 보여, 로컬 환경에 reference
`HCRDotum` 계열 글꼴이 없다는 Stage 169 판정과 일치한다. 해당 글꼴 자산을
무단으로 대체하거나 renderer pagination 규칙으로 보정할 근거는 없다.

## #3820 후보 상태

- 76076의 p4 global page shift 및 그 downstream p33--p36 owner shift: 해결.
- #4491 p26: Stage 169에서 HCRDotum 부재에 따른 glyph raster 차이로 분리.
- HWPX `156492236_규제샌드박스_min.hwpx` tail: 17쪽 및 p5->6, p7->8,
  p15->16 owner 회귀가 통과했다. 남은 본문 tofu는 같은 font-environment 범주다.

따라서 인용된 #3820 comment의 재현 가능한 paginator/fragment owner 원인은 모두
회귀 검사와 full-document ledger로 닫혔다. 남은 pixel 비교는 배포 가능한 기준
글꼴 환경을 공급한 뒤 별도 font-fidelity 작업으로 재평가해야 한다.
