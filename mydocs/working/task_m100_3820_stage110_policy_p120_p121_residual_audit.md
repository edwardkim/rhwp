---
kind: investigation
status: completed
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-10
---

# Task #3820 Stage 110 — 정책연구 p120→p121 잔여 원장 감사

## 범위와 시작 상태

- 브랜치: `task/3820-production-fidelity`
- 시작 commit: `66f129a7f`
- 입력: `samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwp`
- 기준 PDF:
  `pdf/pr3740/hwp/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구)-2020.pdf`
- Stage 98의 미정리 후보 중 p131→p132는 Stage 108에서 stale로 폐기했다. 다음
  우선순위인 p120→p121을 최신 committed source와 PDF로 다시 판정한다.
- 이 stage는 분석부터 시작한다. 실제 owner/geometry 차이가 재현되지 않으면 renderer
  코드를 바꾸지 않고 원장·회귀·증적만 정정한다.

## 기존 계약

실물 회귀 `native_hwp5_earlier_marker_projects_the_p120_footnote_before_body_reset`은
정책연구 전체 215쪽과 다음 형상을 고정한다.

- p120의 `pi=1293`은 line 0..3과 marker 이전 각주 158을 소유한다.
- p121은 같은 문단의 line 4..13을 소유하고 각주 158을 중복 소유하지 않는다.
- p120 본문 bottom은 FootnoteArea separator를 침범하지 않는다.

Stage 98 원장 신호가 이 계약과 충돌하는 실제 잔여인지, 선행 보정 전 산출물을 다시
읽은 stale 후보인지는 최신 text owner와 페이지별 3-way review로 구분한다.

## 완료 기준

1. p120·p121의 본문 line owner, 각주 158 owner, 표/본문/각주/footer 기하를 PDF와
   직접 비교한다.
2. 최신 text-only 원장에서 reference-only/SVG-only, owner-shift, owner-sequence,
   page-boundary 후보를 재생성한다.
3. 기존 회귀가 중복 line·marker 없는 fragment·인접 페이지 유출을 놓치면 최소
   단언을 보강한다.
4. 실제 결함이면 원인과 최소 코드 범위를 이 문서에 먼저 갱신한 뒤 수정한다.
5. 실제 결함이 아니면 provenance가 고정된 2쪽 증적과 focused 회귀로 stale 판정을
   커밋한다.

## 검증 계획

1. 최신 committed source의 동일 binary로 p120·p121 visual sweep을 수행한다.
2. 한컴 PDF와 1-based 페이지별로 확대 판독한다.
3. text-only fidelity 원장을 p120·p121에 한정해 생성한다.
4. 사용자 지정 선행 게이트 `issue_2430_cell_rewrap_threshold`와 p120 exact 회귀,
   관련 실물 회귀 전체를 순서대로 실행한다.

## 결과

### 최신 owner 원장

- 기준 PDF, rhwp SVG, render tree는 모두 `215/215`쪽이다.
- p120·p121의 `text-owner-shift`, `text-owner-sequence`,
  `page-boundary-fidelity`, `visible-text-excess` 후보는 모두 0건이다.
- p121의 PDF/SVG 문자 차이는 0이다. p120의 유일한 차이는 PDF의
  `o + U+0301`과 SVG의 NFC 합성 문자 `ó` 1자의 정규화 표현 차이다.
- 두 쪽의 raw PUA/U+FFFD glyph risk는 0건이다.

따라서 Stage 98의 p120→p121 신호는 현재 source에 남은 owner 결함이 아니라 선행
보정 전 산출물을 다시 읽은 stale 원장이었다.

### PDF 직접 판정

최신 committed source와 같은 바이너리로 정책연구 문서 전체 `215/215`쪽을 export한
뒤 p120·p121의 3-way review를 원본 크기로 대조했다.

- p120은 `pi=1293` line 0..3과 각주 158을 소유한다.
- p121은 같은 문단의 line 4..13을 소유하고 각주 158을 중복하지 않는다.
- p120 본문은 FootnoteArea separator 위에 끝나며 본문·각주·footer가 겹치지
  않는다.
- 요청/완료/누락은 `2/2/0`, 자동 구조 flag는 0쪽이다.

픽셀 match는 p120 `92.21622%`, p121 `90.59494%`다. 글꼴 raster와 antialiasing
차이가 지표를 낮춘다. 자동 구조 flag와 text owner 원장은 정상이나, 확대된 3-way
review에서 p120 선행 표 `pi=1283`의 별도 위치 결함을 확인했다. 픽셀 지표만으로
합격 판정하지 않고 PDF 직접 판독과 render-tree 좌표를 함께 사용했다.

### p120 표 `pi=1283`의 별도 실결함

`pi=1283`은 빈 host 문단의 단일 `6×1` 비-TAC `RowBreak` 표다. 저장 속성은
`HorzRelTo::Column`, `VertRelTo::Para`, `TopAndBottom`, 수평·수직 offset 0이며,
네 방향 outer margin이 모두 `283 HWPUNIT`(약 1mm)다.

- 현재 render tree의 표 bbox는 96dpi에서 대략
  `x=94.5, y=83.2, w=559.4, h=317.2px`이다.
- 한컴 PDF의 외곽선은 같은 단위로 대략 `x=98.3, y=86.7px`에서 시작하며,
  폭·높이는 현재와 사실상 같다.
- 즉 표 내용이나 크기가 아니라 paint origin만 저장 outer-left/top만큼
  좌측·상단으로 이동했다.
- 표 다음 제목의 baseline은 현재 약 `355.28pt`, PDF 약 `355pt`로 일치한다.
  따라서 표 뒤 flow까지 1mm 내리면 이미 맞는 본문을 회귀시킨다.

과거 `57dcc9c6a`의 depth-0 표 outer margin 일괄 적용은 여러 실물 회귀 때문에
`65013dbc4`에서 되돌려졌다. #2097 계열의 empty-host 다중 열 표도 일반 outer-margin
flow 규칙을 사용하지 않는다. 그러므로 이 결함은 broad margin 복원이 아니라,
저장된 단일 empty-host 표 계약을 좁게 식별해 **flow 좌표는 유지하고 표 subtree의
paint origin만 이동**하는 후속 단계로 넘긴다.

### 회귀

다음 exact 실물 회귀를 최신 코드와 `target/pr-review`에서 재실행했다.

```text
CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=target/pr-review \
cargo test --profile release-test \
  --test issue_3738_rowbreak_table_footnote_fragment \
  native_hwp5_earlier_marker_projects_the_p120_footnote_before_body_reset \
  -- --exact
```

결과는 `1/1` 통과다. 이 회귀는 line owner, 각주 158의 단일 소유, separator 위
본문 종료를 고정하지만 `pi=1283`의 outer-margin paint origin은 단언하지 않아
실결함을 놓쳤다. 다음 단계에서 p120·p121·p122 raw line owner와 각주
158·159·160의 단일 소유를 강화하고, 표 bbox와 뒤 본문의 불변 조건을 함께 고정한다.

### 결론

p120→p121의 Stage 98 page-owner 후보는 재현되는 결함이 아니며 `bf2e59c73` 이후
stale 원장으로 폐기한다. 그러나 같은 확대 review에서 p120 `pi=1283` 표의
1mm 좌·상 paint-origin 결함을 새로 확정했다. Stage 110은 두 판정을 혼합하지 않고
원장·현재 review·좌표를 증적으로 보존하며, renderer 수정은 다음 stage에서
분석 문서를 먼저 만든 뒤 수행한다.
