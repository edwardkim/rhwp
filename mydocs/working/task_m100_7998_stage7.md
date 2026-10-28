# #7998 P4 여섯 번째 절편 — 표현 가능한 반복 경계의 속성·편집

- 기준 checkpoint: `bed75db71ce3a219ae60643dfda7efe70f5ccccc`.
- 2026-10-18 다음 절편 진행 승인. B15/B16의 남은 스타일·행열 편집을 나눠 진행한다.
- 원격 이슈 #7998은 open, assignee edwardkim이며 기존 task branch를 유지한다.
- 로컬 증거 root:
  `/home/edward/mydev/rhwp-review-evidence/pdf-table-feedback-20261017/semantic-stage3/p4-periodic-borders`.
  ignored `output/pr-review/issue7998/p4-periodic-borders`로 연결한다.

## 이번 경계와 독립 근거

원본 source paint는 실제 셀에 속하지만 일반 테두리 편집은 BorderFill을 갱신하므로
비표준 paint의 의미까지 바뀌지 않는다. 좌표가 비슷하다는 이유로 배경/가림 그림을 지우거나
사용자 description에 임의 provenance를 숨기지 않는다. 임의의 틈을 점선으로 정규화해
원래 paint를 잃는 것도 허용하지 않는다.

이번에는 한컴의 정식 반복 테두리로 표현 가능한 source부터 연결한다. 작은 정상 HWPX의
실제 테두리 속성을 바꾼 뒤 한컴 Print를 독립 PDF 입력으로 사용한다. 원본 PDF stroke의
색·굵기·반복 길이·위상과 각 물리 셀의 끊김 없는 의미 경계 및 실제 native paint를 대조한다.
표준 속성으로 동일 패턴을 표현할 근거가 없으면 원본 paint 보존으로 돌아간다.
한컴의 작성 객체를 안다는 이유만으로 PDF 추론 결과를 강제하지 않는다.

생산은 `structured_lowering/native_borders.rs::plan`의 source path·coverage 증거이며
`table_paint::partition`에서 소비할 물리 조각을 결정한다. Cell BorderFill → 공통
border_rendering의 side/style → RenderNode Line → Native/fresh WASM backend → HWP/HWPX
저장 후 한컴 Print까지 연결한다. 기본 shared-edge·부분 side·phase 반례와 행열 삽입/삭제·
명시적 스타일 변경을 확인한다. 임의 dash/bitmap/alpha·가림의 일반 스타일 대체와 병합/
지역 격자, P5는 이 작은 절편의 통과로 완료라고 하지 않는다.

기존 빌드·시각 증거는 before로 보존하며, 수정 방향은 작은 독립 입력부터 검증한다.
새 회귀/fixture 등록은 관련 전체 페이지의 Native/fresh WASM 90% 이상과 직접 판독 뒤다.

## 독립 Print에서 확인한 자격 조건

메인테이너가 같은 폴더에 제공한 `native-dash.pdf`, `native-dot.pdf`,
`native-dashdot.pdf`와 추가 `native-dot-widths.pdf`를 보존했다. 실제 Creator는
Hwp 2020 11.0.0.9166이다. 앞의 dash/dashdot 및 굵기 문서는 A4, dot은
399×360pt이며 각 PDF의 실제 MediaBox를 사용한다. 용지 크기만으로 정상 출력을 폐기하거나
다시 출력하도록 요구하지 않았다. 미지원 테두리 종류를 검증 완료로 넓혀 말하지 않는다.

Dot의 on/off 반복은 굵기와 무관한 상수가 아니었다. 독립 출력에서 확인한
0.2/0.3/0.5/1.5mm의 600dpi on/off tick은 각각 7/10, 10/15, 17/25, 52/78이며
pen tick은 5/7/12/35이다. 확인한 네 굵기만 공통 메트릭으로 연결한다. 나머지 굵기와
임의 간격·위상·가림·명시적 PDF dash/matrix는 이 자격으로 원본 paint를 지우지 않는다.
OWPML `DASH`는 IR `Dot`(API 3)이고 OWPML `DOT`는 IR `Dash`(API 2)다.

부모 외곽 Solid·내부 Dot 및 자식 Solid 대조군에서 정상 source의 실제 부모·자식 표를
함께 구성했다. 완전히 Dot인 외곽의 부모 인식은 아직 보류 경로이며, 내부 Dot 대조군의
통과로 이를 해결했다고 하지 않는다. 정규 반복의 끝과 교차축을 비교하는 f32 합산이
실제 닫힌 경계를 거절하던 경우는 기하 snap 범위를 넓히지 않고 반올림 오차 한계로 보정했다.
추가한 실제 끝점 부족 대조군은 별개다.

구현 중 0.8pt 펜/1.2pt on/1.8pt off 대조군이 가까운 0.3mm 속성으로 잘못 승격되는 것을
실행으로 검출했다. 최근접 width index만으로 동일 굵기를 입증하지 않고, 독립 Print의 실제
펜 굵기와 1 HWP unit 반올림까지 대조하도록 보정했다. 이 실패 producer와 출력은
`before-width-guard/`, `diagnostic-wrong-width/`에 보존했다.

메인테이너는 편집 대조군 `edited/{font-source,row,column,border,column-restored}.pdf`
5개도 제공했다. 원본 PDF에서 작성 글꼴을 알 수 없는 Helvetica 대조군과, 한컴 Print용으로
Arial을 명시한 저장 대조군을 구분한다. 같은 편집 문서의 HWP/HWPX를 이 Print와 비교한다.

## 공통 결과와 실제 소비

- `structured_lowering/native_borders.rs::plan`은 source path 방향·pen·색·각 side의
  실제 on/off 구간과 소유 closure를 확인한다. `structured_lowering.rs`의 native plan이
  Cell BorderFill과 소비할 원본 control을 함께 결정한다. 큰 carrier/coverage 비율로
  틈을 메우거나, 배경·가림 그림을 좌표만으로 지우지 않는다.
- `border_rendering.rs::with_native_border_pattern`은 실제 side의 kind/width로 공통
  `cell_dot_metrics_pt`를 소비한다. 측정 높이나 표 분할을 바꾸지 않고, 기존에 선택된
  side의 끝점과 원점에 물리 반복 간격을 붙인다. 가로·세로의 원점을 별도로 재추측하지 않는다.
- `LineStyle.dash_pattern`을 Native Skia PathEffect, SVG dasharray, Rust Web Canvas
  setLineDash가 소비한다. 실제 Studio의 `paint/json.rs::write_line_style`도 `dashPattern`을
  전달하며 CanvasKit `drawCompoundLine → drawStrokeWithDash → PathEffect.MakeDash`까지
  실제 호출을 계측했다. serde 또는 SVG의 통과로 Studio 경로를 대신하지 않았다.
- CanvasKit 확대 판독에서 명시적 무채움·무테두리 글상자도 기본 검은 외곽선으로 그리는
  별도 소비 결함을 확인했다. 누락 style의 legacy 기본 외곽선과 명시적 투명 style을 구분해
  보정했다. 제품 Rust/WASM이 바뀌지 않은 Studio 수정이므로 WASM을 다시 빌드하지 않았다.
  같은 실제 CanvasKit 회귀에서 기존 조건은 FAIL, 보정본은 PASS다
  (`canvaskit-before-after.json`, `canvaskit-regression-before.log`, `canvaskit-regression.log`).

## 최종 producer와 실행 결과

현재 절편의 Native·WASM 제품 입력 2,445개는 `source-runtime.json`으로 고정했다.
SHA-256은 `67540e0b3fa8d5f20408c59902991c03afe95a02eaacd1ef32ab7b158a511b41`이며
검증 후 변경 없음까지 다시 확인했다. Studio types·renderer·package 입력은 별도
`studio-source-runtime.json`으로 고정했다. 최종 CanvasKit renderer SHA-256은
`8818ebd8508ba665ceb3a673ba6730d77d8be4326f7c5c2905a12b549a20bb3b`다.

- Native Skia: `native-build-final.log` exit 0, `rhwp-candidate` SHA-256
  `fc9aa141fac0bba74b46df3ad6f342db223770abe2a9b5ce86d96ec9d7e36dd2`.
- fresh Docker WASM: `docker-wasm-final.log` exit 0, 완료 후 package 전체를 보존했다.
  19,247,353 bytes, SHA-256
  `93d8b025f9275b8776eba54f478aa567105886ce1883c138b88ebbeed2e9727e`.
  실제 Studio 로드 바이트와도 일치한다(`studio-result.json`, `wasm-build-completion.json`).
- host 공용 `target/pr-review`와 Docker volume을 재사용했다. 7700의 JS/WASM은 최초
  해시와 동일하며 작업 Studio 7793만 갱신했다. 실패·중간 producer는 이름을 분리해
  보존했고 최종 검증에 섞지 않았다. 검증 중 처음 두 집중 case의 producer가 달라져,
  해당 두 case만 최종 source에서 다시 확인했다(`composition-final-head.log`, `border-final-head.log`).

| 실행 범위 | 결과·증거 |
|---|---|
| 67개 원본 × PDF/HWP/HWPX | 각 backend **201건·234쪽**, 최저 Native **97.45634%**, fresh WASM **97.41574%**. 90% 미만·쪽수 차이·누락·오류 0. 같은 backend의 무편집 양 저장 재열기 PNG도 직접 PDF 출력과 동일. `native/results.json`, `wasm/results.json`, `final-validation-summary.json`. |
| 이전 B16과 공통 범위 | 각 backend **219쪽 PNG 전부 동일**. Decimal TSV 대조에서 0.01%p 이상 하락 0. `native-base-comparison.json`, `wasm-base-comparison.json`. 최신 upstream/devel 최종 비교는 P5다. |
| 새 승격·가림·위상·끝점 부족·다른 pen | 각 backend 5개×3형식 **15쪽** 전부 90% 이상. 새 fresh WASM 최저 **99.68019%**. `boundary-wasm/results.json`과 위 전쪽 Native 결과. |
| 독립 굵기·편집 Print | Native/fresh WASM 각각 **12쪽**, 최저 **98.87049% / 97.34801%**. 이 중 원본/편집/복원 5개×HWP/HWPX **10쪽**은 **100% / 98.38668%**. `native-extra/results.json`, `wasm-extra/results.json`. |
| SVG 실제 출력 | 굵기/부모 대조군 각 1쪽×두 backend, 최저 **98.96629%**, 부모는 양쪽 **100%**. `svg-results.json`. |
| 실제 Arial 공급 CanvasKit Print | 굵기 **98.86758%**, 부모 **100%**. 명시적 투명 글상자의 검은 외곽선이 사라졌다. `canvaskit-final/*/silhouette.tsv`, `canvaskit-replay.json`. |
| 실제 Native/Studio API | 부모 Dot side 8개, 중첩 소유·셀 포함·내용 순서·누락/중복·행열 편집/삭제·명시적 빨강 실선 변경·HWP/HWPX 재열기 PASS. `cases-final.log`, `studio-result.json`. |
| 정식 집중 회귀 | 기존 37건과 새 `issue_7998_pdf_periodic_borders` 4건 통과. paint transport Node 6건, TypeScript 확인 통과. 실제 CanvasKit 회귀도 PASS. 전체 회귀/lint/package 검사는 이 절편에서 반복하지 않았다. |

명령은 각 results의 `command`와 실행 로그에 보존했다. Rust 새 case는 기존 자동 수집과
`scripts/run-rust-test.mjs` 경로를 사용하며 CI에 별도 호출을 더하지 않았다.
CanvasKit 회귀는 `VITE_URL=http://localhost:7793 CHROME_PATH=<확인한 Chrome>`으로
`node rhwp-studio/e2e/canvaskit-periodic-borders-issue7998.test.mjs --mode=headless`를 실행했다.
기본 Studio는 Arial이 번들 준비되지 않으면 정상적으로 Canvas2D로 전환한다. 이를 CanvasKit
실행으로 세지 않고, 실제 Arial을 따로 공급한 paint replay와 분리했다.

수정 전 B16 CLI는 같은 source를 **실제 표 1개·Dot 정의 0개**로 보존했다. 최종본은
**실제 표 2개·Dot 정의 8개**이며 `before-after-structure.json`에 같은 계약의 FAIL→PASS를
기록했다. 새 정식 Rust 4건 전체를 과거 checkout에서 컴파일·실행했다고 주장하지 않는다.

## 판정·증거 보관과 남은 범위

2026-10-18 메인테이너 **“시각 판정 통과입니다.”**를 기록한다. 비교 화면은
`E:\tmp\issue7998-periodic-border-review\index.html`이며 원본/독립 Print/실제 저장본·
Native/fresh WASM/CanvasKit PNG와 TSV는 폴더 안의 상대 링크다. Studio의 properties와
row 검토 탭도 열었다. 새 fixture·독립 Print·생성 절차와 SHA는
[`periodic-borders.md`](../../pdf/issue7998/periodic-borders.md)에 있다.

이번 B17은 독립 출력으로 자격을 확인한 **표현 가능한 Dot의 실제 속성·편집** 절편 완료다.
완전히 Dot인 외곽의 부모 인식, 미확인 굵기·다른 패턴/위상, 비표준 paint의 일반 스타일
교체·행열 paint 상속, 병합/지역 격자·unequal span, 첫 inline lead 일반화·본문 메트릭과
B08 원래 글꼴 대체는 계속 남는다. P5의 최신 devel·전체 Rust/lint/package/Worker·최종
시각/PR 게이트도 유지한다. `final_pr_ready=false`이며 P4 전체 완료가 아니다.
원격 push·PR 생성·GitHub 댓글/상태 변경은 하지 않았다.

최종 rustfmt·문서/소스 whitespace 검사를 통과했다. 새 case의 크기가 바뀌어 suite가
재배정되면서 한 차례 0건이 실행된 결과는 통과로 세지 않았다. manifest를 다시 준비한
`new-regression-completion.log`에서 실제 4건 통과를 확인했다. PDF xref의 형식 공백은
원본 바이트로 보존하며 비밀 환경 파일·폰트 바이너리·실행 로그·중간 JSON은 커밋하지 않는다.
