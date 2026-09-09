# Task M100 #4969 W10-Q2 — horizontal common shaping 최종 support classification

## 최종 판정

W10-Q2의 최종 판정은 **`bounded-subset`**이다. exact portable source에서 계산한 common shaping 결과를
measurement·line selection·TextRun bbox·next origin·GlyphRun이 함께 소비하는 owner와 원자 rollback은
qualified다. 그러나 제품에서 common GlyphRun을 활성화하는 범위는 아래 두 개의 좁은 horizontal LTR lane이며,
Q2 계획에 적힌 모든 horizontal script·direction·surface를 일반 지원한다고 선언할 수는 없다.

기계 판독 정본은
[`w10_q2_final_support_classification.json`](../../tech/investigations/issue-4969/w10_q2_final_support_classification.json)에
고정한다.

## 활성 지원 범위

두 활성 lane은 공통으로 다음 조건을 모두 만족해야 한다.

- `horizontal-tb`, LTR, bidi level 0
- exact portable source와 유효한 face index·digest·byte length
- variation axis 없음, synthetic bold/italic 없음
- direct old-Hangul complex-required target, 한 exact face·style·script의 homogeneous run
- 한 줄·한 run·한 final target, left alignment
- display projection·character border/fill·inline control 없음
- design y offset·y advance가 0이고 glyph·cluster가 각각 4,096 이하
- line selection·bbox·next origin·common GlyphRun이 같은 shaping measurement `Arc`를 소비

| lane | 추가 조건 | 제품 결과 |
| --- | --- | --- |
| stored-boundary strict lane | 저장/기존 line boundary와 final shaping range가 동일 | TextRun 1 + common GlyphRun 1 |
| NO_LS atomic lane | model LineSeg 0, ordinary single interval, edit reflow·stored prefix·split cell 아님 | 재조판된 line width와 TextRun/GlyphRun geometry를 한 번에 게시 |

두 lane 모두 common GlyphRun을 게시하기 전에 sidecar와 unique source 예산을 예약한다. 어느 단계든 실패하면
부분 geometry나 font resource를 남기지 않고 pristine W9 K1 또는 K0 TextRun으로 되돌아간다. 같은 run에 W9
pair adjustment와 common shaping GPOS를 중복 적용하지 않는다.

## backend 지원 분류

| backend/계층 | 분류 | 계약 |
| --- | --- | --- |
| Native·Node WASM producer | qualified | 같은 glyph·cluster·position·advance와 resource identity 생성 |
| CanvasKit | bounded-subset | exact font resource 검증 뒤 strict common GlyphRun draw |
| Canvas2D | deterministic fallback | common glyph-ID replay를 주장하지 않고 TextRun 유지 |
| legacy SVG | deterministic fallback | TextRun 유지 |
| Native Skia | deterministic fallback | portable blob typeface replay가 증명될 때까지 TextRun 유지 |

TextRun fallback이 남는 backend를 common GlyphRun 지원으로 계산하지 않는다. 반대로 fallback은 실패나 누락이
아니다. backend capability가 없는 경우 기존 출력으로 결정적으로 닫는 Q2 보호 계약이다.

## resource·성능 판정

Q2-D5는 exact source 준비를 unique source당 한 번으로 제한하고 page/document 전달에 opt-in font-by-key 경로를
추가했다. 기본 inline JSON 계약은 바꾸지 않았다.

- 한 active page: TextRun 1, common GlyphRun 1, nominal duplicate 0, font blob 1, face 1
- 같은 source의 1/2/8 run: digest·face 준비 1회
- opt-in 1/2/8 page: document generation당 font fetch 1회
- active inline JSON 619,562B, by-key JSON 10,640B
- N1 warm layer 612.521µs, cold 2,296.597µs — wall time은 참고값이며 구조 gate가 판정 권위

따라서 최초 D4의 page별 portable payload 중복 확대 blocker는 닫혔다. 그러나 지원 surface 자체를
multi-line·multi-run·mixed target으로 확대하는 것은 별도 owner·oracle 증거가 필요하므로 Q2 최종 판정에서 열지
않는다.

## 명시적 미지원·후속 범위

다음 입력은 Q2 성공 범위가 아니며 structured reject 또는 기존 TextRun/W9/K0 fallback으로 닫는다.

- RTL과 bidi visual-order product replay
- Latin default liga의 일반 문서 활성화
- multi-line·multi-run batch, mixed target/run splitting
- multi-interval frame, 일반 edit reflow, stored prefix, split-cell recovery, inline control
- center/right/justify/distribute와 ratio identity·expansion
- nonzero GPOS y positioning
- variation instance — W10-Q3
- vertical writing·`vhea`/`vmtx`/`VORG`·`vert`/`vrt2` — W10-Q4
- Native Skia portable blob typeface replay
- public rejected-attempt annotation

미지원 항목을 문서 버전이나 한컴 build 번호로 분기하지 않는다. 현재 문단·font table·glyph coverage·backend
capability를 feature detection해 같은 입력은 같은 disposition으로 닫는다.

## 최종 검증 근거

N2 checkpoint `422a8f7bc` 뒤 최신 `upstream/devel@f6a6bee8f`을 merge commit `c0998c280`으로 병합하고
재자격화했다.

- #4968·#4969·#5952·#6063 cross-impact: 98 passed
- 전체 nextest: 8,568 passed / 43 skipped / 0 failed
- native-skia full library: rhwp 3,946 passed / 13 ignored, auxiliary 182 passed
- native all-target Clippy `-D warnings`: pass
- Docker WASM release: pass, optimized `rhwp_bg.wasm` 9,739,125B
- Studio unit: 1,238 passed / 1 existing skip / 0 failed
- actual CanvasKit replay·bundled font coverage·renderer backend contract·Studio production build: pass
- private corpus·Hyper-V·host 설치 font 사용 0

## Q3 인계

Q2는 variation vector가 비어 있을 때만 활성화된다. Q3는 이 제한을 단순히 제거하지 않는다. 공식 exact variable
font source, `fvar` axis 범위·default, float canonicalization, instance identity·cache key, backend typeface/outline
capability를 먼저 동결한 수정 수행계획을 별도로 승인받아야 한다. Q2의 line owner·atomic rollback·resource
reservation은 Q3가 재사용할 보호 기반이며, axis 적용 결과를 증명하지 못하면 Q2 TextRun fallback을 유지한다.

## 다음 승인 게이트

이 최종 classification 결과와 checkpoint commit은 승인됐다. 다음은 W10-Q3 variable font instance의 현행
source·axis·backend capability를 재감사하고 수정 수행계획을 작성하는 단계다. Q3 계획 승인 전에는 제품 구현을
시작하지 않는다. remote push·PR·GitHub comment·merge는 각각 기존 승인 없이는 진행하지 않는다.
