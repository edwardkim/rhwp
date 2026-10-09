---
kind: working
status: completed
last_verified: 2026-10-09
---

# #7688 3단계 — Native PNG 수식 글꼴 소비 보정

독립 기준은 `samples/eq-01.hwp`와 1단계에서 검증한 동일 원문의 한컴 2020 Print PDF다.
사용자가 Native 수식의 글꼴 보정 진행을 승인했다. source 기준은 `6f66932a73ee6fd05a74a83bdc438c9499a89a12`다.

## 구현 계약과 실제 소비

본문 `text_typeface_candidates`의 custom → system → bundled → legacy 후보 수집 본체를
`typeface_candidates_for_families`로 공유했다. 본문 후보·우선순위·문자 선택은 유지한다.
수식은 기존 math/CJK family 순서와 CJK 정체 규칙을 유지하며 web의 명조 대체 family를 포함한다.
문자별 glyph coverage는 본문과 같은 `select_typeface_for_character`를 사용한다.

`SkiaLayerRenderer::render_node`의 `PaintOp::Equation` 두 scale 분기에서 같은 `EquationFonts`를
전달한다. 재귀 `render_box`와 bracket의 text 분기도 이를 전달한다. 실제 `draw_text`가
선택한 동일 face의 연속 run을 만들고 그 font와 폭을 가운데 정렬·draw_str 양쪽이 사용한다.
LayoutBox의 줄 소속·원점·저장 scale과 뒤쪽 페이지 배치는 바꾸지 않았다.
전체 renderer 이관이나 sample ID/좌표 clamp 예외를 추가하지 않았다.

## 선행 실행

공용 `target/pr-review`의 수정 전 Native 바이너리를 ignored 증적에 별도로 보존했다.
첫 Native PNG에서 수식 한글 네모가 사라진 것을 직접 판독했다. 동일 PDF의 2px 실루엣은
system screen/print 96.29090%, custom Batang/Noto screen/print 96.53925%다.
저장소 Noto Sans KR Regular의 custom 공급 대조군은 95.63343%다.
이는 선행 결과이며 최종 source SHA·fresh WASM·필수 검사·정식 회귀는 아래에 이어 기록한다.

증적 루트: `output/pr-review/renderer-backend-audit-20261009/equation-font-fix/`.
공통 crop·화살촉과 다른 backend 글꼴 등록은 남은 #7688 범위다.

## 검출 증거와 최종 출력

제품 코드 commit은 `8c59d9f29`, 정식 회귀·OFL 자원 commit은 `ca7f77769`다. 후자는
Rust/WASM 제품 입력을 변경하지 않아 같은 Docker 빌드의 WASM을 재사용했다.
WASM과 Studio/public JS·WASM 해시는 각각 일치하며 초기화 요청도 확인했다.
Vite는 WASM URL 한 줄과 source map을 변환하므로 응답 JS를 그대로 원본 hash와 비교하지 않고,
그 변환을 대조해 원본 JS hash `70cde06a…`와 실제 WASM 응답 hash `d509f8a1…`를 확인했다.

`tests/cases/issue_7688_native_equation_fonts.rs`는 원본의 수식 op 3개를 동일한 쪽·bounds에서
분리하고 `PageLayerTree` 생성자로 본문 text source table을 다시 묶는다.
Regular 400과 Light 200의 독립 OFL 윤곽선이 실제 PNG에 반영되고, 한글 glyph가 없는
Latin-only Batang 후보 뒤에서 같은 CJK 윤곽선으로 대체되는지 screen/print 양쪽을 확인한다.
절대 좌표·SVG hash·픽셀 golden을 새 기준값으로 잠그지 않았다. 자원은 FontTools 4.62.1로
결정적으로 생성되며 기존 OFL 원문을 연결했다.

수정 전 같은 6f 제품 코드에서 `equations ignored supplied font outlines`로 FAIL,
수정 후 PASS(0.253초)를 확인했다. 처음의 개체 수 전제 오류와 분리한 페이지의 text source table
오류는 검사 준비 오류이므로 결함 검출 증거에서 제외하고 별도 진단 로그로 보존했다.
이후 before 검사는 직전의 동일 6f library를 재사용하고 변경한 검사만 다시 컴파일했으며
제품 3개 파일의 hash를 고정한 뒤 현재 코드로 복원했다.

최종 Native PNG의 같은 한컴 PDF 대비 2px 일치율(각 mode의 screen/print 동일):

| 글꼴 공급 | 일치율 | 직접 확인 |
| --- | --- | --- |
| 시스템 | 96.29090% | 수식 한글 표시·분수·괄호·본문 순서 |
| custom Batang/Noto 경로 | 96.53925% | 수식 한글 표시; 일부 숫자 획 굵기 차이 남음 |
| custom Regular | 95.63343% | 동일 원문의 glyph 윤곽선 공급 |
| custom Light subset | 95.25811% | 얇은 윤곽선 공급·glyph 누락 없음 |
| Latin-only 선행 + Regular 대체 | 95.63343% | 정상 Regular 대조군과 동일한 실제 출력 |

Native/fresh WASM SVG Visual Sweep은 1쪽 모두 95.89322%이며 누락·90% 미만 쪽이 없다.
WASM 수정 전후 렌더 트리는 byte 동일하다. 기본 Native PNG의 문단·탭·표·crop·그룹 도형
screen/print 정상 대조군 10개도 수정 전후 byte 동일하다.
수식은 Native PNG 두 profile에서 변경됐다.

최종 비교 이미지: ignored `equation-font-fix/native-before-after-review.png`.
전체 출력·페이지 TSV·font 입력 hash·binary hash는 같은 ignored 증적 디렉터리에 보존했다.
최저 점수만으로 glyph 보존을 대신하지 않고 위 실제 이미지와 font 소비 회귀를 함께 확인했다.

## 최종 검증

검증 head는 `ca7f777695b75f7a77a157eb25f77f6ea48c9b95`, 정책 base는
`6f66932a73ee6fd05a74a83bdc438c9499a89a12`다. 공용 target은
`/home/edward/mygithub/rhwp/target/pr-review`, 빌드 동시성은 `CARGO_BUILD_JOBS=2`다.
실제 명령·종료 코드·소요 시간은 ignored `final-checks-results.json`과 각 단계 로그에 연결했다.

| 검사 | 결과 |
| --- | --- |
| fmt, Native/WASM/workspace all-targets Clippy, workspace build | 모두 PASS |
| Native Skia lib Clippy | PASS |
| suite manifest / source unit tier 정책, 고정 base 비교 | PASS |
| `cargo nextest run --locked --cargo-profile release-test --tests --test-threads 4 --no-fail-fast` | 10,551 PASS, 0 FAIL, 50 skipped; 실행 628.771초 |
| `run-rust-test.mjs issue_7688_native_equation_fonts`, `--features native-skia` | 수정 전 FAIL / 수정 후 1 PASS, screen/print 두 profile |
| Native Skia lib 회귀 | rhwp 3,927 PASS, 0 FAIL, 13 ignored; workspace lib 182 PASS |
| `run-rust-test.mjs issue_2225_missing_picture_placeholder`, `--features native-skia` | 2 PASS |
| `run-rust-test.mjs render_p37_direct_pdf_export`, `--features native-skia` | 4 PASS |

`nextest` 권장 버전 경고(현재 0.9.137, 권장 0.9.140)는 테스트 실패가 아니다.
새 검사와 자원 외에 baseline·허용치·기존 검사를 변경하지 않았다.
최종 Native CLI 빌드도 PASS다. 바이너리 hash `6aa35458…`는 시각 검증용 `c1e32e43…`와
달랐으므로 동일 입력·profile·font 경로로 정상 대조군 10개와 수식 공급 대조군 10개를 다시
출력했다. **20개 모두 기존 최종 PNG와 byte 동일**하다. 두 hash와 실제 명령·새 PNG는
ignored `final-validation-provenance.json`, `final-binary-png-results.json`에 보존했다.
완료한 범위는 Native 수식의 font 자원 소비·glyph fallback과 그 재발 검출이다.
절차는 [출력 백엔드 소비 경로 검증](../manual/verification/renderer_backend_verification.md)에
연결했으며, 실제 glyph 윤곽선이 같은지를 제품 paint 진입점에서 검사한다.

## 남은 범위

이 보정은 #7688 전체 종료가 아니다. 공통 crop·화살촉과 CanvasKit 별도 글꼴 등록은 후속 범위다.
동일 family의 여러 style을 custom 경로에서 하나의 face로 적재하는 기존 정책 때문에,
custom 대조군의 숫자 획 굵기 차이가 남는다. 이 보정으로 style 공급 전반을 해결했다고 주장하지 않는다.
CanvasKit parity의 기존 report-only 차이도 자동 통과로 바꾸지 않았다.
수식 scale 두 분기는 코드에서 같은 context 전달을 확인했지만, scale 없는 입력의 별도 실행 증거를
이번 회귀와 동일하게 확보했다고 주장하지 않는다. 다른 OS·GPU·편집 후 부분 갱신도 이번 보정의
실행 검증 범위가 아니다. 이슈 전체를 종료하거나 원격 PR을 등록·push하지 않았다.
