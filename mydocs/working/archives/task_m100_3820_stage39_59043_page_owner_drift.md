---
kind: analysis
status: in_progress
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-07
---

# Task #3820 Stage 39 — #1921 PDF page-owner 지연의 최초 경계

## 이전 Stage에서 확정한 것

Stage 38은 p16의 outer-wrapper renderer overflow(67줄)를 제거했다. 그러나 한컴 2022 PDF와
직접 비교하면 current p16은 PDF p16보다 두 쪽 이른 source content를 보유한다. 숨은 cell line이
없어졌다는 사실은 PDF page-owner 일치의 증명이 아니다.

## 현재 관측

- p8은 PDF와 text multiset이 같고, 사진 두 개도 source cell 안에 있다. p8은 이번 page-owner
  조사에서 첫 이탈 지점이 아니다.
- current p15의 `나. 정부개입 필요성`은 PDF p13과 대응한다.
- current p18의 `③ 대안의 선택 및 근거`는 PDF p16과 대응한다.
- current p11--p14는 source `pi=98, ci=0`의 6×1 partial table만을 4개 page fragment로
  배치한다. PDF p10--p12에는 이 영역의 담배 사례 표가 3쪽으로 끝나고 PDF p13에서 다음
  `나. 정부개입 필요성`이 시작한다.

따라서 p98의 4쪽 분할은 확인된 한 쪽 지연 후보이며, 나머지 한 쪽의 최초 원인은 p9--p10 또는
p98에 도달하기 전의 source-owner/cut chain에서 찾아야 한다.

## 가설

이 Stage는 `row_geometry_table`이나 baseline을 전역 변경하지 않는다. 먼저 PDF text owner와
current `dump-pages`를 p8 이후 순서대로 맞춰, 첫 non-matching page와 그 PageItem의 partial
cursor/start_cut/end_cut을 기록한다. p98의 row 2/4 transition이 실제로 한쪽을 과소 사용하거나
다음 source unit을 늦게 밀어내는지 확인한다.

## 다음 절차

1. `fidelity_compare --text-only`로 p8--p18의 page owner 후보를 만든다.
2. 첫 후보 쪽과 그 전쪽을 PDF raster와 `dump-pages`/render tree로 대조한다.
3. source table의 row height, cell cut, declared/stored LINE_SEG 계약을 분석 문서에 기록한다.
4. 원인이 확인되기 전에는 page-count pin, baseline, 또는 전역 native RowBreak 조건을 수정하지
   않는다.

## text-only 전수 후보 결과와 첫 결함

p8--p18 text-only+layout ledger에서 p8, p10, p11은 PDF text multiset이 같았다. p9의 네 PUA
차이는 텍스트 추출 차이여서 source owner 이탈로 확정하지 않았다. 첫 명확한 차이는 **p12**다.

```text
p12: PDF-only 44자, SVG-only 0자
p13: PDF-only 456자, SVG-only 4자
p14 이후: page-owner 지연이 누적
```

direct review sheet에서 PDF p12는 블로그/인플루언서 후기 이미지 두 개를 포함한다. current p12는
`pi=98` row 2의 한 image와 빈 영역만 paint한다. `dump-pages`는 p11→p12 boundary를
`end_cut=[29]` → `start_cut=[29], end_cut=[82]`로 기록하고 p12에서 row `2..3`만 배치한다.
source row 2는 many empty line paragraphs와 Square pictures를 포함하지만, 다음 row 3/4의
visible blog/인플루언서 content는 p12 fragment에 포함되지 않는다.

그러므로 Stage 39의 직접 원인은 “p98 RowBreak cell cut이 PDF가 보여 주는 다음 visible row들을
현재 page에 포함시키지 못하고, row 2의 oversized viewport만 소비한다”이다. 이는 검증 전
일반 cell rewrap을 바꾸거나 table height baseline을 갱신해 해결할 문제가 아니다.

## clean baseline 대조

이 현상이 Stage 36/38의 미커밋 수정 때문에 생겼다는 가설도 분리했다. `6af881f29` clean
worktree를 별도 target으로 빌드하여 같은 HWP와 PDF의 p12를 비교한 결과, baseline도 current와
같이 `38.24%` direct-raster 차이를 냈다. 따라서 p98의 image/cut 결함은 이번 branch에서 새로
도입된 회귀가 아니라 기존 `devel` 결함이다. 이후 수정은 baseline을 갱신하거나 “기존에도
그랬다”는 이유로 건너뛰지 않고, source control·cell unit·PDF page owner 계약을 직접 고친다.
