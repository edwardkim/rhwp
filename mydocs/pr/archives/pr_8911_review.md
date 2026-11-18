# PR #8911 리뷰 — fix: 저장 줄 없는 글 문단의 합성 vpos 를 저장 되감김 쪽 경계 판정에 쓰지 않는다 (#8904)

## 최종 판정

**승인 — 해당 원본·변경 범위의 로컬 검증 완료.**

[통합 PR #8923](https://github.com/edwardkim/rhwp/pull/8923)의 code 제출 head `38ce926603d14ff662e29fe6a92acde67c73c0ec`에서 최신 CI를 통과했다. 문서 trailing head의 checks와 실제 병합은 이후 별도로 확인한다. 이 판정은 contributor 원 head와 아래 메인터너 보정을 포함한 검증 code `21d62361fa670858e63fdfc475c236f112e20a3d`에 대한 결과다.

## 접수 정보

- [원 PR](https://github.com/edwardkim/rhwp/pull/8911), 작성자 planet6897, reviewer jangster77. 원 source head는 변경하지 않았다.
- 원 head `f7021fc7b59ba365a1d09c1621f6a75ad583e8d4`; 통합 base `5ea1227505978156082121f1601cb03044a0583f`; 검증 code `21d62361fa670858e63fdfc475c236f112e20a3d`.
- live 원 PR head 재확인: 2026-11-11T02:22:08.125563+09:00, 접수 SHA와 동일, 상태 OPEN.
- 관련 이슈 표기는 원 PR 본문을 따르며, 통합 후 실제 해결 범위와 각 이슈의 잔여를 확인해 댓글·close한다.

## 주장과 코드 검토

가시 글 문단의 합성 vpos를 저장 되감김 증거로 사용하지 않음.

저장 되감김의 쪽 경계 추론에는 양쪽 문단의 비합성 저장 줄만 쓴다. 로드 시 만든 합성 vpos를 저장 증거로 쓰지 않아, 실제 문단의 넘친 셋째 줄이 독립 Print처럼 다음 쪽으로 이동한다. 2쪽 꼬리·3쪽 첫 줄의 소속과 내용 누락·중복을 확인하고 진짜 저장 reset의 기존 계약을 유지한다.

주요 생산 경로:

- `src/renderer/typeset.rs`

조판 규칙은 원본 저장 줄·프레임·개체 소유와 독립 Print를 근거로 검사한다. 새 메인터너 보정은 문서명·문서 식별자·쪽 번호에 따른 제품 분기나 판정 문턱 완화를 추가하지 않았다. 실제 회귀·원본·반례로 확인한 범위와 기존 차이는 구분한다.

## 검증 입력과 결과

96 DPI·Print·해당 compat·현행 Visual Sweep·색상 경계 보정 전2px raw 기준이다. 같은 base/후보 원본·PDF와 글꼴 공급을 사용했다. 표의 수치는 각 문서의 전쪽 최저이며 대표 페이지 값과 혼동하지 않는다.

| 원본 | 후보 쪽수 | Native base → 후보 최저 | fresh WASM base → 후보 최저 | 후보 전쪽 TSV |
| --- | ---: | ---: | ---: | --- |
| `36376635-synthetic-tail-saved-reset.hwpx` | 3 | 61.31 → 100.00% | 61.32 → 100.00% | [Native](../assets/planet6897_clean19_20261110/pr8911/cases/pr8911-36376635-synthetic-tail-saved-reset/native/silhouette.tsv) · [WASM](../assets/planet6897_clean19_20261110/pr8911/cases/pr8911-36376635-synthetic-tail-saved-reset/wasm/silhouette.tsv) |

고정 base의 기존 쪽수 불일치 원본은 PDF와 대응하는 모든 페이지의 값과 PDF에 없는 추가 RHWP 쪽을 함께 보존했다. `base-base-producer-link.json`에 원래 count 실패·추가 쪽 hash를 기록했으며 이를 통과 자료로 표시하지 않는다. 후보의 전체 쪽수는 독립 PDF와 일치한다. 입력 별칭은 동일 byte의 정식 fixture와 대응하며 비공개 글꼴은 Git에 포함하지 않았다.

- `samples/issue8904/36376635-synthetic-tail-saved-reset.hwpx` / `pdf/issue8904/36376635-synthetic-tail-saved-reset-2020.pdf`; 입력·PDF 해시와 producer·글꼴·실제 쪽수는 [Native manifest](../assets/planet6897_clean19_20261110/pr8911/cases/pr8911-36376635-synthetic-tail-saved-reset/native/manifest.json) 및 [WASM manifest](../assets/planet6897_clean19_20261110/pr8911/cases/pr8911-36376635-synthetic-tail-saved-reset/wasm/manifest.json).

## 회귀와 실제 소비 경로

- 최소 글꼴 환경의 전체 회귀: `Summary [ 545.222s] 13227 tests run: 13227 passed (2 slow), 45 skipped`. 개인 글꼴을 제거하거나 비활성화하지 않고 접근 차단한 Mac 검증 환경이다.
- Native Skia 전체 lib·missing-picture·direct-PDF, Native/WASM/all-target Clippy·workspace build·fmt·manifest base 비교를 최종 제품 코드에서 확인했다.
- [전체 Rust 결과](../assets/planet6897_clean19_20261110/rust-final-validation.json), [Native 전쪽](../assets/planet6897_clean19_20261110/native-final-validation.json), [WASM 전쪽](../assets/planet6897_clean19_20261110/wasm-final-validation.json), [실제 browser](../assets/planet6897_clean19_20261110/consumers/browser-validation.json), [SVG](../assets/planet6897_clean19_20261110/consumers/svg-validation.json)를 연결한다. Studio 44건 중 실제 Canvas2D 28·CanvasKit 16건이며, 글꼴 요구 목록 미준비 6건은 Canvas2D fallback으로 기록했다. 지원 CanvasKit 15건의 실제 software surface·readiness를 확인했다. 별도의 누락 그림 screen 반례 1건은 `image:dataMissing`과 readiness 실패를 그대로 기록하며 통과로 바꾸지 않았다. 모든 backend의 glyph 또는 RGB 동일을 주장하지 않는다.
- Mac fresh WASM은 locked wrapper `--target web --out-dir pkg --no-opt` 로컬 대체 빌드다. Docker 최적화 결과가 아니다. root pkg·Studio public·실제 초기화 응답 hash와 실제 Canvas2D/지원 CanvasKit·SVG 소비 결과는 통합 검증 기록에 별도로 연결한다.
- 필수 편람 HWPX Native 383쪽과 collaborator 정본 보호 TSV 전체는 통합 후보에서 실행했다. 다른 PR의 보호 문서 점수로 이 PR 주장의 직접 증거를 대신하지 않는다.
- 원 회귀 source(자동 수집 suite에서 실제 실행):

- `tests/cases/issue_8904_synthetic_tail_saved_reset.rs`

## 같은 페이지 시각 증적

전쪽 TSV와 별도로 원본 내용·쪽 소속·누락/중복·변경 영역·미세 간격을 직접 판독했다. raw·PDF·review·overlay는 각각 같은 원본·페이지·backend다. 100.00%는 2px 내용 실루엣 지표이며 RGB 전체 픽셀 동일을 뜻하지 않는다.


36376635-synthetic-tail-saved-reset.hwpx 2쪽 Native
![raw](../assets/planet6897_clean19_20261110/pr8911/cases/pr8911-36376635-synthetic-tail-saved-reset/native/raw-002.png)
![독립 한컴 PDF](../assets/planet6897_clean19_20261110/pr8911/cases/pr8911-36376635-synthetic-tail-saved-reset/native/pdf-002.png)
![review](../assets/planet6897_clean19_20261110/pr8911/cases/pr8911-36376635-synthetic-tail-saved-reset/native/review-002.png)
![overlay](../assets/planet6897_clean19_20261110/pr8911/cases/pr8911-36376635-synthetic-tail-saved-reset/native/overlay-002.png)
36376635-synthetic-tail-saved-reset.hwpx 2쪽 fresh WASM
![raw](../assets/planet6897_clean19_20261110/pr8911/cases/pr8911-36376635-synthetic-tail-saved-reset/wasm/raw-002.png)
![독립 한컴 PDF](../assets/planet6897_clean19_20261110/pr8911/cases/pr8911-36376635-synthetic-tail-saved-reset/wasm/pdf-002.png)
![review](../assets/planet6897_clean19_20261110/pr8911/cases/pr8911-36376635-synthetic-tail-saved-reset/wasm/review-002.png)
![overlay](../assets/planet6897_clean19_20261110/pr8911/cases/pr8911-36376635-synthetic-tail-saved-reset/wasm/overlay-002.png)
36376635-synthetic-tail-saved-reset.hwpx 3쪽 Native
![raw](../assets/planet6897_clean19_20261110/pr8911/cases/pr8911-36376635-synthetic-tail-saved-reset/native/raw-003.png)
![독립 한컴 PDF](../assets/planet6897_clean19_20261110/pr8911/cases/pr8911-36376635-synthetic-tail-saved-reset/native/pdf-003.png)
![review](../assets/planet6897_clean19_20261110/pr8911/cases/pr8911-36376635-synthetic-tail-saved-reset/native/review-003.png)
![overlay](../assets/planet6897_clean19_20261110/pr8911/cases/pr8911-36376635-synthetic-tail-saved-reset/native/overlay-003.png)
36376635-synthetic-tail-saved-reset.hwpx 3쪽 fresh WASM
![raw](../assets/planet6897_clean19_20261110/pr8911/cases/pr8911-36376635-synthetic-tail-saved-reset/wasm/raw-003.png)
![독립 한컴 PDF](../assets/planet6897_clean19_20261110/pr8911/cases/pr8911-36376635-synthetic-tail-saved-reset/wasm/pdf-003.png)
![review](../assets/planet6897_clean19_20261110/pr8911/cases/pr8911-36376635-synthetic-tail-saved-reset/wasm/review-003.png)
![overlay](../assets/planet6897_clean19_20261110/pr8911/cases/pr8911-36376635-synthetic-tail-saved-reset/wasm/overlay-003.png)

## Merge 후 contributor PR comment 계획

최종 merge SHA·CI·이 검토 문서와 주장 페이지의 raw/PDF/review/overlay를 이미지로 포함해 한국어 존댓말로 안내한다. 메인터너 보정은 기여자가 해결한 부분과 추가로 발견한 결함·보정·검증 이유를 구분한다. 원 fork branch는 삭제하지 않는다.


- 검증 code `21d62361fa670858e63fdfc475c236f112e20a3d`, CI 제출 head `38ce926603d14ff662e29fe6a92acde67c73c0ec`, 통합 PR #8923. 실제 merge SHA는 병합 후 확인해 이미지 URL과 안내 본문에 사용한다.
- 위의 원 기여·메인터너 보정 이유·수치를 유지하고, 같은 입력·쪽·backend의 raw/PDF/review/overlay를 실제 Markdown 이미지로 표시한다. 전쪽 최저를 대표 쪽 점수로 바꾸지 않는다.

| 원본 조건 | comment에 표시할 검토 쪽 | 증적 디렉터리 |
| --- | --- | --- |
| `pr8911-36376635-synthetic-tail-saved-reset` | 1, 2, 3 | [Native/fresh WASM](../assets/planet6897_clean19_20261110/pr8911/cases/pr8911-36376635-synthetic-tail-saved-reset) |
