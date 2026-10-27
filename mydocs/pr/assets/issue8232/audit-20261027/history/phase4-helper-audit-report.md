---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase4-helper-audit-report.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase4-helper-audit-report.md` · 원문 SHA-256: `b7e25e6496a340fbce6659ec77ee3aff6a8fbb8d5326f75f9687556e838afd4a`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

## #8232 추가 조사 — 중첩 helper 누락 보완과 독립 Print 대조

전수 조사와 보정은 아직 완료되지 않았습니다. 브랜치는 `audit/8232-px-regressions-20261025`, source head는 `238de6de41a208648f4ade6b494fe433a81c1fce`입니다. 제품 코드·테스트 변경, 이름 변경, PR 제출은 아직 없습니다.

### 감사 수집기의 누락을 발견해 보완했습니다

처음 숫자·기하 후보를 추릴 때 source helper의 `#[test]`/`cfg(test)` 속성을 확인했지만, **테스트 함수 안에 정의한 중첩 helper가 바깥 함수의 테스트 문맥을 상속받지 못했습니다.** 또한 기하 단어가 없는 숫자 단언도 별도 대조가 필요했습니다.

실제로 `src/wasm_api/tests.rs`에서 다음 두 경로를 추가로 발견했습니다.

- `test_hy001_textbox_inline_pictures_render_for_hwp_and_hwpx` → 중첩 `assert_textbox_picture_roundtrip`: 그림 사이 간격을 `525..550px`로 고정합니다.
- `test_hy002_textbox_non_tac_picture_keeps_declared_size` → 중첩 `assert_textbox_picture_size`: 그림 ID뿐 아니라 `width>600,height>50`으로 대상을 선택합니다. 선택 조건과 원본 선언 치수 보존 검사를 분리해 심사해야 합니다.

수집 경로를 외부 `cfg(test)` 모듈의 상속, 중첩 함수, 같은 파일의 helper 호출, 기하 단어에 의존하지 않는 숫자 단언까지 보완했습니다. 동명 함수·method dispatch·외부 호출은 구문 목록만으로 해결됐다고 판정하지 않습니다.

통합 원장은 **9,664함수/helper 그룹·2,103파일**입니다. 최초 2,707개 기하 후보와 399개 렌더 생성 그룹에 숫자 단언 후보 6,558그룹을 추가한 수입니다. HTTP 코드·문자 개수·바이트/형식 계약도 포함하므로 **위반 수가 아닙니다.** 같은 후보를 wrapper로 중복 등록한 것도 아닙니다.

종전 source 기하 후보 529그룹은 직접 수치의 의미를 분류했습니다. 이 결과는 함수 내부 수치 목적의 분류이며, 중첩·외부 helper·전체 파일의 지침 적합 판정과 구분합니다. 원본 HU·glyph em·직접 만든 PaintOp·검색 offset 등을 절대 페이지 좌표로 오분류하지 않으며, 호출 근거가 부족한 항목은 미검증으로 유지합니다.

### 파일 누락을 PASS로 숨기는 경로도 실제 재현했습니다

공용 `src/renderer/layout/integration_tests.rs::load_document`는 fixture가 없을 때 `None`을 반환하고, 호출 테스트는 곧바로 `return`합니다. 기존에 빌드한 정확한 Native 테스트 바이너리를 빈 fixture 환경에서 실행하자 다음 결과가 나왔습니다.

```text
renderer::layout::integration_tests::tests::test_export_svg_produces_output
테스트 파일 없음: samples/hwpspec-w.hwp — 건너뜀
1 passed; 0 failed; 0.00s
exit code 0
```

이는 **누락 환경의 음성 대조**입니다. 정상 렌더링이나 시각 검증 PASS 증거로 세지 않습니다. 바이너리·source hash, 실행 cwd와 로그를 감사 원장에 고정했습니다. 단순히 PASS 수만 유지하는 보정으로 끝내지 않습니다.

### 종료된 추가 비교

앞선 17조건의 Print/캡처 시도는 모두 종료 기록을 확보했습니다. 그중 Print 실패 2조건은 정확한 입력·PDF hash 및 Print 실행 근거가 남은 기존 자료를 재사용해 현재 source로 재캡처했습니다. **16조건·각 backend 210쪽**의 전쪽 TSV를 확보했으며, 나머지 1조건은 Native/WASM 1쪽 대 Print 2쪽 불일치로 미해결입니다. 시도 종료를 전체 시각 통과로 보고하지 않습니다.

기존 Print 재사용 조건의 전쪽 최저:

| 입력 조건 | 쪽수 | Native raw | fresh WASM raw |
| --- | ---: | ---: | ---: |
| 배포 문서 `distribution_doc.hwpx` — 실제 HWP5 CFB | 3 | 99.69646% | 99.68550% |
| footer logos | 13 | 93.40074% | 93.28630% |

`#6551`의 같은 문서에서도 7쪽은 Native 99.51472% / WASM 99.51876%, 8쪽은 97.60422% / 97.65753%입니다. 다른 쪽의 높은 점수를 8쪽의 px 자격으로 쓰지 않습니다. 전체 46쪽 최저는 69.44170% / 69.45610%이며, 검사 대상 쪽과 전체 최저를 분리했습니다.

### 복합 검사 안의 정상 Print 비교도 구분했습니다

`src/renderer/skia/renderer.rs::raster_options_scale_output_size`는 직접 만든 장면의 배율 검사뿐 아니라 독립 Print 비트맵 3조건의 내용 보존 검사도 포함합니다. 잉크 기준 `min(R,G,B)<232`, 반경 2px, 양방향 미지원 잉크와 합집합 분모의 98% 기준은 현행 raw 실루엣 계산과 대응합니다.

이 세 원본·독립 Print PDF를 현재 PDFium 5.13.0과 고정 Native/fresh WASM으로 다시 검증한 결과, **각 backend 3쪽 모두 raw 100%**입니다. 대표 review/overlay에서 격자와 색상 띠의 정합도 확인했습니다. 과거 PDFium 5.12.1의 기준 PNG와 이번 결과를 섞지 않습니다. 기존 Rust 검사 자체를 이번에 실행했다고 주장하지 않으며, 실제 실행 등록·직접 소비 경로 감사는 별도로 남깁니다.

### 새로 찾은 두 중첩 검사의 독립 대조

원 수정 기록에서 `hy-001`은 TAC 그림 사이 공백 advance 누락, `hy-002`는 non-TAC 그림의 과도한 축소가 원래 결함임을 확인했습니다. 네 입력의 저장 제품을 각각 확인하고 버전에 맞는 한컴 MCP Print를 새로 생성했습니다. Native/fresh WASM 전쪽 TSV는 다음과 같습니다.

| 입력 | 독립 Print engine | 실제 검사 쪽 | Native raw | fresh WASM raw |
| --- | --- | ---: | ---: | ---: |
| `hy-001.hwp` | 2020 | 2 | **81.85601%** | **81.85987%** |
| `hy-001.hwpx` | 2024 | 2 | **81.04687%** | **81.04735%** |
| `hy-002.hwp` | 2020 | 2 | 99.03462% | 99.03339% |
| `hy-002.hwpx` | 2020 | 2 | 98.77377% | 98.77579% |

모두 2쪽 문서이며 원본별 독립 Print와 쪽수를 맞춰 비교했습니다. `hy-001` 2쪽은 본문 자형·표와 로고의 위치 차이가 보이므로, 고정 간격을 먼저 바꿔 통과시키지 않습니다. `hy-002`의 높은 점수도 현재 selector의 타당성을 자동으로 증명하지 않습니다. 선언 그림 치수·소속과 독립 출력의 대응을 계속 확인합니다.

### 처리 원칙과 남은 작업

- 잘못된 기존 px 단언의 보존을 전제로 하지 않습니다. 독립 근거에서 확인한 유효한 결함 검출 목적을 현행 지침에 맞게 보호합니다.
- 실제로 보정하는 파일·함수만 `issue_8232` 기준으로 명명하고, 관련 helper·module·manifest·생성/스크립트/E2E 호출을 갱신합니다. 이전 이름의 wrapper·중복 등록은 만들지 않습니다.
- 전수 조사에서 미분류·호출 연결·독립 좌표·직접 판독이 남아 있으므로 전체 완료로 표시하지 않습니다. 조사 결과를 이슈에 누적한 뒤 부적합 검사 재작성과 필요한 출력 보정·재검증을 진행합니다.
- ignored `output/issue8232-audit/`에 원장·TSV·review/overlay·실패/정상 로그를 보존합니다. PR은 제출하지 않습니다.


### 기존 PASS와 독립 출력의 실제 차이도 확인했습니다

동일 source의 기존 Native 단위 검사 두 건을 정확한 이름으로 실행했습니다. `hy-001`·`hy-002` 모두 각각 1 PASS였습니다. 전체 회귀를 실행한 결과는 아닙니다.

그런데 `hy-001`의 현재 Print RenderTree에서 원본 문단 27의 두 로고 시작점 간격은 **약 539.0 CSS px**이고, 독립 PDF의 두 image object 경계에서는 **532.99334 CSS px**입니다. 약 **6.01px** 차이가 기존 `525..550` 조건 안에 들어갑니다. 두 로고는 원본 문단/컨트롤 소속과 직접 review로 대응시켰고, PDF object는 Form 안이 아닌 page level 0입니다. PDF point→96 CSS px 좌표 변환 및 JSON tree의 0.1px 표시 정밀도를 구분했습니다.

이는 기존 PASS가 원래 보호 목표인 그림 사이 공백 advance의 정확성을 충분히 입증하지 못한다는 실제 대조입니다. 테스트가 출력하지 않은 내부 좌표를 PASS 결과에서 역추정하지 않습니다. 기존 단위 검사의 실행 경로와 별도 Print 소비 경로의 관측을 각각 기록했습니다.

`hy-002` HWPX 원본의 해당 그림 선언은 **48190×4331 HU**이며, 현재 bbox는 약 **642.4×57.7 CSS px**입니다. 독립 PDF image 경계는 약 **642.215×57.697 CSS px**입니다. 크기 selector를 보호 계약 자체로 유지하기보다 그림 소속/ID로 식별한 뒤 원본 선언 치수와 실제 크기의 대응을 검사하는 근거로 연결합니다. 아직 테스트를 수정한 것은 아닙니다.
