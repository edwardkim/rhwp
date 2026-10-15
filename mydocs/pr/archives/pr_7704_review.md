---
kind: review
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-10-13
---

# PR #7704 검토 — 표 조각의 반복 셀 검사 성능

## 최종 판정

**메인터너 보정 후 수용 가능 — #7704 확대 보정 범위의 로컬 검증 완료.**

원 contributor head `896e64ff6d876e676d6ebf987d6669b47653912a`와 보정 통합 후보 `152c2fcbcccd08646fa98273089833ab9237e395`를 구분한다. 원 커밋을 보존한 별도 메인테이너 [PR #7877](https://github.com/edwardkim/rhwp/pull/7877)에 성능 개선·HWPX 차이·전체 회귀 보정이 포함됐다. 최신 devel `552f5228d2dbfd6046c838f14241704f1bcb8a05`가 후보에 포함돼 있다.

최종 전체 회귀 10,219PASS/0FAIL/45skip, Rust lint·workspace build·Native Skia와 Native/fresh Docker WASM 19문서835쪽 최저90.47526%를 확인했다. 마지막 #5543의2020 Print 대조4쪽은95.35221%이며 Native/WASM PNG가같다. 정확한 code head의6개 GitHub workflow도모두성공했다. [통합 self-review](pr_7877_review.md)에 실제 경로·독립 기대·입력 commit·시각 판독·CI를 연결한다. 원 PR은 통합 PR 병합 후 대체 완료로 종료할 계획이며 #7639 전체 성능 개선 종료를 주장하지 않는다.

아래는 이전 후보·실패·철회 과정을 보존한 기록이다. 최종 상태는 이 절과 마지막 FM 기록을 기준으로 한다.

### 이전 16쪽 한정 검증 기록

**메인테이너 16쪽 보정·시각 판정 통과 / PR 전체 시각 gate 보류.**
원 기여자의 두 순회 개선은 계산 의미를 보존한다. devel `920dc29284` 통합본에 메인테이너 보정을 적용한 Native/fresh Docker WASM 전11문서·393쪽 재출력을 완료했다. HWPX 16쪽은 98% 이상이며 작업지시자의 직접 판정을 통과했다. 다른 문서의 점수는 원 PR 출력과 같고, 이번 수정으로 새 미달이 생기지 않았다. 전체 회귀 10,570건과 Native Skia 필수 3종 검사는 통과했다. 그러나 기존 5문서·312쪽의 미달을 면제한 승인은 없으므로 원격 승인·통합 단계로 진행하지 않는다. 검증 종료 시 원격 devel `bfa28a77a2`의 추가 통합을 확인했으며 이 기준의 재검증은 아직 하지 않았다.


## 접수와 트리아지

- PR: [#7704](https://github.com/edwardkim/rhwp/pull/7704), 작성자 `oksure`, base `devel`, [#7639](https://github.com/edwardkim/rhwp/issues/7639)는 `Refs`이며 부분 개선이다.
- 검토 head: `896e64ff6d876e676d6ebf987d6669b47653912a`.
- code head: `3e746bf676611983e6e002e355fd2e25258d05ec`; devel 통합 head `0ebdfc42e5dc61d6ca0f9323aedec631cd7cc131` 이후 현재 head까지는 WASM PNG 두 개 추가뿐이다.
- 최신 비교 base: `f79f2e1d6831a2187657b9c60deabc5d7c8b8e0d`.
- 담당자 `oksure`, reviewer `edwardkim`, milestone `v1.0.0`, labels `layout`, `performance`, `rust`, `table` 반영·재조회 완료.
- maintainer 일반 + 접수/로컬 검증/시각 증적/다수 PR 경로. 최신 base 충돌 simulation 결과 `696dae99d74f14051a9faeb7f4279a94a9c96193`, 충돌·whitespace 오류 없음.

## 원 기여 성능 변경의 구현과 조판 계약

| 변경 | 의미 보존 근거 | 실제 소비 경로 |
| --- | --- | --- |
| `budget.rs`의 `stored_row_heights` | 범위 내 행, `row_span == 1`, 높이 `< 0x8000_0000` 셀만 행별 0.0에서 같은 순서로 max. 빈 행·범위 밖 셀의 취급 동일 | 첫 source frame의 행 끝 인증, source complete frame의 컷 인증 → 스캔 예산·이월 → 기존 조각 배치 |
| `typeset.rs`의 table-wide vpos reset | 기존 행별 존재 검사와 새 범위 내 셀별 존재 검사는 동치. 셀 안 문단을 가로지르는 양수→0 이하 전환과 synthetic LineSeg 제외 유지 | `scan/runner`, `row_step`, `block/prepare`, `whole_fit`, `block/entry`, `continuation/fragment/budget` |

새 helper는 셀 내부 판정을 원 코드에서 추출한 것이다. 셀 사이에 previous vpos를 공유하지 않는다. 순회 순서만 바뀌며 판정 함수에 상태 변경이 없다. row별 caller는 동일 helper를 소비한다. 새로운 문서 ID, 픽셀 여유, clamp, 분할 허용치, 저장 정보 신뢰 조건을 추가하지 않았다. 시작/끝 컷, 요구/예약 높이, 예산 실패 및 실제 paint 계약 자체를 바꾸지 않는다.

행×전체 셀 순회를 전체 셀 한 번 순회로 바꾼다. Table에 지속 캐시를 도입한 변경은 아니므로 편집 후 캐시 무효화 계약이 추가되지 않는다. 다른 행별 탐색과 편집 성능 전체 해결은 범위 밖이다.

## CI와 성능 증거

- 원 code head의 [Full CI 38024324551](https://github.com/edwardkim/rhwp/actions/runs/38024324551): Build & Test, 세 Clippy를 포함한 Lint, Native Skia 실제 실행·성공 확인.
- `eb2da208`은 원 code head에서 Native PNG 두 개만 더했다. 현재 [CI 38122343388](https://github.com/edwardkim/rhwp/actions/runs/38122343388)의 preflight는 이 성공 후보 및 `0ebdfc42`의 current-base merge-tree 일치를 확인해 fast pass했다. skipped worker를 새 Full CI 실행으로 표현하지 않는다.
- 기여자 측정: 동일 Linux/AMD EPYC, `RHWP_2424_PROFILE=1 rhwp info samples/issue2063_huge_cellbreak_table.hwp`. typeset 128,790→5,445ms, pagination 130,771→7,299ms, Node WASM open/pageCount 150.9→14.6초. 상대 개선 측정과 실행 명령이 있다. 검토자 재측정 수치가 아니며 다른 호스트의 절대 목표로 적용하지 않는다.
- 기여자의 1,225개 Native 동일 출력 및 17개 WASM 동일 출력 주장과 실제 재조회한 11개 문서 TSV를 구분한다. 전 코퍼스 실행 원본을 검토자가 독립 재실행한 것으로 쓰지 않는다.

## 검토자 focused 실행

정확한 head에서 `node scripts/rust-test-suite-manifest.mjs --prepare` 후 공용 `/home/edward/mygithub/rhwp/target/pr-review`를 사용했다.

- `node scripts/run-rust-test.mjs issue_6123_rowbreak_boundary_row_line_cut -- --cargo-profile release-test --target-dir /home/edward/mygithub/rhwp/target/pr-review --test-threads 2`: 1 passed. 저장 frame과 행 끝 불일치에서 줄 단위 컷과 앞뒤 조각 컷 연결을 확인한다.
- `node scripts/run-rust-test.mjs p122_stored_vpos_page_break -- --cargo-profile release-test --target-dir /home/edward/mygithub/rhwp/target/pr-review --test-threads 2`: 3 passed. 저장 vpos 되감김의 쪽 소속과 그림 배치를 확인한다.
- `pr7704-focused-height.log`, `pr7704-focused-vpos.log`에 원문 보존. 기존 검사를 실행했으며 새 baseline·fixture·테스트를 추가하거나 기대값을 변경하지 않았다. 이 검사 결과는 전체 시각 일치의 증거가 아니다.

## 검토자 성능 대표 문서 실행

정확한 PR head에서 `cargo build --locked --profile release-test --bin rhwp --target-dir /home/edward/mygithub/rhwp/target/pr-review` 성공 후 같은 fixture에 `RHWP_2424_PROFILE=1 /usr/bin/time ... rhwp info samples/issue2063_huge_cellbreak_table.hwp`를 실행했다. 총 9.88초, pagination 8,745.688ms, typeset 6,325.199ms, continuation 1,970.01ms, 161쪽이었다. `pr7704-cli-build.log`와 `pr7704-head-perf.log`에 보존했다. 검토자 base 실행은 없으므로 이 결과로 독립적인 전후 배수 개선을 주장하지 않는다. 한컴 PDF 쪽수 162는 제출자의 기록이며 실제 head의 161쪽과 구분한다.

## 제출 시각 증적 재검산

[원 TSV gist](https://gist.github.com/oksure/e4ad1f22e723bdf3dfb54001e9114369)의 current base/head Native·WASM 44개를 내려받아 재검산했다. 출력 경로별 11문서 393쪽. 11쌍씩 모든 base/head TSV가 바이트 단위로 동일하며 페이지별 수치에 누락을 숨기는 평균은 사용하지 않는다.

| 문서 | 쪽수 | Native/WASM 최저 | 각 경로 90% 미만 쪽 |
| --- | ---: | ---: | ---: |
| rowbreak HWP | 18 | 92.04216% | 0 |
| cell image HWP/HWPX | 각 8 | 96.74178% | 0 |
| table vpos HWP/HWPX | 각 5 | 98.86640% / 98.86666% | 0 |
| picture toggle HWP | 1 | 99.95151% | 0 |
| rowbreak HWPX | 18 | 31.59993% | 6 |
| giant cell overfill HWPX | 48 | 0.26231% | 36 |
| giant nested table HWP/HWPX | 각 115 | 23.48645% | 각 115 |
| scattered header HWP | 52 | 41.03657% | 41 |

각 경로 합계 313쪽 미달이다. 성능 대표 `issue2063`의 161쪽 대 PDF 162쪽은 기여자가 기록한 기존 차이이며, 11쌍 표에는 포함되지 않는다.

원본/PDF 22개 경로가 검토 head에 들어 있고 실제 파일과 commit 바이트가 같음을 확인했다. `pdfinfo`의 전체 쪽수와 제출 TSV 행 수가 일치한다. 한컴 Creator의 2020/2022/2024 표기는 기준 provenance 보조값이며 Print 생성 이력을 독립 재확인한 것으로 쓰지 않는다.

head에 고정된 Native/WASM rowbreak HWP 7쪽 review PNG를 실제로 열었다. 대표 쪽의 표 외곽, 행 경계 및 본문 순서에 큰 배치 차이는 보이지 않으며 글자 모양·굵기 차이가 있다. 원 제출 PNG 판독이며 검토자의 최신 통합 head 재출력 또는 작업지시자의 시각 통과 판정으로 쓰지 않는다. 대표 한 쪽의 92%로 다른 313쪽을 면제하지 않는다. 정식 font mismatch 예외 JSON도 제출되지 않았다.

증적은 `/home/edward/mygithub/rhwp/output/pr-review/oksure-20261011/`의 `pr7704-gist.json`, `pr7704-submitted-tsv/`, `pr7704-submitted-tsv-summary.json`, `pr7704-input-provenance.json`, `pr7704-preflight.log`, `pr7704-full-ci-jobs.json`에 보존했다.

## 해제 조건

작업지시자의 범위 확장 승인에 따라 메인테이너가 기존 차이의 원인을 보정한다. 수정된 값의 실제 소비 경로, 같은 원본/독립 Print의 전쪽 Native·fresh Docker WASM90% 및 변경 경계의 확대 판독을 확인한다. 최종 head의 focused·전체 회귀·세 Clippy와 정책 검사·Native Skia를 통과한 뒤 확대 보정의 원격 처리 승인을 요청한다. 이전 중간 코드의 통과를 최종 source에 그대로 적용하거나, 자동 점수 기준을 낮추지 않는다.

관련 #7639는 다른 병목과 편집 지연이 남으므로 이 두 순회 개선만으로 전체 종료를 주장하지 않는다. 현재 GitHub approve·merge·comment·close는 수행하지 않았다.


## 메인테이너 직접 시각 판정 준비

사용자 요청에 따라 같은 PR head `896e64ff6d876e676d6ebf987d6669b47653912a`의 Native를 공용 target에서 다시 빌드하고, Docker 표준 wasm 서비스로 새 web package를 만들었다. 코드·테스트·기여자 branch는 변경하지 않는다. 새 준비 증적은 기본 작업공간의 ignored `output/pr-review/oksure-20261011/pr7704-visual-review/`에 보존한다. 기존 gist TSV와 이번 검토자 재출력을 구분한다.

- 빌드: `cargo build --locked --profile release-test --bin rhwp --target-dir /home/edward/mygithub/rhwp/target/pr-review`; 바이너리를 이번 증적의 `candidate/rhwp`로 복사하고 SHA-256을 고정했다.
- WASM: `docker compose --project-name rhwp --env-file .env.docker run --rm wasm`; 최적화를 포함해 8m29s 성공. 새 pkg와 Studio public의 JS/WASM SHA-256 동일성 확인.
- 준비 범위: 제출된 11개 실제 원본과 대응 PDF, Native/Docker WASM 전쪽 각각 393쪽 TSV·PNG 및 분할 경계·미달 대표 review/overlay PNG. 빌드와 실행 로그, 명령·입력 경로·DPI·compat·binary/package 해시는 `provenance.json`, `logs/`, 경로별 sweep manifest에 남긴다.
- 독립 PDF마다 2024 또는 2022 compat을 지정한다. 설치/Studio 웹폰트 정책을 적용하며, 자동 점수를 글꼴 예외나 한컴 일치 최종 판정으로 바꾸지 않는다. 잘못된 font 경로를 지정하지 않는다.
- 검토용 Studio: `http://localhost:7794/?review=pr7704-896e64ff`. 기존 서버를 교체하지 않고 별도 worktree에서 실행한다. CDP Chrome 새 탭에 `rowbreak-problem-pages.hwp`를 실제 file input 경로로 열어 18쪽·Canvas 표시를 확인했다. 실제 응답 WASM hash `3f03e6ed3085b5e3411ac875d38feff565d6b4ad856992d78b1c4ac956df99ce`가 Docker 산출물과 일치한다(`studio-browser-proof.json`).
- 첫 판정: rowbreak HWP 7쪽 → HWPX 4·6·7·8·15·16쪽 → overfill·nested·scattered의 미달과 정상 대조군. HTML은 상대 이미지·원본·PDF 링크를 사용해 전체 폴더 복사 뒤에도 볼 수 있게 구성한다. 판정·완료·merge 승인을 추정하지 않으며 사용자 확인을 기다린다.

### 준비 완료 및 메인테이너 판정 (2026-10-11)

- 동일 head에서 Native와 Docker WASM 각각 11개 문서 전체 393쪽 TSV 및 PNG 산출 완료. 6개 문서는 전쪽 90% 이상, 5개 문서는 각 경로 313쪽 미달로 자동 gate 보류를 유지한다.
- 메인테이너 직접 판정: **rowbreak HWP 16쪽 정상 / rowbreak HWPX 16쪽 하단 결함**. 형식별 판정을 구분하며 HWP 통과를 HWPX 통과 근거로 쓰지 않는다.
- HWPX 16쪽은 하단 두 번째 표·그 아래 내용 영역에서 기준 PDF와 배치 차이가 남는다. 해당 쪽 실루엣 Native 78.46732%, WASM 78.45578%. 이 성능 PR로 새로 발생한 회귀라고 단정하지 않는다.
- 비교 화면: http://localhost:7795/index.html#doc=rowbreak-hwpx&page=16 . 전체 폴더의 상대 경로 HTML과 원본/PDF/TSV/PNG 링크를 제공한다.
- 증적: `/home/edward/mygithub/rhwp/output/pr-review/oksure-20261011/pr7704-visual-review/manual-verdicts.json`, `README.md`, `provenance.json`. PR 승인·병합·원격 게시 없음.
- 추가 확인: 메인테이너가 동일 문서 16쪽 하단의 HWP/HWPX 조판 불일치를 직접 확인했다. 파서 속성·IR·실제 조판 소비 경로의 원인은 아직 확정하지 않았다. 비교 HTML을 CDP Chrome 새 탭의 HWPX 16쪽에 열고 세 이미지 로드·겹침/배율 제어와 상대 이미지 경로 1,335개를 확인했다(`gallery-browser-proof.json`).

### 메인테이너 보정 착수

- 사용자 승인: HWP/HWPX 16쪽 불일치를 원인 분석 후 직접 보정하고 #7704 처리에 포함한다. 로컬 branch `review/oksure-pr7704-20261011` (기존 검토 branch에서 보정 연속 진행); 최신 `upstream/devel` `920dc29284e8a6e624861bfccf68903e19ae309b` 통합 SHA `973abfd5c` (충돌 없음). contributor 원 SHA와 분리한다.
- 원본 생성: 저장된 실제 HWP/HWPX 쌍과 독립 한컴 Print PDF를 그대로 사용, 메타데이터 수동 변경 없음. compat 2022/2024 교차 실행에도 HWP cut=11 / HWPX cut=12로 차이가 유지된다.
- 원인: `table_layout.rs::cell_units_uncached` → 같은 높이·문단/줄 소속을 가진 47유닛 → `advance_row_cut_inner`의 `entering_non_inline_control_range_end` 전체 개체 수용 검사 → 컷 → `table_partial.rs`의 shape 소유 control range 배치. 같은 유닛 목록에도 HWP 전용 gate가 HWPX에서 전체 수용 검사를 끈다. HWPX는 p5 글상자 filler 첫 16px를 소비해 현재 조각에서도 글상자를 배치·clip한다. Shape는 다음 조각에서도 소유 유닛에 따라 내부 내용을 이어 그리므로 Picture의 entry-only 정책과 구분한다.
- 위반 계약: 현재 조각에서 앞 내용이 이미 소비된 후 새 비인라인 개체를 시작할 때는 소유 range의 전체 요구 높이를 확인한다. 새 쪽에서 수용 가능한 글상자의 첫 filler만 현재 쪽에 남기지 않는다. 입력 형식은 이 계약을 바꾸지 않는다. 첫 유닛부터 시작한 fresh fragment와 본문보다 큰 개체의 진행 보장은 유지한다. 태그 없는 텍스트·TAC·CellBreak 표는 이 검사 비대상이다.
- 수정: HWP 전용 조건을 제거하고 RowBreak 비-TAC 및 실제 tagged control range로 검사. 픽셀 clamp·화면 숨김·문서 ID 조건 없음. 기존 helper/높이 결과를 그대로 소비한다.
- 선행 관측: 같은 first-fragment avail 286.2px에서 원 HWP cut11, HWPX cut12. HWPX 하단 글상자 entry가 현재 쪽에 들어가 PDF와 다름. logs `pr7704-maintainer/diagnosis/*units-old.log` (두 목록 동일), `*scan-old.log`. 수정 후 시각 gate와 영향 경계 확인 뒤 정식 회귀 추가 및 필수 lint/회귀를 진행한다.

### 두 번째 원인: 부동 개체 위 호스트 줄의 간격 소유

- `layout.rs::layout_table_item`에서 `host_line_fits_above_offset_float`가 참이면, TAC 간격을 더한 뒤 문단 원점으로 되돌렸다. HWPX는 문단 위 간격(raw 1000HU / 2 = 6.6667px)과 outer top(283HU, 3.7733px)을 잃었다. 정상 HWP는 확정 `InlineBoxPlacement` 원점과 advance_end가 이 간격을 보존했다.
- 진단 probe의 실제 소비값: HWPX `y=615.52, anchor=615.52, placement=None`; HWP `placement.y=527.6933`(단 상대), `advance_end=642.9067`. 같은 앞 문단 끝점에도 최종 TAC 괘선은 HWP 625.94 / HWPX 615.52. 독립 Print PDF의 두 형식 출력은 같은 위치다. ignored `diagnosis/*inline-origin.log`에 기록했고 probe는 제품 코드에서 제거했다.
- 보정은 기존 원점 복원을 표 위 간격 적용 앞으로 옮긴다. 부동 개체가 이미 전진시킨 흐름 하한은 기존 restore로 유지하고, 글줄의 spacing_before와 outer top은 한 번 적용한다. 신규 형식·문서 조건이나 위치 상수는 추가하지 않는다. 이후 `layout_table_control_block`의 확정 placement가 있으면 기존 공통 결과를 우선한다.
- 새 Native/fresh Docker WASM 시각 판독과 정상 대조군은 진행 중이다. 이전 head 결과를 보정 head의 통과로 사용하지 않는다.

### 16쪽 보정 결과와 메인테이너 판정

- Native 16쪽: 78.46732% → 98.18272%; fresh Docker WASM: 78.45578% → 98.16647%. 독립 한컴 Print와 16·17쪽 비교 PNG를 직접 대조했고, 메인테이너도 새 Studio에서 **시각 판정 통과**를 승인했다. 18쪽 전체 쪽수 유지.
- 수정 전 글상자 제목은 16·17쪽 트리에 모두 있었고, 수정 후에는 17쪽에만 한 번 있다. TAC 표 원점도 정상 HWP와 일치한다. `diagnosis/behavior-before-after.json`은 실행 전후 구조 관측이며 정식 신규 회귀 테스트를 대신하지 않는다.
- 정상 HWP 대조군 전18쪽 Native/fresh WASM 최저 92.08273%; 16쪽 Native 98.13736% / WASM 98.12108%.
- 실제 Studio `http://localhost:7794/?review=pr7704-maintainer-p16`에서 18쪽, 16쪽 Canvas 표시 및 응답 WASM SHA-256 `a3c92ddb91393b33d692555d6d3cf35654a12658bed9158af75a8a4c786663f2`가 최종 Docker/pkg/public과 일치한다. 진단용 임시 probe는 제거했다.
- 기존 경계 검사 #6133·#4468·#6045·#6123 각각1건 통과. fmt, native/WASM/workspace-all-target Clippy, workspace build 및 manifest 정책(base `920dc29284`) 통과. 전체 release-test 10,570건 통과(50건 skipped). Native Skia 라이브러리 및 #2225 누락 그림·37쪽 직접 PDF export 필수 3종도 모두 통과했다. 정확한 명령과 종료 코드는 `regression-results.json`에 보존한다.
- HWPX 전쪽은 4·6·7·8·15쪽에서 여전히 90% 미달이다. 사용자 판정은 보정된 16쪽에 적용한다. 다른 문서의 전체 게이트·기존 미달을 면제한 승인으로 해석하지 않는다. 새 정식 렌더링 회귀의 추가는 관련 전쪽 Native/fresh WASM 선행 조건이 충족될 때 진행한다. baseline·기존 기대값·허용치 변경 없음.
- 증적: 기본 작업공간 `output/pr-review/oksure-20261011/pr7704-maintainer/`의 `provenance.json`, `manual-verdicts.json`, `lint-focused-results.json`, `native-spacing/`, `sweeps/*/wasm/`, `native-spacing-review/`, `wasm-spacing-review/` 및 실행 로그. 비교 HTML `http://localhost:7796/`.

### 최종 코드의 전체 시각 재검증

- source `973abfd5c421e7ccef58f4d8954a25ec6faee992` + `diagnosis/maintainer.patch` SHA-256 `0ba8ab738991aa77c06a0084fbbfee10bcd18c8dc6ef66bfaf26f7b9945fe412` (로컬 보정 미커밋 상태). 전11문서 Native/fresh Docker WASM 각393쪽.
- 6문서 전쪽90%이상. 기존 미달5문서·312쪽: rowbreak HWPX 5쪽, overfill36쪽, nested HWP/HWPX 각115쪽, scattered41쪽. 모든 결과의 페이지 수는 기존 출력과 동일하다. rowbreak HWPX 16·17쪽 외의 수치는 원 PR 검토자 출력과 동일하다.
- `native-visual-summary.json`, `wasm-visual-summary.json`, `sweeps/*/{native,wasm}/*/silhouette.tsv`에서 전쪽 최저·미달 쪽과 원 PR head 대비 변화 값을 확인할 수 있다. 시각 부족을 성능 코드의 새 회귀로 바꾸어 보고하지 않는다.

### 실제 분할·이어받기 경로 확인

- 일반 행: `scan/runner/row_step.rs:444`에서 안여백을 뺀 내용 예산 → `:456`의 mixed-nested 예약 컷 → `table_layout.rs:20887`의 `advance_row_cut` → `:17015`의 소유 control range 높이 검사 → `row_step.rs:799`의 같은 컷 표시 높이 및 `:1215`의 수용 누적. 시작 컷은 16쪽 `[]`, 종료 컷 `[11]`; 17쪽 시작 `[11]`에서 나머지 유닛을 소비한다. tagged range 없는 줄, TAC/CellBreak, fresh fragment의 진행 분기는 이번 guard 확장의 비대상이다. 여러 rowspan이 함께 끝나는 block-cut 경로는 이 사례에 적용되지 않으며 이 수정으로 공통화했다고 주장하지 않는다.
- 실행: `diagnosis/final-path-scan.log`에서 16쪽 예산 286.2px, 내용 컷 263.0px·안여백 포함 수용 266.7px, 실제 프레임 281.1px 모두 예산 안에 있다. `final-next-path-scan.log`에서 17쪽 시작 `[11]`, 종료 `[]`, 예약/paint 920.7px·예산 933.5px를 확인했다. 기존 paint의 저장 프레임 보정은 별도 경로이며, 모든 물리 프레임 높이와 예약을 이 수정에서 통합했다고 주장하지 않는다.
- 실제 paint: `table_partial.rs:3252`의 Shape 소유·이어받기 분기는 같은 유닛 컷을 소비한다. 제목의 16쪽 제거·17쪽 단일 표시, 표 윤곽·다음 내용 및 18쪽 전체 쪽수는 전후 트리와 독립 PDF/Native/fresh WASM으로 확인했다. 작은 원래 높이만 fit하는 예산 등 추가 합성 경계는 미검증이며 신규 정식 회귀의 시각 선행 조건을 우회하지 않는다.
- TAC 원점: `layout.rs:13397`의 호스트 줄 복원 → 문단 위 간격/표 outer top → `:13565`의 table control 배치 → `:13591`의 앞 부동 개체 흐름 하한 복원. 확정 InlineBoxPlacement가 있는 정상 HWP는 그대로 우선하고, HWPX의 spacing 소유를 지우던 실행 순서만 고쳤다.

### 최신 원격 기준 확인

- 로컬 검증 종료 후 원격 `devel`은 `bfa28a77a2714482f647ee6a7b9aa7f7876d8404`였다. 검증 기준 `920dc29284`보다 266개 커밋 앞서며 조판 코드도 변경됐다. 기존 검증 결과를 최신 통합 코드의 통과로 바꾸어 보고하지 않는다.
- 최신 코드에도 HWP 전용 range gate와 spacing 적용 뒤 원점 복원이 남아 있는 것은 확인했다. 기존 검증 보정본을 먼저 보존하며, 최신 코드와 통합한 뒤 영향 경계·필수 검사·전쪽 시각 결과를 다시 확인해야 한다. 원격 push/PR 생성/approve/merge는 수행하지 않았다.

### 최신 기준 통합과 범위 확장 (2026-10-12)

- 16쪽 통과 보정은 `ce4189473`에 보존했다. 최신 devel `bfa28a77a2` 통합 head는 `1990ae5c1cf019b9d6cbf8553d87489243a2730a`. 충돌 한 곳은 새 셀별 예산·native 저장 프레임 변수를 유지하면서 HWPX에도 range 검사를 적용하도록 해결했다. 변경된 조판 코드를 포함하므로 이전 전체 회귀를 이 head의 결과로 재사용하지 않는다.
- 이 head의 관련 경계 4건, fmt, Native/WASM/all-target Clippy, workspace build, manifest 정책(base `bfa28a77a2`)은 통과했다. 전체 회귀는 추가 보정 범위가 승인되어 실행 중 중단했으며 최신 head의 완료 검사로 세지 않는다(`current-base/validation-stopped.json`). 기존 `ce4189473` 검증의 전회귀/Skia 통과 기록은 보존한다.
- Native 전11문서·393쪽 완료: 정상6문서 전쪽90%이상, 나머지5문서316쪽 미달. HWPX16쪽 98.30924% / 17쪽 99.82654%. 직접 review PNG에서 표 앞 간격과 17쪽 글상자 시작·윤곽·후속 내용을 다시 확인했다. 최신 devel의 다른 변경도 포함된 결과이므로 과거312쪽과의 차이를 이번 보정의 개선/회귀로 단정하지 않는다. 이 head의 고정 Docker WASM package 전쪽 실행은 계속한다.
- 사용자 범위 결정: **기존 시각 미달도 보정한 후 PR 처리**. 이번 PR에서 시각 게이트 예외는 적용하지 않는다. 순서: rowbreak HWPX 4·6·7·8·15쪽 → overfill HWPX → nested HWP/HWPX → scattered HWP. 원본/Print 및 전체 TSV를 보존하고, 원인 규칙별로 영향 경계·시각 출력을 먼저 고친 뒤 최종 source에서 필수 회귀를 다시 수행한다.
- 최신 증적은 `output/pr-review/oksure-20261011/pr7704-maintainer/current-base/`의 `provenance.json`, `native-summary.json`, `validation-results.json`, `native-review/`, `sweeps/` 및 실행 로그. 사용자 시각 판정은 `ce4189473`의16쪽 판정이며 최신 통합본 직접 판독과 구분한다.

### 기존 미달 보정의 중간 결과 (2026-10-12)

- 6쪽: 새 단의 첫 저장 줄이 앞 문단의 끝·줄간격과 자신의 위 간격을 정확하게 잇는 경우 그 source 원점과 위 간격을 하나의 `StoredColumnTopSpacing`으로 보존한다. typeset의 단 원점, 문단 paint, 뒤 표의 저장 host 원점이 이를 함께 소비한다. HWPX의 누적 좌표가 0에서 시작하지 않는다는 이유로 유효한 저장 표 원점을 제외하던 분기도 같은 인증 결과를 소비한다. 예전 단의 `vpos_ladder_dirty`만으로 새 단의 인증 결과를 폐기하지 않으며, 뒤 문단의 자연 저장 사다리 검사는 유지한다.
- 4쪽: 빈 host의 저장 줄이 앞 본문 끝과 자신의 위 간격을 잇고 선언 표 높이가 온전한 행 prefix를 수용할 때, 마지막 행의 source 물리 높이를 스캔·예약·paint에 함께 전달한다. 이 기존 인증 경로를 HWPX에도 적용해 앞쪽에서 이미 내용이 모두 소비된 행의 빈 잔여가 다음 쪽에 재조판되는 원인을 보정했다. 내용 소진만으로 물리 밴드를 임의 삭제하지 않는다.
- 15쪽: 실제 source reset에서 재개한 1×1 셀의 `cell.height − table.common.height + outer_top + outer_bottom`이 뒤 빈 문단의 저장 원점과 정확히 같은 경우 마지막 물리 상자를 인증한다. 그 원점에서 outer bottom을 제외한 표 하단을 예약·배치에 함께 전달해 그림 높이만으로 빈 closing band를 잃지 않는다.
- 중간 Native 전체18쪽 결과(`complete-frame/candidate/source.patch`, 같은 binary의 `native/rowbreak-hwpx/silhouette.tsv`): 4쪽96.07748%, 6쪽98.00485%, 15쪽97.57100%, 16쪽98.30924%. 7쪽89.22196%·8쪽64.96519% 미달이 남는다. 이는 확대 범위의 최종 통과가 아니며 fresh Docker WASM·최종 회귀·사용자 새 시각 판정은 아직 수행하지 않았다. 새 테스트·fixture·baseline·허용치 변경 없음.
- 7·8쪽: 저장 host 줄과 부동 중첩 표의 앵커·바깥여백 계획을 HWPX 측정/paint가 함께 쓰도록 보정했으나 마지막 child 줄을 7쪽에 과수용한다. scalar child viewport와 원래 source line cut의 차이를 계속 추적한다. 이 미달을 개선 완료로 기록하지 않는다.
- 원본 HWPX를 engine2020 한컴11.0.0.9136의 Print(`pdf_print_method=0`, `hancom2020_pdf_driver_one_up`)로 새 출력했다. job `b4dc5d22-9c46-44d5-ab8e-4021dba7f654`, 전체18쪽, 입력/PDF 해시·service 원문은 `print-refresh/provenance.json`, `status.json`에 보존했다. 7쪽의 마지막 줄 이월은 새 Print에서도 같으므로 기존 PDF의 과거 출력이라는 이유로 차이를 면제하지 않는다.

### RowBreak 7·8쪽의 source owner와 이어받기 여백 보정

- 실제 HWPX child는 1×1 RowBreak, control 없는 8개 저장 줄이 0부터 직전 줄 높이+줄간격을 정확히 잇는다. 마지막 ink 끝+셀 안여백이 선언 표 높이와 같아 독립 저장 프레임으로 인증된다. 이 결과를 기존 canonical 재귀 CellUnit 경로에 투영해 scalar viewport로 원래 줄 소속을 다시 추정하지 않는다. 편집/재조판·source reset·불일치 프레임은 인증 경로에 들어가지 않는다.
- `table_layout.rs::stored_plain_nested_frame` → `nested_table_mixed_fragment_heights`의 child owner 목록 → `mixed_nested_split_from_cut`의 실제 child RowCut → `mixed_nested_flow_extra_from_cut`의 물리 예약 → 실제 child paint와 뒤 parent 문단 흐름을 대조했다. 자식 마지막 줄의 줄간격은 뒤 parent owner가 같은 조각에 없을 때 경계 점유로 보존한다. 저장 float 계획이 이미 포함한 top padding을 legacy 분기에서 다시 빼지 않는다.
- 이어받는 child viewport의 outer top/bottom은 셀 padding과 다른 점유다. `plain_nested_continuation_outer_spacing`의 같은 결과를 실제 child flow와 부모 물리 예약 모두에 포함한다. 그림/괘선을 clamp하거나 뒤 문단만 이동시키는 보정은 없다. 7쪽의 마지막 child 줄은 8쪽에 단일 배치되고, 8쪽에서는 뒤 ③ 문단과 겹치지 않는다.
- `reopened-child/candidate/source.patch`와 고정 binary의 Native 전체18쪽: HWPX 최저90.53120%, 7쪽92.14573%, 8쪽96.92867%. 정상 HWP 전체18쪽 최저92.17336%이며 최신 base TSV와 페이지별 값이 모두 동일하다. 원본 독립 Print의 7·8쪽을 같은 배율로 직접 대조했다. 8쪽 첫 child 글줄의 작은 위치 차이는 남아 있으며 사용자 새 시각 판정은 아직 없다.
- 같은 source의 Docker WASM 빌드 진행 중. 이 중간 결과는 다른 미달 문서 또는 최종 head의 통과 판정이 아니다. 정식 신규 회귀/fixture/기대값 변경 및 원격 반영은 아직 없다.

- 추가 확대 판독에서 8쪽 첫 뒤 문단의 3.76px 차이를 검출했다. 부모 예약만 child 안여백을 포함하고 실제 flow는 내용 합을 쓰던 불일치였다. `plain_nested_continuation_frame_height`가 같은 child RowCut의 물리 높이+바깥여백을 반환하고, 부모 예약과 paint 후 흐름이 이를 함께 소비하도록 바꿨다. `physical-child/candidate/source.patch`의 Native 8쪽98.58706%; 이전 96.92867%를 최종값으로 사용하지 않는다.
- 검증 환경 확인: 실제 `/mnt/c/Windows/Fonts/H2MJSM.TTF`는 요청 face `HYSinMyeongJo-Medium`을 포함한다. 기존 font-style SVG의 Linux local() 탐색에는 그 디렉터리가 없었고, full 임베딩 탐색도 영문 이름과 기존 파일명 연결이 없었다. 로컬 fontconfig에 실제 디렉터리를 명시해 같은 원 face를 공급한 대조를 실행했다. 파일·face·해시는 ignored `fontconfig-provenance.json`에 보존하며 글꼴 파일을 공개·커밋하지 않는다. 글꼴 예외를 적용하지 않는다.
- 실제 글꼴 공급 상태의 `physical-child/native-fontconfig` HWPX18쪽 최저90.85014%, 정상 `native-hwp-fontconfig` HWP18쪽 최저93.62865%. overfill48쪽은 실제 글꼴 공급 후에도5~45쪽 미달이 유지된다. 첫 셀 저장 줄의 시작이 문단 위 간격(2000HU)과 정확히 같은데 기존 로컬 원점 조건이 이를 거부해 source reset을 잃는다.5쪽의 마지막 문장은 원래 줄 상자는 예산에 들어가지만, 이웃 프레임에 속할 마지막 줄간격까지 계상하면 밀려난다. 원점 인증→reset→예약 트림→실제 배치의 소비 경로를 보정할 계획이며 현재 미해결이다.

- `physical-child` 고정 source의 실제 Windows face 공급 상태에서 fresh Docker WASM 전체18쪽은 HWPX 최저90.85014%, HWP 최저93.62865%로 Native와 같은 범위를 통과했다. Docker WASM SHA-256 `fbd51d3ec7bf2aa45a27daab15db6ce392688adc6e0586d7226ca6067f294a2e`; 실제 Studio 초기화 응답 해시·18쪽 표시 증거는 `physical-child/studio-browser-proof.json`. 확대 페이지의 새 사용자 판정은 요청 중이며 다른 문서 또는 이후 source의 통과로 재사용하지 않는다.
- overfill의 첫 문단은2000HU 위 간격을 포함한 정상52.2667px 유닛이다. 이전 중간 진단에서 중복으로 추정한 첫 줄 높이는 실제 giant-cell owner 목록을 대조해 철회했다. 이어받는 양의 source 원점과 경계 줄간격의 인증은 본문 전체 높이에 도달했는지보다 실제 선택 컷·원본 reset·문단 before-space의 동일성으로 확인한다. 첫 조각이 앞 행 뒤에 시작하는 경계도 같은 결과를 소비하도록 보정 중이며, 아직 통과 판정은 없다.

### Overfill의 양의 source 원점과 물리 컷 간격

- `before-boundary/candidate/source.patch`와 고정 Native에서 원본500/1000/2000HU 문단 before-space와 실제 자연 저장 줄의 동일성을 사용한다. 내용 컷의 `page_frame_reset_before`와 같은 source owner를 읽어 첫 조각이 앞 행 뒤에서 시작하더라도 local zero origin을 인정한다. 일반적인 되감김을 전체 본문 높이 조건으로 새 쪽에 강제하는 정책과, 이미 선택한 컷의 원점·끝 줄간격을 보존하는 정책을 구분했다. 첫1·2·5쪽은98.75246/99.42223/99.03060%; 전체48쪽 통과 판정은 아직 없다.
- 6쪽 원본 마지막 문단116의 둘째 줄은 `vpos=73200, lh=1200, ls=720`; 다음 제목117은75620HU로 직전 slot 끝과 자신의500HU 위 간격을 정확히 잇는다. 한컴 Print6쪽에는116 문단의 마지막 줄까지 있고117 제목은7쪽에 있다. 선언 간격이 닫힌 progressing source boundary에서 실제 선택 조각의 마지막 줄간격은 다음 owner로 넘어가는 전진량이며 현재 ink box의 높이가 아니다. 빈 Enter·설명되지 않는 밴드·중첩 child·rowspan 경로는 이 트림을 받지 않는다.
- `closing-line`의 동일 컷 결과를 scan/`row_cut_content_height`/paint가 소비한다.6쪽 컷125→160에서125→161로 바뀌어116 마지막 줄을 수용하고 물리 예약999.5px가1009.1px 예산 안에 있다.7쪽 시작도161로 동일 owner를 이어받는다. 전체 시각 비교는 진행 중이며 신규 정식 검사·baseline 변경은 하지 않았다.

- `closing-line` Native 전체48쪽 결과:1~17쪽 전부90% 이상,18~45쪽28건 미달.6쪽99.11800%,7쪽99.85901%.5쪽은 양의 before 원점 인증 후99.03060%이다.18쪽 이후를 평균값으로 상쇄하지 않으며 Docker WASM/새 사용자 판정/최종 검사 전이다.
- 중첩 표의 최초 차이는3.7733px 바깥 위 여백이다. 원본 HWP의빈 폭0 host는500HU 양의 source 원점과 SectionDef/ColumnDef를 갖는다. 이 비가시 설정을 표 외 다른 개체로 세고 수직 원점이0인 경우만 앵커로 인정해 첫/이어받기 계획에서 outer top을 잃었다. 설정과 source 원점은 글줄 소유와 구분하며, 단일 자연 폭0 개체 앵커의 기존 바깥 프레임 결과를 prepare→budget→확정placement→paint가 한 번 소비하도록 보정했다. HWPX CellBreak의 동일 여백 계약도 형식별 gate에서 분리하지 않는다.
- `anchor-margin` 고정 Native의 중첩 HWP 대표1·2·7·8·9·115쪽:99.32118/99.75656/99.34377/99.03854/97.96304/99.79709%. 전체115쪽의 통과 주장은 아직 없으며 HWPX 전체 실행 중이다.18쪽 overfill은 TAC 단독 host 저장 줄의 높이가 실제 child 선언 높이+outer margins와 정확히 같은데 원점 인증에서 control이 있다는 이유로 제외된 별도 경로를 계속 보정한다.

- 실제 글꼴 공급 상태의 scattered HWP52쪽 Native(`anchor-margin/scattered-native`)는41쪽 미달을 유지한다.3쪽 직접 대조에서 PDF의 첫「인장시험」행이 rhwp3쪽에서 빠지고 다음 행부터 시작해 후속 윤곽·문단까지 차이가 누적됨을 확인했다. 글꼴 차이로 분류하지 않으며2→3쪽 소유/컷을 다음 단계에서 추적한다. 이번 source가 새로 만든 회귀로 단정하지 않고 frozen base와 같은 입력을 비교한다.
- `inline-frame` overfill 대표 실행은18쪽73.61529%,19쪽30.35858%,22쪽22.20404%,45쪽0.85582%로 미달이다. 단독 TAC host 줄 높이 인증을 추가했지만 실제 컷에서는18쪽에 다음 표의 일부가 여전히 배치된다. 원점 개선과 분할 해결을 구분한다. 재진단 probe는 ignored 로그에 실제 줄/child 높이·문단 간격·공통 원점 인증값을 남기도록 준비했으며 최종 source에서 제거한다.

- `frame-probe/frames.log` 원문에 실제 소비값을 보존했다. overfill giant-cell1274유닛의 body1009.147px, 초기local/before 인증 모두true.318 host는before6.667px·(vpos500,lh49542HU), child49260HU+outer141+141HU이고324 host는before6.667px·(500,13049HU), child13049HU+outer0으로 두 원점 인증 모두true. 두host의first unit도hard/stored-frame 모두true였다. 측정 이후 ordinary scanner가 인증에 다시 topology/형상 조건을 적용해 이 경계를 버리는 소비 지점을 확인했다. `strict_saved_frame_break`가 같은 인증 source cut을 소비하도록 수정 중이다. offset scanner는 이미 같은stored-frame flag를strict하게 소비한다. probe는 최종source에서 제거했다.


### 확대 보정의 source 프레임 경계 중간 검증

- `body-frame`은 저장 새 쪽의 첫 빈 줄이 앞 조각에 소비되는 원인을 보정했다. 자연 줄의 마지막 ink·셀 안여백은 본문에 들어가지만 뒤 줄 slot까지는 들어가지 않는 경우, 첫 빈 줄도 새 source frame의 owner로 취급한다. `cell_units` 생산 → 일반 scanner의 strict cut → `stored_frame_origin_for_cut` → partial paint가 같은 경계와 원점을 소비한다. 문단 내부 reset에도 같은 slot 계약을 적용한 `body-slot`에서 overfill37·38·39쪽은 모두90% 이상이다.
- `body-frame`의 nested HWPX 전체115쪽 결과는8쪽 미달(4·5·13·15·29·103·104·105)이다. 저장 empty host의 줄 끝이 명시한 양의 offset보다 위에서 끝나는 float 앵커를 공유한 `explicit-anchor` 대표20쪽에서는4쪽95.94606%로 통과하고7쪽 미달이 남았다. 선언 offset을 무조건 적용하면 기존 #3308의 좁은 직인 표에 회귀할 수 있으므로, 단 기준 가로 원점은 독립적으로 수용한 float 프레임의 `table_left`에만 연결했다. 무인증 offset의 셀 가운데 계약은 유지한다.
- `float-closing` 고정 source/binary에서5쪽99.58773%:4쪽 끝의 중첩 float occupied-bottom과 다음 빈 줄 slot을 함께 검사해 빈 줄을5쪽 첫 owner로 이월한다. source의 child3×3 host는65040HU, vertOffset1395HU, 선언7635HU, 바깥여백283HU이며, 다음 빈 줄은0부터 재개한다. 같은 float 계획을 생산·예약·paint가 소비하고 그림/글자를 숨기거나 clamp하지 않는다. 대표8쪽 중13·15·29·103·104·105는 여전히 미달이다.
- `float-closing/issue3308-tree`에서 직인 표 선언435.1px / 부모641.3px, 좌우 간격이 같아 가운데 배치 계약을 유지했다. 이는 해당 geometry 확인이며 원본 전쪽 시각 검증의 대체가 아니다.
- `declared-opening`은 direct HWPX의 첫 child 프레임을 선언 높이와 원본 ink 끝·안여백의 정확한 일치로 인증한다. overfill의 마지막 wrapper 첫 프레임은71020+1100+141+141=72402HU로 닫히며 다음 source 원점이 되감긴다. 이 source reset과 원래 child cursor를 canonical mixed unit에 보존하고, opening cut의 물리 높이를 `saved_nested_single_cell_frame_height`·`row_cut_content_height`·재귀 flow 예약·paint에 전달한다. 현재 실제 컷·후속 내용 검증을 계속하며 이 인증을 일반적인 단순 backward-vpos와 혼동하지 않는다.
- 고정 `declared-opening` Native 전체48쪽 결과:40개 페이지90% 이상,8개 페이지 미달(21=79.19352,36=40.57251,40=23.74784,41=29.42324,42=45.35085,43=39.17744,44=43.90253,45=30.93183). 전체 쪽수는49→48로 독립 Print와 같아졌지만 배치 완료가 아니다. Docker WASM은 아직 이전 `physical-child`까지이며 이 단계들을 fresh WASM 통과로 보고하지 않는다.
- 단계별 candidate/source.patch, full TSV, 원본 CLI cut 목록과 진단 로그는 ignored `pr7704-maintainer/` 아래 보존했다. 중간 compile 실패(존재하지 않는 enum 이름·private CellUnit 접근·helper 경로)는 환경/구현 작성 오류이며 원래 결함의 FAIL 재현으로 세지 않는다. 정식 테스트·fixture·golden·허용치 변경은 추가하지 않았다.

### 닫힌 행 프레임·인라인 원점 추가 진단 (2026-10-12)

- `float-closing/native-hwp`의 전체115쪽 결과도 HWPX와 같은6쪽(13·15·29·103·104·105) 미달이다. 두 형식의 실제 렌더러 경로를 공통으로 보정한다.
- overfill21쪽은 저장 vpos500+LH1100+상·하 안여백425×2=2450HU,13개 행합=표 선언31850HU로 정확히 닫힌다. 독립 PDF 괘선 간격도2450HU에 대응한다. 원래 fallback은 cellSz2334HU를 반복하고 잔여 높이를 마지막 행에 모았다. `stored_complete_row_frame_heights`가 자연 줄·미편집·행 소유·전체 선언 종점을 인증하며 HeightMeasurer와 paint의 행 높이를 같은 결과로 고정한다. 재조판·되감김·세로 병합·인증 실패는 기존 경로를 유지한다.
- 고정 `closed-row-frame/candidate/source.patch`와 binary의 Native21쪽79.19352→99.26207%, nested4쪽97.52791%. nested 나머지6쪽과 overfill36쪽은 이 단계에서 미달이다. 전체 Native/fresh Docker WASM 및 최종 회귀 증거를 대신하지 않는다.
- 인라인 표 fallback은 텍스트 전체 폭을 prefix로 쓰던 경로를 원래 저장 줄의 문자·개체 순서로 추적했다. nested13쪽은 첫6공백과 뒤2공백이 있으며, ordinary compositor의 `trailing_space_width_after_last_inline_object` 계약을 공유해야 가운데 정렬 폭과 cursor advance를 혼동하지 않는다. 초기 `ordered-inline-frame`은 이 말미 정렬 공백 계약이 빠져 여전히13쪽86.95155%였다. source와 실패 출력을 보존하고 수정 검증을 계속한다.
- overfill39쪽 실행 로그(`host-entry-frame/scan-probe.log`)는 예산999.720에 마지막 note 행까지 받아 consumed1103.213이 된 후 재시도 cut1173→1169로 줄인 것을 보여 준다. 따라서 공간에 들어가는6.6·6.7·6.8 행도40쪽으로 밀렸다. 뒤 spacer 유무가 초과 행의 수용 근거라는 `visible_tail_fits_before_spacer` 가정을 제거하고, 앞서 존재하던 독립 source reset·실제 closing spacing 인증만 유지하는 검증을 시작했다.

- `deferred-picture-frame` Native 대표7쪽에서13쪽의 말미 공백 정렬을 보정해90%를 통과했다.15쪽은42.62572→51.37297%의 부분 개선이며, 저장 원점이 앞 쪽에서 소비된 그림 띠의 실제 높이·바깥여백·캡션을 canonical 예약과 배치에 연결했다. 직접 review에서 그림 위치는 개선됐지만 뒤 내용 간격은 여전히 미달이다.
- `strict-row-capacity` 고정 source/binary의 Native overfill48쪽은21=99.26207,39=99.17561%,40=42.09464%; 미달은36·40~45의7쪽이다. 뒤 빈 줄의 존재를 예산 초과 수용 근거로 삼던 두 scanner grace 호출을 제거했으며, 실제 컷으로6.6~6.8행이39쪽에 남고 note 행이40쪽으로 이월된다. rowbreak HWPX 전체18쪽 최저90.85014%로 무회귀이며 nested 대표15·103~105는 이 단계에서 여전히 미달이다.
- 중첩 문서29쪽의 non-TAC Para 기준 표는 문단 왼여백1000HU/912HU가 실제 가로 원점에서 빠졌다. depth>0 Para 좌표와 일반·분할 caller의 style 여백을 연결하며, 첫 줄 hanging indent를 표 원점에 섞지 않는다. 무인증 Column 오프셋의 #3308 가운데 배치 계약은 유지한다. `paragraph-object-frame`에서 검증 중이다.
- 중첩 문서103쪽 child14×2의 r9는 왼셀 두 문단과 오른셀 한 문단의 서로 다른 두 text range가 각각0→0에서 이어진다. 자연 줄·미편집·동일 column/width·rowspan1·모든 셀의 공동2-slot 경계 및 원래 행 상자합이 선언 object frame보다 큰 조건을 함께 대조한다. 문단-local zero만으로 페이지 경계로 승격하지 않으며 원본 지점과 실제 cut의 연결을 검증 중이다.
- `paragraph-object-frame` Native29쪽은 가로 원점 보정만으로78.97230→82.91362%였고, `joint-row-projection`에서 온전한 자식 표의 분할 metadata를 실제 내용 컷과 구분해 바깥여백의 중복 전진을 제거한 뒤29쪽90%를 통과했다. metadata 존재를 부분 표시로 대신하지 않는다.
- 공동 zero-origin 경계는 cell clone의 pointer 동일성 대신 해당 행의 열 소유로 연결했다. canonical 자식 행 투영에서도 실제 source boundary가 있는 행의 두 유닛을 분리했다. `joint-row-projection` Native104쪽34.27017→75.89734%의 부분 개선,103=75.41435,105=19.35457%로 미달이며 분리된 내용과 남은 물리 밴드를 계속 대조한다.
- overfill36쪽은 p607 TAC 표의 저장 줄18101HU 뒤 p608의 원점20321HU에720HU 줄간격과1500HU 문단 앞 간격이 이미 포함된다. 일반 multi-row 부모의 post-table 스냅만 이를 빼지 않아20px 중복됐다. 독립 saved-boundary 인증을 사용해 뺄 앞 간격을 결정한다. nested15쪽도 재배치한 picture band의 canonical 높이와 paint 후속 전진이 달랐으므로 같은 높이를 실제 흐름에 소비하도록 보정 중이다.

- `shared-successor-flow` Native 대표 재출력: overfill36=97.51299,39=99.17561%; nested15=95.44317,29=99.31356%. 남은 nested103–105 및 overfill40–45는 미달이며 최종 full Native/fresh Docker WASM 증거가 아니다.
- nested104의 두 번째 TAC 그림은 자연 저장 LH21707HU와 음수 LS−20317HU로 후속 빈 줄이 그림 띠 안에 이어진다. 실제 개체 높이와 문단 전진1390HU를 구분하며, 앞 source 프레임 종점64170HU가 본문에 들어가고 다음 그림 높이는 들어가지 않는 되감김을 페이지 경계로 처리한다. 음수 LS·빈 문단을 삭제하거나 합산 높이로 중복하지 않는다.
- nested103의 공동 source reset 행은 선언 object opening height33150HU와 앞9행29880HU의 차이3270HU를 물리 밴드로 소유한다. canonical cut 수용 높이와 실제 마지막 행 높이 override가 같은 source 인증 결과를 소비하도록 보정 검증 중이다. 뒤 내용의 유닛 컷과 빈 밴드는 별개이며 재조판·비공동 local zero에는 적용하지 않는다.

- `joint-opening-picture-frame`의 공동 opening 밴드 적용으로 nested104쪽은91.14637%다. `saved-successor-bands`에서는 자연 저장 empty/successor 원점의 간격과 음수 LS 그림 문단의 실제 전진을 공통 소비했다. nested103=98.24209,104=91.14637,105=86.27969%이며105의 가로 그림 원점은 여전히 미달이다. overfill41=98.39160%: 선언 opening 내용 높이의 세로 정렬 후 source before를 다시 더하던 경로를 제거했다. rowbreak HWPX 대표4·6·7·8·15·16·17쪽은 모두90% 이상이다. 이 값은 Native 선택 쪽 진단이며 전체/fresh WASM 결과가 아니다.
- `coincident-empty-frame`은 overfill40쪽의 첫 빈 문단과 뒤 비고 문단이 source vpos0을 함께 소유하고, 다음 문단부터1600HU씩 이어지는 원본을 대조한다. 선언 셀11450HU와 마지막 저장 ink+활성 padding9850HU의 차이1600HU는 첫 빈 문단 advance와 같다. 빈 문단을 삭제하지 않고 같은 저장 슬롯 소유와 닫힌 물리 프레임을 canonical seed·측정·전체 셀 배치에서 공유했다. Native40=99.47704,41=98.39160%;42–45는 미달이다. source.patch·binary·원본 review/overlay는 기본 작업공간 `pr7704-maintainer/coincident-empty-frame/`에 고정했다.
- overfill42의 짧은 행은 줄과 활성 안여백의 요구 높이를 행 수20 이하에서만 반영하는 기존 분기로 축소됐다. PDF 괘선의 짧은 행 높이23.494/22.216px와 source LH+활성 padding을 대조했으며, 행 수 조건을 일반 조판 근거로 수용하지 않는다. 측정과 실제 배치 양쪽의 소비 경로를 수정하고 정상 대조군과 함께 재검증할 예정이다.

- `shared-row-padding`의 전체 Native393쪽에서 rowbreak 두 형식36쪽은 전부 통과했다. overfill48쪽은42~45쪽4건, 중첩 두 형식230쪽은각105쪽1건, scattered52쪽은41건 미달이다. 나머지5문서27쪽은 통과했다. 평균값으로 미달을 상쇄하지 않는다.
- `stored-horizontal-frame`은 저장 줄의 column_start+segment_width+문단 right가 실제 셀 inner width와1HU 안에서 닫히는 경우 그 가로 영역을 generic inline pen에 사용한다. nested105의 그림 x는 PDF267.056px와 Native267.18px이며 Native105=99.67902%,13=99.18042%,104=91.14637%다. HWPX overfill의 정상 저장 grid는 행별 셀 폭과 cell spacing이 선언 폭에서 닫히므로 부모 안쪽 폭으로 임의 축소하지 않는다.42=87.96151%,43~45는 미달이다. 후속 `saved-host-fragment-margins`에서 colspan 내부 spacing의 수용 산술을 보정했다.
- `saved-host-fragment-margins`는 폭이 유효한 빈 host의 자연 저장 ink가 명시적인 Para 상대 표 offset보다 앞에서 끝난다는 기존 frame 인증을 사용한다. 같은 top/bottom outer band를 continuation 예산과 실제 원점에 반영했다. scattered Native1=95.55121%,2=95.34085%,3=99.25233%,4=68.51623%.4쪽은 앞쪽 셀의 `바다모래인 경우` 두 줄이 동일 가로 영역에서0→0으로 저장됐는데 현재 row cut이 둘째 줄을 앞쪽에 함께 소비한다. 전체52쪽 재측정 중이며 종료 판정이 아니다.
- 중첩 repeated-header continuation은 atomic child row의 scalar viewport가 머리행 소유를 표현하지 못한다. `nested-repeat-reservation`은 명시적 child RowCut을 전달하고, 동일 child row metrics의 leading header band를 row/block scanner 예산·cut 요구 높이·recursive child paint parent 높이에 공유하는 보정 후보다. 독립 Print43~45쪽의 반복 머리행을 근거로 선택했으며 현재 빌드/영향 쪽 검증 중이다. 새로운 정식 테스트·fixture·baseline 변경은 아직 하지 않았다.

- `nested-repeat-reservation` Native 선택 쪽에서 머리행 반복은 복구됐지만 overfill42=88.62588,43=47.34337,44=36.75523,45=28.87441%,46쪽의 쪽 소속 차이가 남는다. 원본13행6열 rowspan3 칸은 source 첫 frame ink7400HU+둘째 frame ink10600HU+두 frame의 top/bottom padding(각283HU)=cellSz19132HU로 정확히 닫힌다. 현재 rowspan 높이 기반 컷은 마지막 줄의 line-spacing600HU까지 요구해 마지막 `-접지전압` 줄을43쪽에 중복 배치한다. 선언이 닫히는 이 프레임 경계의 줄 전진과 글줄 상자를 구분해 앞·뒤 straddle 컷 및 실제 cut 요구 높이에 같은 결과를 적용하는 보정 후보를 검증한다.
- `saved-host-fragment-margins` Native52쪽은 미달34건으로 줄었으나4쪽과15쪽 이후 경계가 남아 있다. `plain-line-frame-owner`는 원본 plain-text의 같은 문단·같은 가로 영역·서로 다른 text_start가 vpos0을 다시 소유하는 경계를 CellUnit, row scanner와 paint의 컷 계약으로 전달한다. 옆 가로 조각·개체 guide·편집/재조판에는 적용하지 않는다. 미달 쪽 검증이 진행 중이며 최종 승인 결과가 아니다.


### 저장 줄 경계 및 바깥여백의 후속 보정 (AT–AV)

- `nested-source-row-cursors/candidate/source.patch`와 Native 원문은 같은 문단의 자연 저장 줄이 동일 가로 영역에서 증가한 textpos와 vpos=0을 가지는 실제 쪽 경계를 child RowCut으로 투영한다. 반복 제목행과 바깥여백도 부모 예약과 재귀 paint가 함께 소비한다. overfill 43쪽97.85494%,45쪽99.854%,46쪽99.994%; 42쪽88.564%,44쪽78.889%는 미충족이다. nested HWP/HWPX 대표13·103·104·105쪽은 각각99.180/98.242/91.146/99.679%다. 이는 전체115쪽 또는 fresh WASM 판정이 아니다.
- `single-opening-margin/candidate/source.patch`: `partial_rowbreak_fragment_spacing_px`가 이미 예약한 원본 표의 위 바깥여백을 `prepare_table_fragment_budget`의 후속 opening 분기가 다시 차감했다. paint는 `repeat_fragment_outer_margin`으로 재개방을 제외하고 있어 예산과 실제 원점이 달랐다. 같은 predicate를 예산 후속 분기에도 적용해 한 번만 소비한다. scattered 대표4쪽98.096→99.371%,6쪽56.927→93.021%;5·7·15·16·17쪽은 미달이 남았다. 전체52쪽 재측정 실행 중이다.
- overfill44쪽 원본 child 행24의 셀은 Center 정렬, 선언43992HU, 첫 저장 프레임 마지막 ink41960HU와 유효 안여백566HU를 가진다. 한컴 Print의 행 상자491.941→1068.271px에 비해 AT Native는490.7→1057.7px로 짧으며 첫 문단과 뒤 문단 모두 차이가 있다. 내용 컷이 물리 blank band를 소비하지 않는 점을 source minimum과 실제 페이지 예산으로 분리해 예약·child paint·정렬에 공유하는 AV를 검증한다. 숫자나 문서 ID로 분기하지 않으며 아직 성공 판정하지 않았다.
- 신규 정식 회귀·fixture/golden/baseline은 전체 관련 Native/fresh WASM 90% 및 직접 판독 전 추가하지 않는다. 최신 전체 lint/회귀/Docker WASM/시각 gate 완료 전이며 원격 반영·PR 생성·merge 준비 완료가 아니다.

- AU scattered 전체52쪽은 쪽수52/52, 미달30쪽이다. 미달 목록은 `single-opening-margin-scattered-full/sweeps/scattered-hwp/native/scattered-hwp/silhouette_manifest.json` 원문에 보존했다.
- AV 대표4·5·6·7 및42·43·44·45쪽은 AU와 같은 수치로 물리 band 확대가 아직 실제 경로에 적용되지 않았다. 원본 overfill의26×9 child는5×1 →1×1 →26×9 두 깊이 wrapper 안에 있으며, 최상위 scanner는 nested-row atom이 아니라 mixed recursive source cursor를 선택한다. 깊이 하나에서만 fragment를 찾던 AV producer를 실제 재귀 컷 경로에 연결해야 한다. 미검증 코드의 성공으로 보고하지 않는다.


### 이어받기 유닛과 빈 영역의 소유 보정 (AW–BA)

- `spanning-line-owner`는 원본 rowspan의 높이 기반 컷에도 마지막 줄의 ink와 successor advance를 구분하고, 같은 문단의 자연 저장 0→0 경계를 이어받기 컷에 반영한다. 앞쪽에서 이미 소비한 줄의 반복과 후속 행의 연쇄 증가를 제거했다. Native scattered 4=99.99061%,5=96.28458%,6=94.00777%;7·15·16·17은 미달이다. 새 전체52쪽/fresh WASM 결과는 아니다.
- `stored-span-frame-alignment`는 선언 높이로 닫히는 첫 저장 rowspan frame과 명시적인 다음 source frame에서 모든 남은 내용을 받는 continuation에 원본 valign을 보존한다. 같은 결과로 compose window와 최종 정렬 원점을 결정한다. scattered 5쪽97.03806%; PDF의 이어받은 시멘트 라벨 위치를 직접 비교했다. 기존 컷과 관계없이 모든 rowspan에 Center를 강제하지 않는다.
- `recursive-source-blank-band`는 5×1→1×1→26×9 실제 recursive cursor로 자식의 closing blank band를 투영했다. 그러나 wrapper가 그 밴드를 자신의 Center 정렬에 사용해 전체 표를 내려 overfill42=64.13350%,44=64.74759%로 악화됐다. 이 실행 결과는 실패이며 source.patch·PNG를 보존했다.
- `descendant-band-owner`는 같은 컷의 자식 소유 밴드를 wrapper의 정렬 여유와 분리했다. overfill42=88.56376%,43=97.85519%,44=78.59683%,45=99.85368%. 회귀 방향은 제거됐으나42·44의 물리 상자와 미세 원점 차이는 남았다. 최종 승인·완료가 아니다.
- scattered 7쪽 앞부분의 원본 r140 c6은 두 줄의 마지막 ink2300HU+활성 padding566HU=선언2866HU로 닫힌다. PDF 행145.601→183.799px와 Native145.9→188.6px가 다르다. 온전한 저장 셀에서 마지막 LS300HU까지 최소 높이에 더해 행이 커지는 실제 소비 경로를 확인했다. BA는 자연 progressing 원본 프레임의 ink+padding 폐합을 공통 producer로 확인한 뒤 측정·row resolve·CellUnit 끝 advance가 함께 소비하도록 보정한다. 빈 Enter와 설명되지 않는 물리 밴드를 삭제하지 않으며, 편집/재조판·vpos reset·개체 포함 셀에는 이 폐합을 적용하지 않는다. 빌드와 영향 쪽 검증 진행 중이다.

- `closed-cell-ink-owner`(BA) Native overfill42·43·44·45와 scattered7·15·16·17은 AZ와 동일했다. scattered6쪽은94.44050%로 일부 개선됐지만 위반된 경계를 모두 해소하지 못했다. 선언 폐합 helper의 존재만으로 실제 호출 경로가 해결됐다고 판정하지 않는다.
- BB에서 발견한 경로: `continuation.straddle_page_bands`는 이미 앞 쪽 끝의 빈 공간을 셀별로 누적하지만, `rowbreak_straddle_cut_units`는 그 원장을 받지 않고 이전 행의 원래 content height로 소비량을 다시 추정한다. 실제 PDF6쪽 r138은58.496px이며 Native는42.0px이고, 그 차이로 앞쪽의 세 번째 글줄이 다음 쪽에서 다시 표시됐다. 이어받기 demand와 최종 paint 모두 같은 cell-index 원장을 받아 앞쪽 물리 공간을 계상하도록 연결했다. 원장을 PageItem에 보존하고 cursor query·endnote clone에도 전달했다.
- BB는 행 경계에서 끝나는 원본 RowBreak의 걸침 셀이 본문 끝의 남은 공간을 소유할 때, 원래 scan 높이와 그 빈 band를 함께 예약하고 마지막 행 override로 paint에 전달한다. 이미 내용 컷이 있는 조각·block cut·각주 예약·편집/재조판에는 새 행 경계 확장을 적용하지 않는다. 빈 band를 다음 쪽에서 다시 소비하지 않도록 실제 수용량을 기존 continuation 원장에 기록한다. `cargo check --locked --lib --target-dir /home/edward/mygithub/rhwp/target/pr-review` 통과; 최종 build·경계 시각 검증 진행 중이다. 이 check는 필수 Clippy/full regression/fresh WASM 검증의 대체가 아니다.


### 누적 물리 공간과 원본 경계의 재검증 (BB–BF)

- `shared-straddle-physical-ledger` Native scattered6=99.99083%,7=99.80694%,5=98.12947%로 개선됐다. 원본 행·다음 쪽의 줄 순서와 외곽을 직접 비교했다. 전체52쪽과 fresh WASM은 아직 실행하지 않았으며 다른 미달을 면제하지 않는다. source-side tier 정책(base `bfa28a77a2`)도 통과했다.
- 대조군 rowbreak HWPX8쪽은43.33145%로 회귀했다. frozen binary별 같은 원본8쪽 트리 대조에서 `resumed-line-frame-owner`까지883.5px 표 조각, `nested-source-row-cursors`부터732.5px였다. 따라서 원인은 BB의 물리 공간 원장이 아니라 AT에서 양수→0 reset까지 무조건 source cut으로 확대한 조건이다. 앞선 원인 추정은 이 실행 결과로 정정한다(`bisect-rowbreak8/`).
- `recursive-cursor-reservation`(BC)의 재귀 reservation 일반화 및 BD의 정렬 조건 제거만으로는 overfill42·44/rowbreak8의 차이가 해소되지 않았다. 같은 helper를 부르는 것만으로 소비 계약이 성립했다고 판정하지 않는다.
- BE에서 선언된 **전체 셀 높이**를 모든 첫 source cut의 최소 높이로 사용하는 가정을 제거했다. source declaration이 선택한 첫 프레임을 닫는지 별도 입증이 필요하다. overfill24행의 가시 셀은 첫 프레임 마지막 vpos40500+ink1460+LS1466+활성 padding566=선언43992HU로 닫힌다. rowbreak8쪽 가시 셀의 reset 직전 값은28160+1100+660+566=30486HU로 선언46376HU와 다르다. BF는 이 독립 폐합 또는 원래0→0 경계를 요구하며 임의 pixel 허용치를 추가하지 않는다. 최종 검증 진행 중이다.
- scattered 두 번째183행 표의 다음 문단 vpos3414HU는 반복 머리글1566+마지막행1566+바깥 위141+아래141과 정확히 같다. host의36066HU→3414HU rewind는 마지막 행의 새 쪽 소속을 입증한다. source→준비 상태→scan 컷→PageItem→paint로 같은 행 소속을 전달하는 보정을 진행한다. 첫 BE 출력은 적용되지 않아 원인을 재검토 중이며, 구현 존재를 통과로 보고하지 않는다.
- 이전 p16/대표 쪽 통과 기록은 보존하되 확대 코드의 전쪽 gate, 필수 lint/회귀/Skia, fresh Docker WASM의 최종 통과로 재사용하지 않는다. 원격 게시·push·PR 생성·merge는 아직 수행하지 않았다.


### 내용 컷 후보와 쪽 소유의 구분 (BG–BH)

- BG의 source suffix producer는 앞쪽의 rowspan-only 행을 이유로 무관한 뒤 suffix까지 거절하던 가정을 제거했다. 소유를 인증하는 반복 머리글과 실제 suffix 행의 선언 높이만 합산하며, 관련 행의 값이 없으면 인증하지 않는다. scattered14=99.99854%,15=99.99482%,16=99.76910%,17=99.99613%;7쪽99.80694% 유지. 전체52쪽 산출 진행 중이며 fresh WASM 판정은 아니다.
- BF rowbreak8쪽은97.67675%로 회귀가 해소됐지만, overfill44쪽은44.05551%로 미달이 증가했다. 구현을 성공으로 확정하지 않는다. 원시 XML 재검증에서 overfill의 reset 직전 줄은40960+LH1000+LS600+padding566=43126HU이며, 위 문단에서 제시한1466HU 마지막 간격에 의한43992HU 폐합 추정은 확인되지 않았다. 진단 근거를 정정한다.
- 자연 저장 줄의 같은 가로 영역 내 재시작은 **분할 가능한 내용 컷 후보**를 제공한다. 실제 쪽 소속을 강제하는 인증과는 다르다. BH는 nested-row 유닛의 세분화와 page-boundary 인증을 분리한다. 후보를 없애서 전체 행을 원자화하거나 후보가 있다는 이유만으로 쪽을 강제하지 않는다. 실제 scan budget이 선택한 같은 child cut을 paint에 전달한다.
- 실제 overfill42쪽 probe: root에서 child1×1은content981.947/extra12.920px인데 `split.visible_height`는1010.267px여서 spare가0이었다. 다음 child26×9의 visible978.187px도 실제 행 조각의 상자 합과 다르다. 부모 투영 viewport를 자식 물리 높이로 대체 사용한 것이 band 전달을 막는다. BH는 같은 `nested_cut_physical_height`의 선택 행·컷 높이 합을 실제 child 여유 계산에 사용한다. probe 원문은 `source-suffix-and-frame/overfill-render.log`; 진단 probe는 최종 제출 전 제거한다. 빌드·영향 경계 검증 중이다.


### 일반 셀 컷과 반복 머리글 예약의 실제 경계 (BJ–BK)

- BG scattered 전체52쪽은20쪽 미달이었다(`source-suffix-and-frame/sweeps/scattered-hwp/native/scattered-hwp/silhouette_manifest.json`). BJ는 RowCut의 슬롯이 일반 셀만 열순으로 열거한다는 기존 계약을 producer에도 적용했다. 같은 행에서 시작하는 rowspan 셀은 별도 이어받기 원장을 소유하므로 일반 셀의 유효한 컷을 거절할 근거가 아니다. 기존25px 최소 조각 조건의 예외도 actual source cut 일치와 실제 예산 검사를 유지한다. BJ Native19=99.99672%,20=99.59611%,21=97.99206%;22·26·39·48·51은 아직 미달이다. 20쪽 대표 PNG에서 첫 이어받은 `(ISO3633)`와 `KS M ISO2507-2`, 후속 표 내용·외곽을 대조했다.
- BH/BJ의 overfill42=98.92338%,43=94.80874%는 actual child cut의 물리 상자와 blank band 공유로 개선됐다.44·45쪽 미달의 실제 원인은 `nested_atomic_continuation_header_height`가 atomic 완전 행에만 머리글을 예약하는 데 있었다. 한컴 Print와 child paint는 행 내부 컷으로 시작해도 머리행을 반복한다. BK의 `nested_continuation_header_height`는 body의 start/end 컷·terminal 여부와 물리 머리행을 구분한다. CellUnit→row/block budget→cut height→nested viewport에 같은 leading header 높이를 전달한다.
- BK Native overfill42~46쪽은98.92338/94.80874/97.93569/99.85368/99.99365%이며 rowbreak HWPX8=97.67675%,16=98.30849%를 유지했다.44쪽의 반복 머리행, 3.4~3.6 행의 줄 소속·외곽을 대표 review PNG로 직접 대조했다. 미세 간격 확대와 전쪽 Native/fresh Docker WASM, 필수 lint·전체 회귀는 최종 head에서 추가 확인해야 한다. frozen source/binary는 `partial-row-header-reservation/`에 보존했다.
- BJ22쪽은 원본 행237의 첫 문단 `압축강도`를21쪽에 수용하지 않아 전체22쪽이 한 줄 낮았다. capacity cut `[1,1]`, 남은23.493px에 대해 closing unit이17.333px(ink13.333+LS4.0), padding7.547px를 요구했다. 두 plain 문단의 원본 ink와 중간 advance·active padding이 선언2866HU를 닫는다. BL은 이 유효한 complete cell flow를 근거로 이미 budget이 선택한 문단 경계의 마지막 advance를 후속 owner와 구분한다. 로컬vpos0만으로 page 소속을 강제하지 않으며 개체·빈 문단·dirty/reflow·닫히지 않는 물리 band에는 적용하지 않는다. 빌드·영향 쪽 재검증 진행 중이다.


- BL Native scattered21=100.0%,22=97.47495%이며 rowbreak HWPX8/16은97.67675/98.30849%를 유지했다. 기존25px 정책의 우회는 유효한 complete cell flow와 **이미 capacity-selected cut**에서만 적용하고, actual split_total 예산은 유지한다.26·39·48·51쪽 미달은 남아 있다(`closed-paragraph-capacity-cut/`). BL 고정 source의11문서/393쪽 Native 재검증을 시작했다.
- BM 진단: scattered25쪽은 PDF 마지막「신도」행 뒤 빈 band를 남기고「톨루엔가용분」을26쪽으로 이어받지만 rhwp는42행까지25쪽에 소비한다. 실제 `saved_first_fragment_source_frame=(969.08px,989.88px)`, content budget972.3px, 근접 추정 `source_end=43`이고43행 수용량970.787px다.26쪽부터 한 행 차이가 누적된다. source frame보다 실제 수용량이 커도 근접 행 추정·overflow waiver가 그대로 적용된 것이다. BM은 현재 flow와 일치하고 본문 안에 실제 들어가는 원본 source frame·완전한 저장 grid·미편집 자연 줄을 인증한 경로에서 그 높이를 실제 첫 조각 행 예산으로 소비한다. 빈 공간은 straddle 소유 원장에 보존하고 다음 행을 앞쪽에 당기는 허용치로 쓰지 않는다. 끊긴 grid·dirty/reflow·본문 밖 선언에는 적용하지 않는다. 빌드·원본 출력 확인 중이며 완료 판정이 아니다.


- BM은 실제 scan에서 pi29의 첫 수용 컷을0..43→0..42, pi53을0..16→0..15로 바꿨다. 뒤쪽 clipping만 숨기지 않고 다음 조각의 시작 owner도 같은 컷으로 바뀌었다. Native25=99.98714%,26=99.98719%,38=99.77711%;21/22는100.0/97.47495% 유지. 전체 Native/fresh WASM 및 확대 판독은 아직 최종 증거가 아니다.
- BN의 반례: 원본 pi59는 완전한238행×13열 grid, 모든238행의 셀 폭 합48008HU, 모든 셀에 양의 자연 LineSeg가 있다. 그런데 `stored_row_boxes_exceed_object_frame`는 일반 셀이 없는 rowspan-only 행의 `None` 때문에 알려진 다른 행들의 합까지 폐기했다. 이 함수의 질문은 전체 높이를 정확히 재구성하는 것이 아니라, 전체 표가 선언된 object frame을 초과하는지 확인하는 것이다. 알려진 ordinary row boxes의 합은 full table의 하한이며 그 하한만으로 선언을 초과하면 충분하다. 무관한 unknown 행을0높이라고 그리거나 컷을 추정하지 않는다. 실제 분할 높이·continuation 소유는 기존 row/block·straddle 조회가 계속 결정한다. 빌드·영향 쪽 검증 진행 중이다.


- BL 고정 후보의 전쪽 Native에서 rowbreak HWP18쪽 최저93.62865%, HWPX18쪽 최저90.85014%, overfill HWPX48쪽 최저93.03871%로 각 전쪽90%를 충족했다. BM 뒤 scattered39쪽도99.21114%이며 아직393쪽 전체·fresh Docker WASM 최종 통과가 아니다.
- BN은 pi59의 source frame 제한을 실제 예산334.3→331.1px로 연결했다. 그러나47→48쪽의 행99 첫 줄이 여전히 기각됐다. scan은19.120px content budget에서13.333px의 컷`[1]`을 이미 선택했는데 `stored_plain_restart_cut`의「완결된 일반 형제 셀도 있어야 한다」는 가정 때문에 source cut을 거절했다. 이 행에는 일반 셀이 하나뿐이며 양옆은 별도 continuation 소유의 rowspan 셀이다. 독립 Print47쪽은 첫 줄,48쪽은 다음「뮴)」줄부터 소유한다. BO는 일반 셀 모두가 이어지는 경우와 일반 셀이 하나인 경우에도 동일한 자연 저장 컷을 수용하도록 잘못된 completed-sibling 가정을 제거한다. 실제 높이 예산과 rowspan 소유·빈 band 계약은 유지한다. frozen BN 진단은 `known-row-height-lower-bound/scattered-scan.log`; BO 빌드·실제 출력 검증 중이다.


- BL 전쪽 실행 driver는3개 문서(84쪽) 완료 후exit143으로 중단됐다. nested HWP/HWPX 전115쪽의 SVG/render-tree 및 PDF raster 생성 흔적은 있으나 전체 TSV/manifest가 없으므로 완료·통과로 집계하지 않는다. 나머지 전체 검증은 최종 후보로 재실행한다. 완료된3문서의 원문 결과는 보존한다.
- BO 영향 쪽 Native에서 rowbreak8/16=97.67675/98.30849%, nested HWPX13/103/104=99.18042/98.24209/91.14637%, overfill42~45=98.92338/94.80874/97.93569/99.85368%로 해당 쪽은90%를 유지했다. scattered47~52쪽과 nested105쪽은 비교 진행 중이며 이 기록을 전쪽 gate로 해석하지 않는다.

### 2026-10-12: BO 잔여 경계와 BP 진단

- BO `ordinary-source-cut-without-sibling`의 Native 직접 비교에서 scattered 47~50·52쪽은 각각 100.00000 / 99.97722 / 100.00000 / 99.85542 / 99.91901%다. 51쪽은 84.77695%로 미달이다. 중첩 표 13·103·104·105쪽과 overfill 42~45쪽, rowbreak HWPX 8·16쪽은 90% 이상을 유지했다. Native Clippy는 통과했으나 최종 전쪽·WASM 검증은 아직 아니다.
- 51쪽의 첫 인상은 열 폭 차이였지만 실제 RenderTree와 PDF 괘선 좌표를 대조하면 열 경계는 일치한다. 큰 차이는 뒤 표의 세로 원점이다. Native 뒤 표 y=189.4, PDF 약192.43; 앞 표의 끝 이후 빈 문단과 ‘아. 기타’도 약3px 빠르다.
- 원본 HWP→파서 진단 HWPX 투영에서 뒤 빈 문단은 저장 vpos=3876HU, 다음 제목=5476HU, 뒤 표=7396HU다. BO 최종 조각은 반복 머리행과 행237을 수용한 뒤 3634HU만 흐름을 전진시켜 242HU(3.22667px)의 후속 원점 차이를 만든다. `native_text_frame_terminal_successor_gap`는 원본 자연 줄/원점 되감김/물리 프레임을 검사하지만, 첫 조각의 양수 offset과 host spacing_after=0을 이유로 종료 프레임의 독립 원점까지 거부했다.
- BP는 이 두 거부 조건을 제거한다. 최초 anchor offset은 시작 조각 소유이며 종료 이후 원점의 유효성 조건이 아니다. 원본의 자연 줄·미편집·후속 원점 되감김·표의 물리 프레임·같은 단 예산 검사는 유지한다. 입력 후속 LineSeg → 공통 gap 결과 → emit의 occupied+gap 흐름 → 뒤 빈 문단/제목/표 실제 원점으로 소비된다. 음수 gap은 수용하지 않아 출력 겹침을 clamp로 숨기지 않는다.
- 임시 `RHWP_DIAG_FRAGMENT_FRAME` 블록을 제거하고 ordinary source cut 변수명을 실제 의미에 맞췄다. BP 소스와 동결 바이너리를 별도 경로에 남긴 뒤 영향 페이지와 전쪽에서 검증한다.

### BP 실제 경로의 반례와 BQ 공통 인증

- BP 51쪽은 BO와 같은84.77695%로 실제 개선되지 않았다. `stored_table_has_physical_text_frames`의 구 문단간 진행·양수→0 리셋 조건을 통과하지 못해 gap의 소비 경로에 들어가지 않았다. 앞 진단의 조건 제거만으로 수정했다고 판정하지 않는다. BP Docker 빌드는 이 불완전 후보에 대해 중단했으며 최종 fresh WASM으로 집계하지 않는다.
- 전쪽 BP에서 rowbreak HWP19쪽/Print18쪽 불일치를 발견했다. frozen BL/BM/BN/BO의 같은 원본 `dump-pages` 대조는 BL18쪽, BM부터19쪽이었다. 따라서51쪽 보정으로 생긴 회귀가 아니라 BM의 예산 인증이 넓었던 회귀다. 처음 다른 경계는7쪽 pi21: 기존 행0..4 컷`[3,16]` 대신0..3을 수용해 과도한 빈 영역을 남겼다. 원본 pi21에는 내부표가 있으며 부모 LineSeg와 child 줄/컷은 다른 좌표 도메인이다. 부모 object 높이가 전체 행합보다 작다는 것만으로 child 컷의 첫 프레임을 인증할 수 없다.
- BQ는 `stored_plain_fragment_table`이라는 공통 입력 인증으로 원본·미편집·미재조판·완전 저장 폭·선언을 초과하는 알려진 행합·직접 제어 없는 자연 text line coordinate domain을 확인한다. 중첩 개체는 별도 recursive cut/physical-frame 경로의 소유다. 같은 인증을 첫 예산과 종료 후속 원점 gap에 사용하여 두 단계의 잘못된 추측을 제거한다. 구 physical-text-frame 경로는 기존 적용 범위로 유지한다. 원본 plain cell의 빈 글줄은 인증에서 버리지 않는다.
- BP Native/WASM Clippy와 workspace build는 통과했으나 all-targets에서는 기존 `issue2424_block_table_context_owns_step_lifecycle` 생성자의 신규 상태 필드가 빠져 빌드에 실패했다. BQ는 `source_terminal_suffix_start: None`만 추가해 기존 검사 계약을 유지한다. 신규 렌더링 회귀·fixture·기대값 변경은 아직 하지 않았다.

- BP scattered 전52쪽은52/52이며 미달5쪽(28=54.54046,29=59.33675,30=55.25055,31=60.05561,51=84.77695%)이다. 나머지47쪽의90% 이상을 전 문서 통과로 바꾸지 않는다. `stored-terminal-successor-origin-full/sweeps/scattered-hwp/native/scattered-hwp/silhouette.tsv`와 manifest 원문에 보존했고28~31쪽의 직접 비교를 생성 중이다.

- BQ rowbreak HWP는 다시18쪽이며 대표7/8/9/18쪽은93.62865/97.67675/98.83962/97.88029%다. 기존 중첩/overfill/rowbreak HWPX 대표도90%를 유지했다.
- BR 진단: scattered27쪽은 원본 pi33 행51의 일반 셀「모양 및 치수」를 수용하지만 함께 시작하는 rowspan11·6의 새 owner는 미완성 조각이다. 남은 약23.4px에 병합 owner를 열어 「포장용」「모르타」만 일부 표시하며, Print는 그 owner를28쪽에서 시작한다. `RowScanQuery`의 일반 행 whole fit는 행 단위 완결을 셀 owner의 완결로 대용해 기존 partial-cell `MIN_TOP_KEEP_PX` 계약을 건너뛰었다. BR은 새 spanning owner의 물리 row region이 현재 잔여를 넘는지를 실제 rows+spacing으로 확인해 동일 기존 keep 정책을 적용한다. 시작 쪽 강제 진행·온전한 owner 수용·유효한 내부 저장 cut은 기존 경로를 유지한다. 새로운 숫자/문서 ID/행 수 임계값을 추가하지 않는다. 예산 실패는 content를 소비하기 전에 행51에서 컷하여 다음 조각의 시작 owner와 paint도 같은 경계를 받는다. 최종 전쪽 검증 전 후보이며 새 정식 테스트는 아직 추가하지 않았다.

- BQ51쪽은99.90746%로 개선됐다. 첫 원본 plain frame과 후속 문단 원점을 연결한 실제 gap이 소비됐다.
- BR의 기존25px guard 확대는 실패했다. scattered가54쪽으로 늘고 앞쪽 정상 경계부터 바뀌었으므로 채택하지 않는다. 일반 행의 완결/병합 owner의 미완결을 구분하더라도 모든 짧은 병합 시작에 고정 min-keep를 적용한다는 가정은 독립 Print와 맞지 않았다. `merged-opening-keep/`의54쪽 dump·비교 PNG·원문을 보존한다.
- BS는 이 guard를 제거하고 natural first source text frame의 실제 ink+active padding을 사용한다. 일반 한 줄 owner는 자기 한 줄만 요구한다. 같은 가로 영역과 연속 vertical_pos=prev ink+line spacing(+문단 간격)를 인증하며, 자연 되감김은 first frame을 닫는다. 문단 로컬0/0처럼 원본 축을 인증할 수 없는 입력은 미검증으로 남겨 기존 경로를 쓴다. 새 owner가 시작될 때 첫 source frame이 지금 예산에 들어가고 fresh 단에도 들어가는지를 확인하되, 이 요구를 첫 행 높이에 더해 뒤 행과 이중 예약하지 않는다. 기존 CellUnit 소유·행 컷과 실제 paint의 이어받기는 유지하며 영향 출력에서 직접 검증한다.

- BP 전쪽 Native driver는10문서375쪽의TSV/manifest를 산출했다. 중첩HWP/HWPX 각115쪽 최저91.14637%, overfill48쪽93.03871%, rowbreak HWPX18쪽90.85014%, cell image HWP/HWPX 각8쪽96.87527%, vpos HWP/HWPX 각5쪽99.436%, picture toggle1쪽99.95151%다. scattered52쪽은 앞 기록의5쪽 미달이다. rowbreak HWP는19/18쪽수 불일치로 측정 불가였으며BQ가18쪽으로 복구했다. driverexit2와 각 문서 실패/통과·미측정은 원문`stored-terminal-successor-origin-full/native-aggregate.json`과logs에서 분리했다. 이 결과는최종BS/후속head의검증이 아니다.


- BS는 scattered 28~31쪽을 개선하지 못했다(54.54046/59.33675/55.25055/60.05561%). 원본 pi33의 완전 셀 소유는 있지만 행별 저장 폭합은 47953/48104HU 두 값으로 달라 전체 `stored_complete_column_frame` 인증이 성립하지 않는다. 별개의 셀 첫 유닛을 수용하는 데 표 전체 폭 인증을 요구한 가정도, 연속 저장 축 전체를 불가분으로 간주한 가정도 채택하지 않는다. PDF28쪽은 행77의 여러 줄 병합 owner를 실제로 부분 수용하고 행80을29쪽으로 이어받는다. `initial-source-owner-frame/`에 BS 원문과 source/binary를 보존했다.
- BU는 **기존 CellUnit의 첫 유닛 높이 + active cell padding**을 일반 행에서 함께 열리는 각 rowspan owner의 수용 조건으로 쓴다. 한 줄 셀의 마지막 advance와 뒤 줄이 있는 첫 유닛의 advance는 기존 유닛 producer가 구분한다. 전체 셀/첫 연속 frame을 원자화하지 않으며, 개체를 가진 재귀 셀은 이 직접 plain text 경로에 들어오지 않는다. 원본 RowBreak/미편집/미재조판 및 해당 셀의 실제 저장 줄·소유를 확인한다. scan-local 행별 최대값을 한 번 산출하여 불필요한 전 표 재인증을 반복하지 않는다. 실제 예산→end_row→이어받기→paint 소속을 영향 쪽에서 확인할 때까지 후보로 남긴다. BT는 누락된 `ScanInput` 패턴 필드로 컴파일에 실패했고 실행 증거로 세지 않는다.

- BU의 첫 유닛 advance 전부를 첫 수용 높이로 쓰는 가정도 실패했다. scattered는54쪽으로 늘고 원래4쪽의 원본 행53에서 먼저 차이가 생겼다. 그 셀 첫 문단은 두 LineSeg가0→0으로 나뉘며, 뒤 문단의 줄간격은 현재 조각의 ink를 늘리지 않는다. 실패 원문·54쪽 dump는 `first-unit-opening-reservation/`에 보존한다.
- BV 후보는 표 전체/셀 전체/advance 합 대신 **첫 문단의 원본 저장 프레임 ink+active padding**을 조회한다. 같은 문단의 저장 rewind가 첫 프레임을 닫으며, 뒤 문단은 별도 줄 소유로 계속 capacity cut을 허용한다. 입력 경계 반례는 원본 행53의 문단 내0→0, 행51의 동일 문단 내0→1500, 행77의 여러 독립 문단이며, 한컴 Print의4·27·28·29쪽과 실제 컷으로 대조한다. 아직 실행 검증 전이므로 채택·완료로 판정하지 않는다.

- BV scattered는52쪽을 유지하고27=99.77192%,28=95.78226%로 개선됐으나29~31은62.52953/57.72923/64.37011%다. 실제 컷은51..81이며 Print는51..80이다. `RHWP_TABLE_DRIFT=1`에서 같은 original continuation이 host_before1.9px/outer_bottom0px로 scan되고, emit의 걸침 칸 page-end band는 별도로 표 outer bottom을 제외한다. 즉 paint가 비워 두는 하단 영역을 scan이 ordinary row 수용에 사용한다. BW는 이미 재개하는 nonterminal 개체 프레임의 actual bottom margin을 scan budget에 예약하고 **같은 start state를 재스캔**하여 end-row/cut/physical-demand를 함께 확정한다. 종료 조각의 trailing-flow 계약은 유지한다. 기존 하단 band의 중복 차감 여부도 실제 소비 위치에서 확인 중이다. 아직 후보이며 전쪽 게이트 판정 전이다.

- BW Native dump는52쪽이며 pi33 컷이27쪽5..51,28쪽51..80,29쪽80..126,30쪽126..171,31쪽171..214로 바뀌었다. 대표 PDF29쪽 시작「표면층 두께」/30쪽「반복압축재하」/31쪽「온도 의존성」과 소유를 추가 대조 중이다. nonterminal refit는 read-only `scan_table_fragment(&TypesetState)`를 같은 FragmentStart로 호출하므로 첫 스캔의 cut을 소비하지 않는다.
- BX는 scan에서 이미 예약한 margin을 emit의 `accepted_straddle_end_band`·이어받기 원장의 `page_end_band`에서 다시 빼지 않도록 한다. 상단 프레임 predicate→pending bottom property→nonterminal actual scan refit→FragmentBudget→emit의 physical band·후속 flow·straddle ledger를 연결했다. 종료 조각은 refit하지 않고 기존 terminal trailing 정책을 소비한다. 새 상수/문서ID/row-count 예외는 추가하지 않았다. 최종 후보의 전체393쪽 Native/fresh WASM와 직접 판독 전에는 최종 gate를 선언하지 않는다.

- BX scattered代表20쪽 모두90% 이상(최저97.47495%)이며28~31은99.14627/99.97455/97.84429/100.0%다. 기존 rowbreak/overfill 대표도90% 유지, 전쪽 Native rowbreak HWP/HWPX 각18쪽 전부90% 이상이다. 그러나 nested는20/115쪽으로 내용 누락이 발생하여 최종 gate 실패다. 해당 full driver는 실패/중간출력을 보존한 채 중단했으며393쪽 통과로 표현하지 않는다.
- 실제 nested 컷은20쪽에서717..755. `terminal_cut_consumed`와 다음 단계의 `skip_consumed_row`가 `advance_row_cut(...MAX).consumed_height<=0`를 유닛 소진으로 대용했다. height0·저장 경계로 멈춘 probe 뒤에 실제 owner 유닛이 남으면 종료 근거가 아니다. BY는 두 지점에서 기존 `row_cut_units_exhausted`의 셀 소유별 cut>=units.len 검사로 바꾼다. spanning owner는 그 별도 소유 원장이 끝나기 전 소진으로 판정하지 않는 기존 helper 계약을 따른다. 신규 정식 회귀는 아직 추가하지 않았으며, 수정 전20쪽 누락·수정 후 전체 내용/쪽수·Native/fresh WASM 검증을 이어서 연결해야 한다.
- BX fmt·Native Clippy·WASM lib Clippy·workspace build·workspace all-targets Clippy·manifest/tier(base bfa28a77a2) 모두 통과했다. 이는 BY/후속 수정의 최종 lint 증거가 아니다.

- BY는 nested HWPX115쪽을 복구했다. 변경 경계20=98.91736%,21=99.13718%,마지막115=99.79709%이고13/103/104/105 대표도90%를 유지한다. 유닛755 앞뒤의 이어받기와 끝 쪽은 `structural-cut-exhaustion/logs/nested-pages.json` 및 `nested-middle/`에 보존했다. 같은 BY의 scattered代表20쪽은 모두90% 이상(최저97.47495%)이다. 새 전쪽 Native driver와 Docker WASM 빌드를 시작했으며 아직 전쪽 gate 완료가 아니다. BY fmt/세 Clippy/workspace/manifest/tier는 통과했고 기존 nextest·Native Skia3종은 실행 중이다.

- BY 전체 nextest는10,220건 실행/10,144통과/76실패/45skip이다(`logs/by-nextest.log`). Skia3종 후속 pipeline은 nextest 실패로 실행되지 않았으며 통과로 기록하지 않는다. 정상 devel bfa28a77a2의 같은 실패76건을 별도 worktree에서 대조하여 기존 실패와 보정 회귀를 분리 중이다. 회귀 기대값·baseline·허용치 변경 없음.
- BZ는 저장 inline pen의 owner 판정을 visible-char projection 대신 기존 `stored_control_line_indices`의 PARA_TEXT UTF-16 축으로 통일한다. #6122의 두 컨트롤은 visible text gap을 공유해도 서로 다른 저장 줄을 소유한다. 이전 그림 뒤에 둘째 그림의 x를 더한 잘못된 pen이 폭 개행 후의 실제 원점을 덮어쓰던 원인이다. 원본/전후 render tree는 `regression-diagnosis/{current-base,ordered-inline-frame,ordered-picture-pen,structural-cut-exhaustion}/render_tree_006.json`에 보존했으며 BZ 실행 전이므로 완료로 판정하지 않는다. BY frozen Docker WASM은 pkg/public 동일 SHA-256 24707e47d2c9cf262e59e9ec40a87d7db56986c5b4d2b25c95ac59379e71964c이고 전쪽 WASM 드라이버가 실행 중이다. BZ Rust와 이전 BY 증거를 구분한다.


### BY 전쪽 결과와 회귀 대조 / BZ 후속 진단

- BY Native 11문서393쪽 중392쪽은90% 이상이다. nested HWPX110쪽은39.55812%이며, 같은 쪽 빈 문단2408이109쪽에서 이월되어 `[부록 1]`부터25.6px 낮아진다. 전쪽 gate 완료·통과가 아니다. 원문은 `structural-cut-exhaustion-full/native-aggregate.json`. 같은 frozen Docker WASM은9문서163쪽까지 모두90% 이상이며 nested 두 문서는 실행 중이다.
- 고정 devel bfa28a77a2의 별도 worktree에서 BY 실패76건을 그대로 실행한 결과76건 모두 통과했다(`logs/by-regression-base-nextest.log`). 새로 유발된 검사 실패로 기록하며, 독립 Print와 다른 기존 검사 가정은 개별 증거로 분리한다.
- BZ 첫 source frame 진단: cosmetics pi13의cut/paint/whole-fit 첫15행 합은969.2666666666667px로 선언 frame과 정확히 동일하다. 실제 기각은 엄격한 부동소수 누적 비교가 원인일 가능성이 있으며 원문은 `confirmed-source-boundary/logs/cosmetics-rows.log`. prep pi376은9행221.52px지만 source frame218.02666666666667px다. 저장 마지막 행은1000HU ink/500HU advance와 CENTER 정렬이고 실제 끝 조각은 빈 physical band를 줄여도 ink가 들어간다. 무조건 nearest row overflow를 허용하지 않고 source frame 안의 실제 content/align 점유로 인증·예약·paint에 같은 last-row 높이를 전달한다. scattered pi29의43행970.7866666666667px는 source969.08px를 넘고 마지막 행의ink+active padding도 잔여19.1733px에 들어가지 않으므로42행 경계를 유지해야 한다.
- BZ는 원래 UTF-16 inline owner 공유 외에, zero-origin source cut의 **선택된 정확한 후보**에서 다음 자연 advance 또는 앞 물리 frame 폐합을 확인한다. 나중 확인된 rewind로 앞 후보를 승인하지 않으며 단순 reset과 쪽 소유를 구분한다. `confirmed-source-boundary/`에source patch/Native CLI를 고정했고 focused 기존 검사 실행 중이다. 이는 최종 lint/회귀/fresh WASM 증거가 아니다.

### CA–CC: source frame의 실제 ink·간격 계상

- BZ focused 기존 6건 중 #6122와 #6761 네 반례(총5건)는 PASS, #6706은 앞 간격 누락으로 FAIL이다. #6706의 source3980HU를 구역 reflow가3412HU로 재구성한 뒤, 아직 소비하지 않은568HU before-spacing을 HeightCursor가 다시 차감한 원인이다. 원본 스냅샷의 gap=before, 재구성 gap=0임을 인증한 경우 차감하지 않고 실제 paint가 한 번 적용하도록 보정한다.
- CA Native에서 cosmetics11쪽/첫15행, prep140쪽/37쪽 첫9행을 복구했고 scattered52쪽 유지. cosmetics의 같은 source frame binary sum만 f64 summation bound로 비교하며 새 physical tolerance는 없다. prep는 마지막 행 원본 plain ink+active padding이 잔여 slot에 들어가는 경우에만 scan 전에 cut/whole-fit/paint 모두 같은 slot 높이를 소비한다. scattered의 잔여19.1733px보다 요구20.88px가 커 인증되지 않는 반례를 유지한다.
- CA selected Native cosmetics2·3 / prep37·38은 gate passed이고 쪽수도 일치한다. 전쪽/fresh WASM/final source 통과로 승격하지 않는다. 원문은 source-frame-ink-budget/sweeps/.
- CC는 저장 물리 쪽 reset 직전의 빈 Enter도 마지막 line box는 보존하고 사용되지 않는 successor advance만 같은 cut-height helper에서 거둔다. 진행하는 다음 vpos만 확인하는 empty-owner helper는 0 reset에 적용할 수 없으므로 그 가정을 제거했다. #6706 before-spacing 수정이 Native 빌드 실행 중 들어간 중간 build는 종료해 증거에서 제외했으며 고정 source의 재빌드를 시작했다.

### CD — 닫힌 저장 줄 프레임 재비교(로컬 검증 중)
- CC Native: nested HWPX 109/110/111쪽은 99.79772/97.41933/97.87759%; 110쪽 빈 줄의 line box는 보존하고 다음 프레임으로 넘길 trailing advance만 제거했다. 110쪽의 제목·본문·문단 순서와 시작점을 독립 PDF에 직접 대조했다.
- CC Native: rowbreak HWPX 4/6/7/8/15/16쪽은 모두 90% 이상(최저 93.61131%, 18/18쪽). 16쪽 하단 표와 글상자·뒤 내용의 배치를 직접 대조했다.
- CA 대조군: cosmetics 2/3쪽과 prep 37/38쪽의 표 외곽·글줄·뒤 문단을 직접 판독했다. 각각 11/140쪽 복구. 전체 페이지와 fresh Docker WASM은 후속 최종 소스로 재검증한다.
- #6706: 원본 source vpos 3980→16460의 간격은 앞 Shape 줄 높이 11064 + 현재 앞 간격 1416HU다. 재구성된 model vpos 3412→14476에는 그 간격이 없다. px→HU 변환은 가장 가까운 정수 HU로 복원하지만 CD에서도 제목 y=295.0이 남았다. 실제 원인은 이전 문단에서 소비한 앞 간격 568HU도 재구성된 누적 축에서 빠지는 것이다. source certificate가 입증한 연속 model 원점에는 절대 vpos snap을 적용할 수 없다. 실제 앞 항목 끝을 원점으로 소비하고 현재 문단 앞 간격을 한 번 적용하도록 CE에서 보정한다. 별도 px 허용치를 늘리지 않는다.

### CE — 재구성 축의 앞 간격과 빈 문단 다음 프레임
- #6706 소비 경로: 원본 source vpos와 model vpos의 차이 → HeightCursor의 source gap 인증 → 연속 재구성 축에서는 이전 항목의 실제 끝을 원점으로 사용 → typeset/paint에서 현재 앞 간격 한 번 적용. 절대 재구성 vpos는 앞 항목의 소비 간격을 포함하지 않으므로 원점 snap 근거로 쓰지 않는다.
- #1770: 2쪽 마지막 두 줄(원본 vpos 68152/70072HU, LH1200/LS720) 뒤 빈 문단(vpos0)의 line box는 다음 쪽 소유다. visible hard-break 신호가 없어도 원본의 자연 줄·공유 좌표 사다리·빈 문단의 원점 되감김을 확인한 용량 컷은 앞 줄의 trailing advance만 제외한다. 강제 조기 페이지 나눔이나 48px 초과 허용을 복구하지 않는다. 컷 선정·높이 소비·배치가 기존 shared trim 결과를 소비한다. CE 빌드와 독립 PDF 대조는 미완료다.
- CE 기존 실패 76건 재검사: 28 PASS / 48 FAIL. #6706, #6035, #6122, #6761 반례 2건, prep 페이지·소유·KoPub 등의 기존 검사들이 복구됐다. #1770은 5쪽이며 같은 2쪽 컷 81이 남는다. 마지막 두 줄 이전에 중첩 표에서 만든 누적 공간이 source vpos보다 커지는 원인을 추가 추적한다. 기존 검사·baseline은 변경하지 않았다.

### CF — 메인테이너 시각 판정 및 중첩 표 원장 보정
- 메인테이너가 Docker WASM Studio에서 rowbreak HWPX 4·6·7·8·15쪽의 표 분할·빈 영역·이어받기 간격을 직접 확인하여 시각 판정 통과로 회신했다. 이 판정은 제시한 WASM 산출물에 대한 것이며 후속 Rust 변경의 최종 검증을 대신하지 않는다.
- #1770 원본의 빈 표 host 문단6은 다음 저장 줄25319HU가 표 오프셋·높이·바깥여백만으로 닫히지만 per-row 측정은 spacing-after 4px를 중복 계상했다. 문단25는 제어문자 위치0과 뒤 텍스트, 전체 저장 줄이 positive offset 이전에 끝나는 head-host인데 tail-only 조회가 실패하여 host 29.6px와 nested fallback 4px를 중복 예약했다. 공유 `stored_nested_float_placement`에서 기존 head-host producer도 호출하고, per-row 유닛은 독립 successor가 닫는 동일 frame의 opening/terminal band를 소비한다. 최종 cut·paint·쪽수는 실행 대조 전이며 새 검사·기대값은 변경하지 않았다. CE의 빈 successor 추가 분기는 기존 stored-frame flag와 중복이고 #1770 원인이 아니므로 제거했다.

- CF Native #1770은 한컴과 같은4쪽을 복구했고 전쪽 최저93.01608%다(1=95.71753/2=96.71739/3=93.01608/4=94.25576). 1쪽 계약기간·업무내용 끝, 2쪽 근로시간 표와 임금 두 끝줄, 3→4쪽 준용규정 이어받기 및 마지막 서명란을 직접 판독했다. 내용 소속·순서·표 외곽과 다음 문단이 일치하며 기호·괘선 굵기의 작은 차이는 남는다. source/CLI hash와 TSV·PNG는 `shared-nested-host-frame/`에 보존했다. 최종 fresh WASM은 아직 미실행이다.
- #6712의 새 차이는 Square 표 자체의 reservation이 아니라 그 옆 저장 빈줄18–26을 다시 더한 중복이다. base와 CF에서 앞 표 mixed 유닛은 동일하지만, base0px인 옆 밴드가 CF17.1/21.3px로 합산되어 총172.2px 커졌다. paint는 공통 `stored_nested_table_empty_wrap_spacer` 소유 판정으로 해당 이미 계상된 밴드를 건너뛰는데 새 일반 empty-successor producer가 그 판정보다 먼저 높이를 적용했다. 기존 source 옆 밴드 소유와 standalone empty Enter를 구분하여 같은 결과를 소비하도록 후속 보정한다.

- CF 잔여48건 재검사는 2 PASS / 46 FAIL이다. #1770 및 해당 oracle 쪽수 검사 복구를 포함하며 원문은 logs/cf-48-nextest.log와 regression-diagnosis/cf-48-result.json. CG는 Square 표가 이미 소유한 옆 빈줄 밴드에 general empty-successor advance를 중복 적용하지 않도록 같은 paint owner predicate를 소비한다. 독립적인 빈 Enter와 이번 rowbreak 보정은 유지하며 경계·대조군을 다시 실행한다.

### CG–CI: 빈 밴드 소유와 온전한 저장 표 프레임
- CG 기존 잔여46건 재검사: 9 PASS / 37 FAIL. 원문은 `logs/cg-46-nextest.log`, 정확한 검사별 결과는 `regression-diagnosis/cg-46-result.json`이다. 최초76건 중39건이 복구됐으며 최종 전체 회귀 통과가 아니다. 한국어 가정통신문 전2쪽 Native는97.91671/94.09359%다. 중국어1쪽은83.04707%,2쪽95.41581%이며 고정 devel bfa28a77a2의 새 CLI로 같은 입력/Print/글꼴을 재실행해 같은83.04707%를 확인했다. 기존 미달과 이번 회귀를 구분하고 원문을 `latest-pinned-base-controls/`에 보존한다. 기존 검사의 기대값을 바꾸지 않는다.
- 초기 CG 비교 driver에는 FONTCONFIG_FILE이 누락됐고 review 준비 경로도 이전 폴더 규약이었다. 해당 비교 원문은 보존했고, Windows face 공급 설정·고정 CLI를 사용한 `owned-wrap-empty-band-windows-fonts/`에 새로 재실행했다. 이전 결과를 최종 증거로 사용하지 않는다.
- #5057의 구 PDF는 7쪽 모아 찍기였다. 한컴 MCP에서 원본 lastSavedVersion에 맞는2020 engine/PrintMethod0/one-up 인쇄 계약으로 독립13쪽 기준본을 생성했다(job3190df04-280a-4610-83d5-54ec7876f4c3, 실제11.0.0.9136, preprocessing none). 입력/PDF·MCP 결과는 `cf-regression-visual/reference/`와 logs에 보존했다.
- CH는 마지막 행까지 포함한 첫 source frame도 closing ink+active padding을 검사하도록 했지만, 13쪽 복구만으로 해결 판정하지 않는다. 7쪽72.98515%이며 scan의 저장 행합과 paint의 비례 축소 행합이 달라 표 바닥이 짧아진다. 전체 원본 행합39657HU, 개체38873HU, 마지막 행4095HU의 잔여3311HU(44.14667px)는 원본2줄 ink2760HU+padding282HU보다 크다. 앞7행 원점을 비례 축소할 근거가 없다.
- CI 후보는 원본의 완전한 열 소유·자연 단조 줄·plain cell·활성 padding·각 행 ink fit로 **마지막 물리 밴드만 닫힌 전체 행 프레임**을 생산하고 HeightMeasurer, LayoutEngine resolve, whole-row cut 조회가 함께 소비한다. 부분 source frame·중첩/rowspan·되감김·편집/reflow는 적용하지 않는다. `stored_plain_object_row_frame_heights → measure row_heights / scan whole cut / paint resolve`와 후속 공통 높이 fitting의 덮어쓰기를 실행에서 확인한다.
- 같은 열0/0만으로 physical page reset을 인증하던 가정도 제거한다. 앞 행+ink+padding이 개체 프레임을 닫는 기존 인증 또는 셀 물리 프레임 폐합을 요구하며, 가로 wrap pair와 공동 source frame 경계의 별도 계약은 유지한다. 기존 #6712 수동 same-column 반례와 실제 rowbreak/nested/overfill을 함께 재실행한다. 두 보정 모두 후보이며 새 회귀·fixture·baseline 갱신 및 원격 반영은 아직 하지 않았다.


### CI–CK 후속 원점·프레임 계약 보정
- CI 기존75개 integration 실패 재검사에서41 PASS /34 FAIL이다. source-side gray-tail 1건은 이번 integration 재실행 범위가 아니며 여전히 미해결로 구분한다. 원문 `logs/ci-75-nextest.log`, 검사별 `regression-diagnosis/ci-75-result.json`. 기존 기대값·래칫·fixture 변경 없음.
- CI 전쪽 Native: rowbreak HWPX18쪽 최저90.85014%, nested HWPX115쪽 최저91.14637%, #1770 전4쪽 최저93.01608%. 후속 CK 코드/fresh WASM 통과로 대신하지 않는다. #6706 대조군은64쪽 중55쪽 미달이며 기존 위치·그림 표현 문제를 자동 예외로 취급하지 않는다.
- CJ는 원본 plain object closing-row 프레임을 소비하는 첫 조각에도 `fragment_flow_origin + host_before_overhead + vert_offset_overhead` 원점을 함께 전달한다. 전체 내용이 수용돼도 마지막 빈 밴드 보정 때문에 PartialTable로 출력되는 경로를 놓치면, 새 단에서 예약한 위여백3.77333px가 paint에서 빠진다. 유효한 기존 placement는 우선하며 편집·부분 컷·이어받기·불인증 프레임은 새 분기의 대상이 아니다. `budget.fragment_placement → emit record_paragraph_float_placement → layout_partial_table_item resolved_table_top → table_partial y_start`의 실제 연결을 확인했다.
- CJ #5057은13/13쪽, 전쪽 Native 최저96.52065%, 7쪽99.28375/8쪽99.93625%다. 7쪽 세 표의 모든 행 괘선·2줄 마지막 행·앞뒤 표제와 8쪽 신청서의 서식·서명·붙임란을 직접 대조했다. 독립 Print와 본문 소속·외곽·내용 순서가 일치하며 선 굵기/글꼴의 미세 차이가 남는다. `shared-original-frame-origin/`에 CLI/source patch/provenance/TSV/review PNG를 고정했다. fresh WASM은 아직 미실행이다.
- CK는 위 바깥여백을 열었다는 사실만으로 아래 여백까지 예약하던 일반 추론을 제거한다. 반복 여백·캡션·종료 프레임의 명시적인 기존 계약은 유지하고, 실제 nonterminal paint가 닫는 내용 상자만 예약한다. #5966 최초 차이는135쪽의 같은 start cut63에서 end cut3→2였고, 아래 여백3.77333px의 추가 refit이 독립142쪽 Print의 마지막 유닛을 기각했다. 구현 후보 실행 후 관련 원본/rowbreak/중첩/overfill을 다시 대조한다.
- CK는 첫 source frame의 소유 판정과 trailing trim의 양을 분리한다. #5885의 첫 빈 줄은 유효한 source 줄과0→0→1920HU의 형제 셀 사다리를 공유하지만 trim=0일 수 있다. 양수 trim을 물리 경계 인증의 대용으로 쓰면1쪽 마지막 줄 조각이 최소25px 조건으로 이월된다. 같은 저장 인증과 실제 CellUnit 소속을 bool producer로 공유하며, 전체0 사다리·controls·편집·로컬 reset 반례는 기존 배제를 유지한다. 빌드/실행 중이며 완료로 기록하지 않는다.


### CL: 그림 안내 밴드와 재검사 결과
- 기존 그림 흐름 producer가 이미 소유한 빈 안내줄에는 일반 저장 successor advance를 다시 더하지 않는다. 독립 빈 Enter의 순차 점유는 유지한다. 공개 #1921 원본은 38→37쪽, #5885는8→7쪽, #5966은143→142쪽으로 복구됐다. 쪽수만으로 완료를 판정하지 않는다. `shared-object-guide-ownership/`의 patch SHA-256 `2b90c41740f720332100aa972d150eda2a8262206c66a30ab8f3e4ec22cc9090`, CLI `bf7b99e12fb176136927dfa9bddb5a9bd30735d4a6bce85393a1cd2341154ee5`에 고정했다.
- 기존75개 integration 실패 재검사에서53 PASS/22 FAIL이다. 원문 `shared-object-guide-ownership/logs/retry-nextest.log`, 개별 결과 `regression-diagnosis/cl-75-result.json`. gray-tail source-side1건은 이 재검사 범위 밖이다. 처음 실패76건 중23건이 아직 통과하지 않았으며 최종 전수 검사 결과가 아니다. 검사 기대값·baseline·fixture 변경 없음.
- 같은 CL Native 선택 쪽: rowbreak HWPX4·6·7·8·15·16, nested HWPX109·110·111, #5057 7·8, #1770 전4쪽, #5885 1·2·3, #1921 11·12·13은 비교 gate를 통과했다. 최종 head의 전쪽/fresh Docker WASM 결과로 대신하지 않는다. #5885의1·2쪽에서 source 첫 빈 줄의 쪽 소속·표 외곽·이어받는 신용도/기술개발 내용을 직접 대조했다. #1921 11쪽은 그림·내용 순서는 복구됐으나 마지막 외곽 아래 빈 띠의 차이가 보여 pinned devel 출력과 별도 대조 중이다. 점수 통과만으로 이 차이를 면제하지 않는다.
- 편집 재현에서 HWP는 원본115→편집115→재열기116쪽, 직접 HWPX는115→편집123→재열기115쪽이다. #2185의 HWP 편집 직후 쪽수는 복구됐지만 저장 계약은 미충족이다. 실제15쪽의 중첩 개체 유닛 위치가 달라지는 원문·source/CellUnit trace를 `regression-diagnosis/cl-edit-*`에 보존했다. 전체 편집 세션이라는 이유로 변경되지 않은 자식의 독립 저장 프레임까지 기각하는 가정을 제거해 국소 dirty/재조판/저장 폐합 인증을 사용하는 CM 후보를 검증한다. 후보 구현은 통과 판정이 아니다.


### CM–CO: 후보 철회와 capacity/source 경계 분리
- CM은 변경되지 않은 자식의 저장 프레임에 전체 편집 세션 배제를 제거한 후보였다. 동일 공개 재현에서 HWP 115→115→116쪽, HWPX 115→123→115쪽으로 CL과 같아 원인 해결이 아니었다. 해당 후보 변경을 철회했다.
- CN은 deferred TopAndBottom 그림의 국소 인증만으로 예약·paint가 같은 picture band 높이를 쓰게 한 후보다. HWP 115→115→116쪽으로 미해결이고 HWPX 편집 직후는124쪽으로 악화되어 철회했다. `local-picture-band-ownership/candidate/` 및 `regression-diagnosis/cn-edit-*`는 실패 증거로만 보존하며 완료 증거가 아니다.
- CL 전쪽 Native 재실행 중 scattered HWP52쪽에서11쪽이90% 미달이다. 보존한 BY→BZ→CA→CC→CD→CE→CF→CJ→CK CLI로 원본 컷을 대조해 BZ의19·20·21·22·47~52쪽 차이와 CK의28~31쪽 차이를 분리했다. 기존 입력/Print/글꼴이 같고 BY가 전쪽90% 이상이므로 현재 수정으로 유발된 회귀다.
- CO는 같은 줄의 서로 다른 저장 텍스트 범위를 가용 높이로 나누는 capacity 컷과, 여유가 있어도 다음 페이지로 보내는 물리 source 경계를 분리한다. 원래 `stored_plain_restart_cut(..., false)`는 capacity 후보·최소 물리 높이를 생산하고, `(..., true)`는 독립적인 프레임 폐합까지 요구한다. 고아 줄 기각 면제에는 capacity가 실제 선택한 정확한 컷을 사용하며 whole-row 강제 분할에는 기존 physical 인증만 사용한다. `row_plain_restart_capacity_cut → row_step의 정확한 end_cut 대조 → 실제 높이 fit → partial 배치`를 검증 중이다. 문서 ID·수치 예외 및 검사 기대값 변경은 없다.
- #1921 11쪽 아래 빈 띠 외곽 차이는 pinned devel bfa의 같은 원본·Print·글꼴 재출력에서도 동일했다. 새 CL 회귀와 구분하고 `cl-regulatory-pinned-base-visual/`에 보존했다. 기존 차이의 해소나 최종 fresh WASM 통과로 보고하지 않는다.


### CP–CQ: 마지막 source 줄 꼬리와 저장 조회 문맥
- CO는 scattered19쪽 컷을 복구했지만 #6761의 모든0/나중확인 두 반례를 위반했다(22개 중20 PASS/2 FAIL). 더 긴 사다리 중간의 local0를 마지막 줄 꼬리로 취급한 잘못된 가정을 수정했다.
- CP는 capacity의 마지막 source 줄 꼬리(다른 보통 셀은 이미 완결)를 physical source 강제 경계와 구분한다. 같은 자연 문단의 마지막 줄로 이월할 때 앞 줄의 successor advance는 그 다음 줄 소유이며, 후보 높이와 실제 cut 높이가 같은 trailing-trim을 소비한다. 임의의 앞0와 뒤에 확인된 rewind는 physical 인증이 아니다. Native 컷은 scattered19쪽147행[1,1] 및47쪽100행[1]을 복구했다. #6761 네 검사와 #6712 열/그림/빈 줄22개 모두 PASS다(`logs/cp-restart-focused-nextest.log`, native-skia feature). 전쪽 비교는 실행 중이며 CK에서 발생한28~31쪽은 별도다.
- CQ 원인 후보: 저장 `writeback_reflowed_table_frames`가 pagination의 start_cut을 live IR `cell_unit_line_anchors`로 조회하나, 저장 전에 paint/cursor가 호출되지 않으면 persistent LayoutEngine에 effective 편집 profile/본문 크기/구역 간격이 설정되지 않는다. 같은 helper라도 컷 구성 문맥이 다르면 유닛 서수가 다른 줄을 가리킨다. 저장 조회 전에 캐시를 지우고 동일 profile/overlay, section column spacing 및 실제 page layout을 설정하는 보정 후보를 빌드 중이다. 편집·재열기 검사 전에는 해결로 판정하지 않는다.
- CL 11원본 전393쪽 Native 측정이 완료됐다.382쪽90% 이상/11쪽미달은 scattered HWP에 한정된다. rowbreak HWP18쪽 최저93.62865%, HWPX18쪽90.85014%, nested HWP/HWPX115쪽 각각91.14637%, overfill48쪽93.03871%다. `shared-object-guide-ownership-full/native-aggregate.json` 참조. 이는 CL 고정 소스의 수치이고 이후 CP/CQ·fresh WASM의 최종 결과가 아니다.

- CP scattered 전52쪽 Native 측정 완료:49쪽90% 이상/29·30·31쪽 미달(62.52953/57.72923/64.37011%). BZ에서 발생한19~22 및47~52쪽 차이는 전쪽 측정에서 복구했다. CK의28쪽 한 행 추가로 시작한 후속3쪽 차이는 남는다. 전체 컷/TSV는 `terminal-restart-tail/`에 고정했으며 fresh WASM은 미실행이다.


### CR: 아래 여백을 물리 경계와 교집합으로 예약
- CQ의 query 문맥 후보는 공개 재현에서 HWP115→115→116/HWPX115→123→115쪽으로 바뀌지 않았다. 해당 후보만 철회했으며 `writeback-pagination-context/`와 `cq-edit-*`는 실패 증거로 보존한다. 실제 source/normalized IR의 컷 서수 대응도 대조해야 하므로 저장 원인 완료로 판정하지 않는다.
- CK의 아래 여백 제거는 scattered28쪽에 표면층 두께 행을 추가했다. 원본 continuation 예산은986.4px, 추가행까지985.7px로 남은0.7px가 바깥 아래 여백1.88px보다 작다. 독립 PDF는 그 행을29쪽부터 소유한다. 반면 #5966의135쪽은 각주/본문 예산868.0px가 실제 body876.8533−상3.7733−하3.7733=869.3067px보다 이미 작다. 일반 아래 여백을 거기서 다시 빼면 중복 차감이다.
- CR은 공통 `column_rowbreak_nonterminal_bottom_margin_px`로 nonterminal 이어받기 프레임의 외곽 아래 점유를 scan·paint가 함께 소비한다. budget은 실제 body 경계에서 opening/caption/offset/아래 여백을 뺀 content limit과 기존 footnote/footer capacity를 교집합으로 사용한다. 첫 scan이 nonterminal이면 정확한 컷을 이 경계에서 다시 선택하며, 더 작은 예산에는 여백을 다시 차감하지 않는다. 종료 조각/기존 반복 여백은 각자의 기존 계약을 유지한다. 빌드/실행 대조 전 후보이며 검사/기대값 변경 없음.

- CR 고정 후보 전쪽 Native에서 scattered52/52쪽 모두90% 이상(최저93.25935%)으로 BZ·CK에서 발생한 전체 미달을 복구했다. rowbreak HWPX18/18쪽 최저90.85014%도 유지됐다. 나머지3문서 측정과 fresh Docker WASM은 아직 완료 전이다.
- CR 기존75 integration 재검사는51 PASS/24 FAIL이다. CL에서 통과한 #4138 두 편집 저장 검사 및 #7379 캡션 예산 검사에서 새 실패가 발생했으며, 편집 거대 셀의 live 쪽수도115→117로 악화됐다. 이 후보를 최종 완료로 판정하지 않고 캡션/원본 프레임/재조판 소유의 실제 소비 경로를 보정한다. 기대값·baseline 변경 없음.

### CS: 바깥 여백 재개와 걸침 셀의 물리 끝 밴드 분리
- CR의 핵심 가정인 “위 여백을 재개하는 모든 중간 조각이 아래 여백도 재개한다”는 실제 경로와 다르다. `emit`의 기존 `accepted_straddle_end_band`/`straddle_page_bands_after_fragment`는 **걸침 셀 안의 행 경계**가 남은 빈 물리 밴드를 소유한다고 정의한다. 같은 행의 내용 컷(거대 셀 편집)과 캡션 종료를 기다리는 온전한 행 경계는 그 소유자가 아니다. #7379의 캡션 아래 여백32700HU가 중간4..6행에435.76px로 잘못 예약된 실제 실패를 확인했다.
- CS는 물리 아래 경계를 실제 걸침 owner의 행 경계 컷에 연결한다. 위 여백 재개를 아래 여백 소유의 대용으로 쓰지 않는다. 같은 helper는 scan 결과의 end-row/컷 유형과 paint의 최종 end-row/컷 유형을 소비하고, 각주 예산과의 교집합은 유지한다. 문서 ID·편집 세션 예외·px 상수와 검사 변경은 추가하지 않았다. 빌드/경계 실행 전 후보이다.

- #5128 진단에서 원본 HWP와 HWP5-origin HWPX의3구역 pi296은 같은 host/source top770.0933, 측정 행합102.1067, 뒤 host rewind를 가지지만, `query_whole_table_fit`의 `source_frame_matches_profile`이 **컨테이너 종류(hwpx_container)** 로 계약을 선택했다. 원본은 native stored-tail 인증을 기각하고5행 뒤 분할하나, 마커를 가진 왕복 HWPX는 일반 HWPX의 넓은 source-top 인증으로 통째 수용하며45쪽부터 소유가 달라진다. 저장 조판의 출처 프로필(HWP5-origin)과 ZIP 컨테이너를 분리해야 한다. 현재는 진단이며 source 변경/후속 검증 전이다. 공식PDF는 같은 원본 출력과 대응하지 않았고 올바른2020 Print MCP가SaveAsDocument false로 실패해 독립 출력 부분은 미검증으로 남긴다.

### CT: grid 조각과 ordinary-row 내용 조각, 출처 보존
- CS는 #4138 두 건/#7379를 복구했고 기존75검사는53 PASS/22 FAIL(native-skia)였으나 scattered28쪽을81행까지 수용하여29~31쪽을 다시 어긋나게 했다. 첫 후보의 “마지막 컷을 가로지르는 셀이 있어야 프레임이 아래 여백을 소유한다”도 잘못됐다. 걸침 owner가 이번 조각에서 끝나더라도 그 표의 grid 조각은 여전히 중간 물리 프레임이다. 남은 CS 전체 측정은 이 실행 결함 확인 뒤 중단했고 로그/부분 산출을 보존했다.
- CT는 **grid 행/걸침 block 조각과 ordinary row 안의 text-capacity 조각**을 구분한다. grid 중간 조각의 바깥 아래 여백은 독립 본문 경계와 교집합으로 예약한다. ordinary row 내용 컷은 해당 row의 기존 물리 높이/패딩 경로를 사용하며 위 여백만으로 아래 여백 재개를 추론하지 않는다. 캡션 개체의 아래 여백은 마지막 캡션 뒤 상자를 닫으므로 중간 grid 조각에 전달하지 않는다(#7379 독립 소유 계약). #6981의 실제 걸침 block 컷도 같은 grid 프레임 경로로 검증한다.
- #5128은 `source_frame_matches_profile`을 컨테이너 판정 대신 실제 저장 조판 출처 `hwpx_stored_layout`으로 선택한다. HWP5-origin HWPX는 원본 HWP5 인증을 소비한다. 원본 HWP와 직접 HWPX의 기존 계약은 바꾸지 않는다. 후보 구현 후 전체 소유/쪽수 및 정상 대조군을 실행한다. 검사 기대값 변경 없음.

- CT 고정 Native 원본 비교: rowbreak HWPX 18/18 및 scattered HWP 52/52 모두90% 이상. overfill48쪽 중44쪽89.73059%, facility6981의3/4/5쪽56.28537/49.64161/33.12275%로 미달이다. focused native-skia34건은28 PASS/6 FAIL이며 #5128 두 검사와 #4138/#7379를 복구했다. #6981 giant-straddle1건 및 giant-cell 편집/저장5건은 미해결이다. source/command/TSV는 `grid-fragment-source-provenance/`에 고정했다.
- 다음 진단은 같은 컷 서수를 소비한다는 writeback 주장이 실제 live IR/normalized IR에서 성립하는지 실행으로 확인한다. deferred pagination도 begin은normalized/step·finish는live를 소비하는 경로가 있어, 같은 helper라는 이유만으로 공통결과로 판정하지 않는다. 사용자 승인 rowbreak 시각 통과와 새 원본/편집 회귀 판정은 구분한다.

### CV: 지연 그림 소유 경계와 저장 사다리의 역할 분리
- CU 실행에서 giant-cell live/render 문단2507개 및 유닛·anchor 목록이 같았다. IR 정규화 차이가 이 재현의 원인이라는 가정은 기각한다. 저장 전 유닛531=문단314,532=문단315,533=문단310의 그림,534=원본 rewind 문단316이다. 저장본은 새 rewind 문단314 앞에 그림이 옮겨져531=그림,532=문단314로 바뀌었다. 같은 helper를 호출해도 저장 LineSeg 변경 후 소유 순서가 바뀐 실제 증거다(`regression-diagnosis/cu-unit-focus.json`).
- CV는 해당 원본 rewind가 지연 그림의 흐름 순서도 소유한다는 계약을 보존한다. 그 anchor→다음 원본 frame 범위의 text-capacity 컷을 host 사다리로 대체하지 않고 원본 rewind를 남긴다. 같은 셀의 다른 보통 줄 컷은 계속 저장한다. 그림 자신의 유효한 원본 정보가 있으면 다른 문단 편집 여부와 무관하게 같은 physical band 높이를 사용한다. 편집한 그림/합성/무효 source는 기존 독립 certificate에서 기각한다. 새 기대값·fixture 변경 없음; 후보 실행 전이다.

- CV의 정확한 DocumentCore 편집 재현은1자/56자 모두115→115쪽이며15쪽의 그림 유닛 순서/뒤 쪽 귀속을 복구했다. 두 경우 차이는103·104쪽으로 좁혀졌다. 같은 host unit 컷3933에서도 writer가중첩 host2276 뒤 successor2277의 원본 rewind35046을73820으로 바꿔 내부 마지막2줄을 뒤 쪽으로 밀었다. host 한 줄 안 recursive 컷에 대응하는 다음 source frame도 보존해야 한다.
### CW: 표현할 수 없는 recursive 컷과 공통 행 capacity 꼬리
- nested row 안에서 시작하는 컷은 host LineSeg를 새로 열지 않는다. 따라서 host의 다음 원본 rewind를 삭제하지 않고 공유 anchor의 앞/뒤 줄 범위를 보존한다. 일반 Opens 컷은 그대로 되쓴다.
- overfill44쪽 미달은CT 바깥 여백 보정 전CP 후보부터 존재한다. CL과CP 실제 소스 diff는 capacity 마지막 꼬리 간격 면제를 개별 셀에서 인증한 것이 핵심이다. CW는그 면제를 공통 ordinary-row capacity-cut producer가선택한 같은 셀·같은 컷에만 연결한다. 한 셀 마지막0만으로 형제 셀/중첩 조각의 예약을 바꾸지 않는다. 고정CLI의44쪽 actual frame/컷/trace는`regression-diagnosis/*-overfill-p44-tree/`와`cl-cp-clean-table-diff.txt`에 보존했다. 새 테스트조건/fixture 변경 없음; 후보 검증 전이다.

- CW는 recursive successor의 원본 LineSeg를 복원했으나1/56자115→115쪽의103/104 차이는 남았다. 완료 증거가 아니다.
### CX: unchanged empty successor relation
- CW의 unit trace에서103쪽 앞의 같은 원본 빈 줄2279가 무편집/재열기30.93333px, 다른 문단 편집 뒤26.67333px가 됐다. 원본LineSeg/control은 같으며 `stored_empty_successor_advance` 호출부의 session/cell/table 전체dirty 조건이 국소 원본 증명을 무효화했다. CX는 helper 자체가 확인하는 현재/다음 문단의dirty·synthetic·전진/원점 계약을 쓰며 측정/배치가 같은 결과를 소비한다. 실제 편집된 빈 줄/후속 문단의 무효 정보는 helper에서 기각한다.
- CP capacity-tail의 새 간격 면제에서nested child 내부의local cut은physical page 종단이 아니다. 기존page-reset trim과 같은 좌표 계약으로 parent frame이 소유한 간격을 보존한다. 독립 인증된 기존source 경계는 유지한다. 후보 실행 전, 기대값 변경 없음.

### CY: 변경되지 않은 자식 행의 저장 경계
- CX는 overfill44쪽의 세 표 상자를 CL과 같은 높이(999.8/992.5/988.7px)로 복구했다. giant 편집/저장은0·1·56·61자115쪽 및200자116쪽을 유지하지만103/104쪽 귀속 차이는 남는다.
- 새 유닛 대조에서 동일한 중첩 표2280의 행9는 무편집/재열기에서 두 저장 조각(43.6/28.45333px), 다른 문단을 편집한 세션에서 한 행(49.78667px)으로 투영됐다. `stored_joint_zero_origin_boundary`와 `stored_plain_line_restart`의 전역 session_edited가 변경되지 않은 자식의 원본 경계를 기각했다. CY는 전역 세션 상태를 제거하고 행의 모든 peer 원본·유효 줄 및 해당 셀/표의 실제 재조판 상태를 계속 확인한다. source → 공통 row-cut producer → recursive projection/paint가 같은 경계를 소비하는지 검사한다. 기대값 변경 없음.

- CX overfill 원본48쪽 Native 전쪽 TSV는 최저93.03871%,90% 미달0쪽으로44쪽 미달을 복구했다(`local-empty-successor-contract/sweeps/overfill-hwpx/native-silhouette/overfill-hwpx/silhouette.tsv`). CY 재검사 및 fresh Docker WASM은 아직 실행 전/진행 중이며 이를 최종 gate로 승격하지 않는다.
- CY 정확한 DocumentCore 편집 재현: 빈 문자열·1·56·61자115→115쪽,200자116→116쪽. 각 재현에서 **모든 쪽의 단어/순서가 저장 전후 동일**하다. `regression-diagnosis/cy-exact-edit-hwp-{0,1,56,61,200}/`에 원문 IR·저장본·쪽 소유/본문 증거를 보존했다. 기존34 native-skia 검사와 원본11문서393쪽 Native 측정은 실행 중이다.

### CZ: deferred pagination의 정규화 입력 일치
- CY 기존34검사는32 PASS/2 FAIL로 #7114 여섯 경계 및 HWP 편집/저장을 복구했다. 잔여 #2185 HWPX 편집 후124→115 기대 차이와 #6981 걸침 셀 외부 A9~10 줄은 실제 미해결이다.
- `begin_deferred_pagination`은 정규화 문단과 거기서 측정한 표를 사용하지만 `step`/`finish`는 live 문단을 전달했다. CZ는 begin과 같은 source revision의 `section_render_paragraphs`를 이후 단계에도 전달한다. dirty revision/stale 판정과 live 편집 정보는 그대로 유지하며, 측정과 컷/배치의 입력 도메인을 일치시킨다. 기존 #2185에서 수정 전 HWPX 편집124쪽 실패를 재현했으므로 수정 뒤 같은 검사를 재실행한다. 새 기대값 변경 없음.

### DA: 국소 source 경계의 closing advance
- CZ의 정규화 입력 보정만으로는 HWPX 편집124쪽이 복구되지 않았다. 동일 입력으로 일반·deferred가 모두124쪽이며 저장 재열기는115쪽이다. 보정 완료 주장을 하지 않는다.
- 공개 giant HWPX의 원본·편집 CellUnit4361개를 비교하면 각 줄의 높이/순서는 동일하며 문단82의 frame 표지 하나만 달랐다. 9쪽에서 같은 start309의 끝이 편집347/저장348로 갈라지고 다음 쪽은347..348 한 줄 조각이 된다. `stored_page_reset_cut_trailing_trim`가 전역 편집 상태로 미변경 source 줄의 closing advance까지 기각했다.
- DA는 이 helper와 body 폐합 인증에서 전체 세션/셀 dirty를 원본 줄 쌍 유효성의 대용으로 쓰지 않는다. 현재 closing/successor 문단의 실제 dirty·format·합성 줄과 물리/중첩/rowspan 좌표 계약은 계속 확인한다. 저장된 다음 frame의 advance와 현 페이지 잉크 점유를 구분하며 scan·paint가 같은 trim을 소비한다. 기존 #2185 및 source/편집 반례 재실행 전 후보다.

- DA에서 단일 문자 HWP/HWPX의 일반·deferred는115→115쪽 및 전쪽 내용 순서가 같았으나 기존34 검사에서30 PASS/4 FAIL로56·61·200자 경계가 새로 실패했다. 두 변경을 일괄 성공으로 취급하지 않는다. source body 폐합은 선택된 fragment의 시작 컷을 반영하지 않는 원본 좌표 인증이므로 국소 줄 쌍만으로 전역 배제를 제거한 변경을 되돌리고, closing advance 변경을 별도로 검사한다.

- CY 고정 후보 원본11문서393쪽 Native 전쪽 TSV: 모든쪽90% 이상, 누락/쪽수 차이 없음. 최저 rowbreakHWPX90.85014%, nestedHWP/HWPX91.14637%, overfill93.03871%, scattered93.25935%. 원문/집계는 `local-child-source-boundary/native-silhouette-scores-results.json`과`native-aggregate.json`이다. 후속DB 코드와fresh Docker WASM 결과로 대신하지 않는다.

### DB/DC: closing advance 분리와 실제 걸침 내용 커서
- DB는 source body 폐합의 세션/셀dirty 배제를 복원하고 유효 closing/successor 줄 쌍의 advance 보정만 유지했다. 정확한 HWPX 일반·deferred 1자115→115쪽, HWP56/61자115→115쪽 및200자116→116쪽, 각각 전쪽 본문 순서가 저장 전후 같다. `local-source-tail-advance/candidate/`에CLI·patch·출처를 고정했다. 전체 회귀/freshWASM 판정은 별도다.
- #6981 facility 원본4쪽의 실제 컷36..70은906.67px를 소비하지만 해당 칸 조각은855.5px다. 시작을 저장reset으로 아래로 맞춘 뒤 끝은 여전히 원래 물리prefix에서 추정한 것이 원인이다. 끝만 줄이면 다음 조각은 원래prefix에서 다시 시작해 내용이 누락될 수 있다. clipping/좌표clamp/예상값 완화로 처리하지 않는다.
- DC는 `StraddleCellCarry`에 물리 빈띠·다음 내용유닛·현재 수용 범위를 서로 다른 필드로 보존한다. `ordinary_rowbreak_fragment_heights`가 기존ordinary-row 컷·상하패딩·시작/끝override·rowspan 끝행 요구를 공통 계산한다. `emit → accepted_straddle_fragment_units → PartialTable의fragment_cut → table_partial의cell_line_ranges`가 같은 유닛 범위를 소비하고, 다음scan의`straddle_continuation_demand`는 전달된next_unit에서 시작한다. block cut은 기존 독립 인덱스 계약을 유지한다. 새 테스트/golden/기대값은 추가하지 않았으며 실행 대조 전 후보다.

- DC 기존focused34 NativeSkia **34 PASS/0 FAIL**. 기존75 integration 재검사 **60 PASS/15 FAIL**다. 전체 전수 검사는 아직 실행 전이다. facility는5쪽245줄에서escaped0이며 이전243줄 중 누락0/추가2다. 추가A14·B1~2는원본IR의cell22/para70·71에서확인했고4개 원본 소유가 모두각각 표시된다. 독립PDF3~5쪽배치미달은별개로남으며gate완료로기록하지않는다.
- 신규내용carry는ordinary RowBreak/CellBreak 조각에만적용한다. block cut은기존블록별절대유닛부기가내용을소유하고새ordinary창을만들지않는다. 두컷도메인을한높이로대체하지않는다. source원본전체/예산반례확인후필요한bridge만수정한다.


### DD 전쪽 완료와 DE/DF 회귀 보정

- 작업지시자는 Docker WASM Studio `http://localhost:7794/?review=pr7704-rowbreak-all-pages`의 HWPX 4·6·7·8·15쪽을 **시각 판정 통과**로 판정했다. 기존 16쪽 판정과 함께 보존하며 다른 문서의 예외 승인으로 확대하지 않는다.
- DD `rowbreak-final-validation/candidate/source.patch` SHA-256 `f6aa665e7ee5f4984d6819c2f853598e7c9c74b0adebf471a69317d0992c80f3`, CLI `107e4e759d780eca76324103094160f36bea354268020cec16a13ede9d4cedaf`, Docker WASM `7dce391b73591605485a27b588469d945fc02b731b6b5c178ce748f83b85e7cb`의 원본 11문서/393쪽 Native·WASM 전쪽 TSV가 완료됐다. 두 backend의 문서별 최저값은 동일하며 rowbreak HWP 93.62865%, HWPX 90.85014%, scattered HWP 93.25935%, overfill 93.03871%, nested HWP/HWPX 각 91.14637%, cellimg HWP/HWPX 각 96.87527%, vpos HWP/HWPX 99.43600/99.43614%, pictoggle 99.95151%다. 전쪽 누락·90% 미달·쪽수 불일치 없음. 증적은 ignored `output/pr-review/oksure-20261011/pr7704-maintainer/rowbreak-final-validation/` 아래 원문 TSV/manifest/로그다.
- DD 전체 nextest는 10,220건 중 10,198 통과·22 실패·45 skip이다. 실제 내용 중복·페이지 증가·anomaly 증가, 기존 검사의 잘못된 동일 조각 가정과 수정된 overfill 결함에 의존하는 CLI 검사를 구분해 조사하고 있으며 기대값/baseline을 완화하지 않았다.
- DE: spanning cell이 자신의 첫 행 안에서 이미 두 문단을 소비했지만 같은 행에서 재개하는 경우, `straddle_page_bands_after_fragment`가 `cell.row < next_row`만으로 원장을 삭제했다. 입력 컷의 명시적 `fragment_cut/next_unit`을 소유한 같은 시작 행도 원장을 이어받도록 수정했다. 생산된 accepted cut → 물리 band/next_unit 이월 → `rowbreak_straddle_cut_units` → partial paint의 같은 내용 커서다. 아직 손대지 않은 미래 셀은 이월하지 않는다. 기존 #6803 검사 수정 없이 DD 두 실패 → DE 세 통과. 실제 root pi6 표의 5쪽 전수에서 문단 0..66이 정확히 한 번씩 출력됨을 별도 RenderTree로 확인했다(`own-row-straddle-content-cursor/6803-all-pages-paragraph-ownership.json`). DE 전체 시각·필수 검사는 미완료다.
- DF 구현 전 독립 근거: #2097 Print 1쪽은 「마) 에너지 공급」 첫 줄을 소유하고 2쪽에 나머지를 조판한다. 자연 저장 같은 문단의 연속 줄에서 마지막 줄의 ink와 다음 줄을 전진시키는 spacing을 구분해 컷 선택·높이 예약·paint가 동일한 trailing advance를 소비하도록 보정한다. #5585 Print 14→15쪽은 선언 행의 빈 tail을 이어받는다. 조사 초기의 global blank-band discard 가설과 달리, 새 `source_complete_frame_last_row`가 첫 조각 안에서 닫히는 행을 전체 물리 행처럼 짧게 대체한 실제 경로를 확인했다. 전체 grid 종료만 그 마지막 unused band를 닫을 수 있고, 중간 첫-fragment frame은 capacity budget으로만 소비해 scanner가 행의 나머지 공간을 이어받도록 수정한다. 이 두 변경의 실행 판정은 아직 진행 중이다.
- EMPTY: 같은 셀의 연속 두 문단이 모두 local vpos=0인데 뒤 양수 vpos를 글로벌 successor advance로 수용해 빈 줄 상자가 용지 밖으로 나갔다. visible image는 정상이어도 실제 빈 줄 공간 계약은 별도로 지켜야 한다. 반복 local zero 뒤의 양수 delta를 shared `stored_empty_successor_advance`의 인증에서 제외하며, 실제 positive→zero page restart와 인증된 coincident empty/text slot은 유지한다. 코드 변경 후 원본 Print 전쪽 및 anomaly 재검증을 진행한다.
- task1716 scattered HWPX 22쪽은 DD 63.82847%로 미달이다. 같은 원본/Print를 고정 base bfa CLI에서 다시 출력한 대표 비교도 54.28257%로 미달이었다(`scattered-hwpx-regression-baseline/`). 이는 현재 후보의 해당 페이지 전체 개선 완료 증거가 아니며, 관련 anomaly 증가도 baseline 갱신으로 감추지 않는다.


### DG–DK 경계 회귀 진단

- DG `capacity-restart-full-suffix/candidate/provenance.json`의 자연 저장 local restart 컷은 한 줄짜리 suffix 가정을 제거했다. #2097은 3→2쪽으로 복원되었고 #6803 기존 3검사는 계속 통과했다. #5585 17쪽 Native 전쪽 최저 96.62601%, EMPTY 2쪽 99.78553%; 해당 stage의 fresh WASM은 미완료다.
- #2097 1쪽은 같은 Print의 고정 base bfa 전쪽 Native 50.31610%, DG 49.67213%로 이미 기준 미달이다. 2쪽은 양쪽 98.73484%. 복원된 쪽수와 전체 시각 gate 통과를 구분하며 기대값을 변경하지 않았다.
- DH는 HeightCursor의 문단 앞 간격 보정을 자리차지 표 호스트에서 제외해 보았으나 6문서의 실제 anomaly가 변하지 않았다. 이 분기는 되돌렸다. DI는 임시 flow 진단으로 실제 배치 원점과 저장 계획을 추적한 실행본이다.
- DJ `column-owner-and-prefix-end/` 실행: hwpspec의 새 텍스트 겹침 5→0, 하단 넘침 0. 원인은 측정 원점이 아닌 열 시작 전용 `query_captioned_column_rowbreak_placement`가 열 중간에서도 `st.current_height`로 표 계획을 생성해 실제 저장 앵커를 덮어쓴 경로였다. 열 시작에서만 이 계획을 생성하도록 수정했다. 일반 자리차지 배치는 기존 호스트 앵커를 사용한다. neartop의 저장 prefix 끝 검증만으로는 실제 용지 밖 출력이 해소되지 않았다.
- DK 구현 전 확인: HWP3 sample16의 문단 742/746/749/751은 측정 `flow_advance=17.3`, 실제 배치 `34.45`로 갈렸다. 재구성된 사다리의 spacing-before 누락 경계에서는 HeightCursor가 저장 절대좌표 복원을 중단하지만 `spacing_trim_restorable`은 복원 가능으로 판정해 앞 간격과 마지막 줄간격을 함께 덜었다. 동일 source→rebuilt 경계 판정을 공유하고 그 경계에서 전량 흐름을 예약하도록 보정 중이다. 최종 실행 판정은 아직 미완료다.
- 중첩 rowspan external 입력의 8쪽은 r1c2의 모든 문단을 소유한 채 행 상자가 304.6→257.7px로 짧아져 다음 셀과 겹쳤다. 전체 내용과 물리 행 밴드의 소비 위치를 추적 중이며 제한 좌표/clamp나 baseline을 추가하지 않는다. 독립 MCP Print `f7feaf11-ee98-4108-9ce4-50fd0bebc182`는 preprocess none / driver one-up / method 0 / 70쪽으로 생성됐다. 입력 72쪽 후보와 쪽수 차이는 별도 미검증으로 보존한다.
- DD 393쪽 Native/fresh WASM 자료를 코드가 달라진 DG–DK의 최종 증거로 재사용하지 않는다. 기존 기대값·fixture·golden은 변경하지 않았으며, 현재 코드는 원격에 반영하지 않았다.

### DL–DO: 물리 예약과 저장 좌표 축의 경계

- DL은 자식 표의 측정 캐시가 없는 partial 경로에서도 온전히 포함된 rowspan 행의 물리 높이를 보존했다. external 8쪽의 새 텍스트 겹침 5→0이며, 공개 MCP Print는 같은 병합 셀의 외곽 높이를 독립적으로 증언한다. DK의 source-spacing 경계 보정만으로는 HWP3/neartop를 해결하지 못했다. DL 원본 rowbreak HWP/HWPX 18쪽씩·overfill 48쪽 Native 전쪽 TSV 최저값은 각각 93.62865%,90.85014%,93.03871%로 모두 통과했다. fresh WASM은 이 stage에 대해 미실행이다.
- DM은 전체 표 프레임과 선언 행 관계를 확인하던 예약 조건을 제거했지만, 남은 anomaly가 줄지 않았고 rowbreak HWP 2쪽이 미달(최저47.89862%)했다. DM 변경은 되돌렸다. `whole-row-physical-reservation/`의 실패 증거를 보존하며 배치가 쓰는 행 높이가 모든 원본 컷의 예산을 대체할 수 있다는 가정을 수용하지 않는다.
- DN은 첫 source frame의 끝 행 override를 바꿀 때 `consumed`도 같은 물리 높이로 바꿨다. 원본 negative 표는 행 내용68.73333px→source frame75.17333px인데 이전 예약은68.73333px 그대로여서 emit가 남은 밴드를 중복 가산했다. 기존21쪽·내용 소유를 유지하며 새 하단 넘침1→0이다. scanner 결과 → emit의 partial_height/끝 밴드 → 마지막 행 override/straddle ledger → partial paint에 같은 높이를 전달한다.
- DN의 일반 opening HC 스냅 실험은 scattered의 새 넘침3→0이었지만 rowbreak HWP/HWPX가 각각18→19쪽, external72→73쪽으로 변했다. 이 스냅 실험은 되돌렸고 source-frame 소비 보정만 유지한다. 기존 테스트/baseline/fixture는 바꾸지 않았다.
- DO 구현 근거: HWP3 6쪽의 첫 호스트141은 이전 원본 쪽 좌표를 갖고 후속142의 원점은 되감긴다. paint는142를 처리한 뒤 prev/page/lazy 축을 소거하지만 pagination은72548HU base를 뒤 문단에도 계속 적용했다. 후속 문단의 앞·뒤 간격이 측정에서만 덜어져 TAC 표152의 확정 원점이 실제 앞 제목151과 겹쳤다. `paragraph_origin_rewinds`를 paint와 `finish_paragraph_anchor_state`가 같이 사용해 같은 owner 뒤 축을 끊는다. 일반 첫 RowBreak 조각도 예산이 이미 수용한 flow origin을 확정 placement로 전달하고, 독립 object-frame 조회가 있는 기존 계획을 우선한다. 현재 DO Native 빌드/대조 검증 전으로 완료 주장을 하지 않는다.


### DO 실행 결과와 DP 예약/좌표 보정

- DO에서 HWP3 새 텍스트 겹침 2→0, negative/scattered 새 하단 넘침은 0이었다. 그러나 일반 opening fragment의 flow-origin placement 추가는 rowbreak HWPX 11쪽 일치율을 86.45555%로 낮췄다. 이 일반 fallback은 철회했다. 저장 좌표 축을 paint와 같은 owner 뒤에서 종료하는 전환과 source-frame consumed 보정만 보존한다.
- DP는 rowspan이 컷을 가로지른다는 이유만으로 마지막 행을 남은 페이지 전체까지 늘리는 `accepted_straddle_end_band`를 제거한다. 공통 컷의 행 상자는 원래 수용한 물리 높이를 소비하며, 남은 빈띠는 기존 continuation ledger의 `avail + header - partial - margin`으로 따로 이어받는다. 이는 내용 유닛을 추가 수용하는 근거가 아니며 기존 분할 경계 검사로 검증한다.
- DP에서 page/lazy 축이 모두 없는 경우에도 spacing-before형 트림을 허용하던 호출을 수정했다. 계상값 → `flow_advance_height`/`flow_trimmed_spacing_before` → 페이지 current-height/뒤 문단 fit → paint 순차 원점에 같은 전량 간격을 전달한다. 앞 간격을 복원할 저장 page base가 없는 상태를 lazy-base가 있는 상태와 같은 계약으로 처리한다.
- DP Native 후보 patch SHA-256 `553ef6a6b58eededd6948bf63e9791ebad9de5fd097ef953c4cb4e7cac93005a`, CLI `5b9f418d1cf98c494e40d178a0ce97477df16a098d005e2ede698df996c3182e` (`reserved-row-frame-and-unbased-flow/candidate/provenance.json`). 6문서 anomaly 대조에서 negative/scattered/hwpspec/HWP3는 새 off-canvas·겹침·하단 넘침 0이다. external은 base와 같은72쪽·off-canvas3·하단3. neartop의 새 off-canvas2→0이나 base205쪽 대비206쪽과 하단 신호 증가 13→14는 남아 있으므로 완료로 판정하지 않는다.
- DP rowbreak HWP/HWPX Native 전쪽은 각각18쪽이며 최저93.62865%/90.85014%로 복구됐다. overfill 및 원본 나머지8문서/309쪽 전쪽, 기존34경계 검사와 fresh Docker WASM은 진행 중이다. 기존 검사·fixture·golden·baseline은 변경하지 않았다.


### 원격 devel 병합 확인과 DQ 후보

- 작업지시자의 원격 devel 병합 후 진행 요청에 따라 `git fetch upstream devel`과 `git merge upstream/devel`을 실행했다. 최신 원격은 `bfa28a77a2714482f647ee6a7b9aa7f7876d8404`이며 기존 `1990ae5c1` 병합을 통해 현재 HEAD `1563c48b3a2981992c70e958892543fe4e033a93`에 이미 포함되어 있다. ancestor 검사 성공, 병합 결과 `Already up to date`; 기존 미커밋 보정과 검증 증적을 보존했다.
- DP 원본 나머지8문서/309쪽 측정은 완료됐다. nested HWP/HWPX 각115쪽 최저91.14637% 및 나머지 소형 문서는90% 이상이나 scattered HWP 7쪽69.98213%,46쪽74.89956%로 미달이다. 마지막 행의 끝 빈띠를 일괄 제거하면 걸침 셀의 내용 컷도 바뀌므로 DP의 제거를 완료된 규칙 보정으로 수용하지 않는다. `reserved-row-frame-and-unbased-flow/remaining-original-validation/`에 원문과 실패 증거를 보존했다.
- DP 기존34경계 검사는30 통과/4 실패(#7379)였다. DQ는 내용 소비 없이 이월한 첫 조각에서 이전 host frame의 원점을 재사용하지 않고, 새 단 시작의 실제 캡션 배치 조회를 확정 결과로 전달한다. 별도로 저장 successor endpoint를 절대 좌표로 투영할 때 유효한 page base 없이0을 가정하던 경로를 제거했다. 생산한 확정 placement→fragment 예산→PartialTable 실제 원점 및 저장 endpoint→뒤 문단 flow 소비를 각각 대조한다.
- DQ patch SHA-256 `504bffaf241831a48caa3a267061a5cd353dc661895e9f506e99b1812309ddb8`, CLI `8100d6aeb0d85dbd5184031c6cc8e1ff3e1359dae18bc21aea66014d1f78beba`로 고정했다(`frame-owner-and-source-axis/candidate/provenance.json`). Native 6문서 anomaly에서 negative/scattered/spec/HWP3 새 off-canvas·겹침·하단 넘침0, external은 base와 같은72쪽/off-canvas3/하단3이다. neartop는 여전히 base205쪽 대비206쪽·하단13→14여서 미해결이다. 기존34경계 검사와84쪽 Native 전쪽 대조는 실행 중이며 fresh Docker WASM/최종 전수 검증은 미완료다. 검사 조건·baseline·fixture 변경 없음.

- DQ 경계 재실행은 **31 통과/3 실패**다. `unproven_empty_picture_table_frame_keeps_measured_flow_origin`은 복구됐으나 캡션 단일 행의 첫 조각/끝 조각/양수 오프셋 이월은 남아 있다. 진단 실행에서 변형 표가 fragment scanner에 진입하지 않고 whole-fit 경로로 수용됨을 확인했다(`frame-owner-and-source-axis/logs/caption-opening-trace.log`, `TABLE_DRIFT pi=728` / 실제 owner 쪽 출력). 첫 캡션 placement 갱신만으로 이 whole-fit 수용 결함을 해결했다고 주장하지 않는다. DQ rowbreak HWP·HWPX 전쪽 Native 비교는18쪽씩/90% 이상이며 overfill 비교는 진행 중이다.

- DQ Native84쪽(18+18+48) 전쪽 비교 완료: rowbreak HWP93.62865%, HWPX90.85014%, overfill93.03871%, 미달/누락/쪽수 차이 없음. 최신 원격 devel 기준 확인은 완료했으며 최종 merge 준비는 미해결 경계와 전체 재검증 후 판정한다.

### DR: 통째 수용의 캡션 종료 프레임

- 구현 전 DQ 실패 trace에서 변형된 단일 행 `effective_height=468.9`와 캡션/종료 여백이 현재 쪽에 들어가지 않는데도 선언 행 상자만으로 whole-fit에 진입했다. scanner의 마지막 유닛/캡션 예약 전에 조기 수용된 경로다.
- `query_captioned_rowbreak_frame`은 기존 열 시작 producer와 같은 캡션/종료 여백 계약을 사용하며, 열 중간에서는 이미 인증한 표 원점을 보존한다. `entry`의 whole-fit는 그 occupied_bottom을 예산으로 판단하고, 성공 시 같은 placement를 paint에 전달한다. 원점·끝점을 각각 추측하는 대체 fit 조건을 추가하지 않는다. 절대/음수/가운데·아래 앵커와 캡션 없는 표는 기존 경로다. 기존 경계34 및 실제 문서로 검증 전이며 기대값 변경 없음.

- DR 34검사는 통과했으나 hwpspec55·59쪽에서 새 텍스트 겹침2건을 확인했다. 열 중간에서 원점 계획이 없다는 이유로 순차 cursor를 곧바로 캡션 원점으로 사용한 것이 원인 후보다. source HC의 실제 owner 조회가 없는 이 fallback을 검증 완료로 취급하지 않는다.

### DS: 첫 조각 원점과 rowspan 끝 빈띠의 동일 예산

- 독립 scattered HWP Print 7·46쪽과 DD/DP 직접 비교에서 빈띠를 삭제한 DP는 앞 조각의 걸침 내용 한 줄을 뒤로 넘기고 이후 행들을 약4px 내렸다. 빈띠는 행 컷이 표현하지 못하는 물리 공간이지만 걸침 셀은 그 공간을 실제로 소유한다. 정확한 조각 원점에서 빈띠를 계상하도록 복원한다.
- HWPX32쪽의 기존 하단 넘침은 budget flow827.626px와 paint HC원점835.907px(+8.28px)이 달랐다. HC조회는 직전 owner의 cursor를 읽기만 하고 확정 anchor를 반환한다. 저장 page/lazy 축·유효한 원본 host가 없는 재조판은 기존 흐름을 사용한다. 기존 문단 snap과 같은 cursor 구성 helper를 사용한다. 통째 캡션 상자도 기존 paint의 원점 조회 결과를 보존한다.
- 확정 origin→page_avail→scan 컷/consumed→끝 행의 물리 band 예약→accepted_straddle_fragment_units→PartialTable 끝 override·내용 cursor→paint의 같은 origin/행 상자를 연결한다. 이월할 physical band는 다음 ledger에 따로 누적한다. HC 상태를 새로운 문단 상태로 바꿔버린 DN 실험과 구분한다. 기대값·baseline·fixture 변경 없이 검증 전이다. 첫 적용 스크립트가 빈띠 복원 전 중단되어 그 부분본 빌드는 종료했고, 이를 실행 결과로 세지 않았다.

- DS 기존 경계34 PASS. Native patch `ce8adf077fa8610c1fbb0ef8c769172876035ac524caebafba85854f06492a8e`, CLI `00c3746a5de30226b5c73579dfd4bad640baf4debc4be0ddcc0071c0811f17fd`를 고정했다. Native136쪽 및6문서 anomaly 재실행 중으로 최종 시각/회귀 통과를 아직 선언하지 않는다.

### DT: 저장 되감김 경계의 좌표계

- neartop 원본 구역4 문단97은 저장vpos55596/줄높이13808/간격660HU이고 후속98은 쪽 상단으로 되감긴다. 현재 lazy base26626HU로 투영하면 직전 줄 끝은579.17px로 실제 occupied575.2px 부근이지만 기존 helper는 원시934.19px를 바로 비교해 그 쪽 경계를 기각했다. 독립 Print는20쪽 끝 표3과21쪽 시작 표4를 별도 쪽에 둔다.
- WholeFitPage가 선택한 source page/lazy 기준과 동일한 열 anchor로 직전 저장 줄 끝을 변환한 뒤 실제 흐름과 비교한다. source축이 없으면 위치 일치를 주장하지 않는다. 원점 생산→문단 whole-fit 소유 경계→새 쪽 전환→paint의 같은 page/lazy 계약을 검사한다. 컷/페이지 기대값을 고정하거나 빈 줄을 숨기지 않는다. 새 검사·기준값 변경 없이 실행 전 후보다.

### DU: 부동 표 원점과 글줄 소유의 구분

- DS는34경계 검사 통과와 별개로 rowbreak HWP/HWPX18→19쪽, hwpspec 새off-canvas6/겹침5/하단7, neartop205→203쪽·겹침6→109를 만들었다. 실패 증거를 보존하고 완료로 수용하지 않는다.
- 코드 대조 결과 HC조회가 모든 Table에 대해 부동 anchor를 만들었다. TAC는 글줄 소속인데도 NextLine placement에 넣었고, query가 캡션 producer에서 기각돼도 그 임시 placement가 남았다. DU는 문단 기준 자리차지·Top/Inside 부동 표 원점 계약에 이 producer를 적용하며 TAC/절대/가운데·아래 개체는 해당 줄/개체의 기존 좌표 경로로 돌려준다. 문서/수치 예외를 더한 것이 아니라 혼합한 좌표 도메인을 분리한다. DT의 source 되감김 좌표계 보정은 함께 대조한다.

- DU는34경계 PASS였지만 실제 문서의 회귀를 해소하지 못했다. scattered HWPX52→54쪽/HWP3 64→66쪽, neartop205쪽이지만 새겹침105건 등이다. source축이 있다고 현재 host가 같은 원본 프레임에 속한다고 단정한 가정이 잘못됐다. 결과는 `float-owner-and-rewind-axis/`에 보존한다.

### DV: 직전 저장 owner의 폐합 근거

- 부동 원점 조회는 직전 실제 text owner가 현재 host의 원본 LineSeg를 선언 문단 여백과 정확히 연결할 때만 수행한다. source/rebuilt 좌표가 다르면 기각한다. scattered HWPX의40끝60773+1200+720=41원점62693HU가 독립 원본의 폐합 증거다. TAC/이전 표/합성/다른 source frame은 기존 owner 경로를 유지한다.
- 캡션 전체 상자가 선언 행만의 fit을 기각하는 계약과, 실제 원점을 대체하는 계약을 구분한다. 미인증 열 중간 origin을 출력에 강제하지 않고 기존 source/paint 앵커를 보존하며, 인증한 경우에만 같은 전체 frame을 전달한다. 임시 source anchor는 실제 캡션 frame producer가 적용되는 경로에서만 만들고, producer에서 기각된 다른 방향/비분할 표의 anchor는 남기지 않는다. Native/경계 검사 재실행 전 후보다.

- DV는 off-canvas 증가를 해소했지만 원본의 페이지 수 증가와 spec의 새 겹침2건은 남았다.54/66/73/207쪽을 성공으로 판정하지 않는다.

### DW: 실제 인증한 source축만 경계 판정에 사용

- Whole-fit의 `current_page_vpos_base`에는 실제HC축이 소거된 뒤에도 첫PageItem의 옛host vpos를 빌려오는 fallback이 있다. 이는 saved bounds의 보조 기준이며 새 source frame 인증이 아니다. 경계 위치 비교에는 `st.vpos_page_base/vpos_lazy_base`의 실제 유효 축만 전달하고, 축이 없으면 기존 채움/소유 규칙을 유지한다.
- 통째 캡션 원점의 새 임시 placeholder를 제거한다. 기존 인증 원점 또는 실제 새 단에서 생성한 프레임만 확정 배치를 소유한다. 선언 행만으로 캡션 종료 공간을 수용하는 것은 별도 complete-caption fit에서 기각한다. 임시 origin이 producer를 통과하지 않아도 남는 분기는 쓰지 않는다. 원본 페이지 수·겹침·하단 및34경계를 다시 대조한다.

### DX: 문단 소유 원점에서 투영한 후행 상자

- DR 이후의 표 HC조회와 DT/DW 되감김 축 변경은 실제 문서의 페이지 증가를 해소하지 못했다. 해당 시험 변경을 DR 원본 소스로 되돌렸다. 각 후보patch/CLI/실패로그는 보존한다. 캡션 완전 상자의 수용 판정과 걸침 셀의 물리 끝 밴드는 유지한다.
- 근본 원인 추적: 기존 `stored_paragraph_successor_flow`가 원본 글줄 끝+간격+뒤 여백과 다음 빈 표 host의 원점을 이미 인증하지만, page/lazy축이 없는 `project`는 이를 기각했다. paint의HC는 다시 그 간격을 복원해 측정과 배치가 갈렸다. Source축의0을 가정한 옛 동작도 잘못된 절대끝을 적용했다(#7379).
- DX는 해당 문단의 수용된 실제 원점에 인증한 local line-box 높이와 간격을 투영한다. 기존 절대축이 있으면 기존 투영을 유지한다. 인증 producer의 원본/편집/합성/폐합 조건과 format의 실제 줄 수 판정을 바꾸지 않는다. 결과→paragraph flow end/content bottom→typeset 누적 소비→paint의 stored_paragraph_flow_ends를 같은 결과로 연결한다. 표 원점의 별도 추정/clamp나 기대값 변경은 추가하지 않았다. DX 실행 전이다.

### DY: 온전한 자리차지 표의 column source origin

- DX는 HWPX 새 하단 넘침3→1로 줄었으나 #7379의 미인증 원점 반례를 다시 실패시켰다(380.653/367.32). typeset의현재 cursor를 paint의 문단 원점으로 가정한 relative 투영을 철회했다. 원점 조회가 없는 경우는 DR의 `project` 기각을 유지한다.
- 실제 trace에서32쪽 첫항목은 PartialTable이 아니라 온전한 문단 기준 자리차지 Table36(source0)이다. paint 초기 page base0은 온전한 float 뒤에도 유지되지만 `commit_deferred_table_anchor`는 이를 지웠다. 그 뒤 빈37→본문38에서 typeset는lazy621HU,paint는page0을 사용해8.28px 차이가 누적됐다.
- DY는 같은 `stored_table_column_origin`을 column 초기 paint와 typeset 첫 온전한 표 완료에서 소비한다. 실제 paint처럼 문단 기준 자리차지 whole table은 기존 축을 유지하고, PartialTable/TAC/다른 개체의 기존 reset은 유지한다. 원점 생산→whole 표 완료 transition→후속 paragraph/source flow end→fragment budget→paint의 동일 첫column원점을 대조한다. 캡션 complete fit과 source 끝 band 소비는 유지하며 신규 기대값/fixture 변경 없음.

- DY 기존34 PASS. 다만 실제32쪽의Table36은 지연 큐가 아닌 일반 배치 경로라 `commit_deferred_table_anchor`에 도달하지 않는다. helper존재를 실제 경로 공통화 증거로 삼지 않는다. DY 원문136쪽 측정과6문서 anomaly는 별도로 보존한다.

### DZ: 일반 whole-table 완료 경로 연결

- `seed_whole_table_column_origin`을 지연 완료와 일반 `finish_paragraph_anchor_state`의 온전한 문단 기준 자리차지 표 완료에 함께 연결한다. 첫SourceTable의 source/host/acceptedplacement→같은helper의PageOrigin→다음empty37/body38의동일HC축→paragraph40 sourceend→Table41 예산을 실제trace로 재확인한다. Source태그의원본/합성 여부도측정origin provenance에보존한다. Rawsource0을전역default로사용하지않으며이함수의첫Table소유원점으로만사용한다. 실행전후보고와393쪽/회귀검증은아직완료하지않았다.

### EA: TAC 완료 뒤 저장 좌표 축 종료

- DZ 첫 whole table의 source origin 복원으로 scattered HWPX 새 넘침3→0/52쪽이 유지됐다. 다만 rowbreak HWPX11쪽 마지막 표가6.6667px 올라가 전쪽 최저84.99889%로 미달했다. 앞 Table2의 source500HU 기준을 TAC Table3 뒤에서도 측정만 유지하고 paint는 소거한 실제 경로 차이다.
- `host_line_covers_object`의 높이 포함 검사는 TAC 배치 뒤 원점 보존의 근거가 아니다. 실제 `layout.rs`의 `was_tac` 완료 전환과 같이 TAC Table을 그 조건에서 제외한다. 별도 확정 inline 끝점·stored wrap/source fragment·문단 기준 float 전환은 유지하며, 문서 번호/좌표 예외를 추가하지 않는다. 현재 실행 전이며 기존 기대값·fixture는 변경하지 않았다.

- EA 첫 검증에서는11쪽84.99889%가 그대로였다. 같은 TAC가 `inline_placements.advance_end`로도 축 유지에 진입했기 때문이다. 확정 흐름 끝과 후속 문단 source축 인증을 구분해 이 유지 조건도 paint의 TAC 완료 전환과 일치시킨다. EA의6문서 새 신호는 DZ와 같으며 새 기대값을 추가하지 않았다.

### EB–EC: 저장된 첫 프레임의 빈 밴드 소비와 이어받기

- EB는TAC `advance_end` 유지까지종료해 rowbreak HWP/HWPX36쪽 모두Native90%이상복구했다. source_patch`12a9d8308fa35eee93481d2828806ededfa328218b7a0bdd6f79894d524eb576`,CLI`e03b20031cce6376cbffb113f2028e4ca9c6509a456bb9e2e9941e9b25d7371e` (`tac-flow-end-axis-closure/`).
- scattered HWP45쪽Table59의 sourceFrame은331.053px,원래내용행합323.12px다. DN에서reserved consumed를331.053으로바르게바꾸었으나그차이의빈밴드를이어받는걸침셀ledger에전달하지않아46쪽끝행26의높이가3.133px커졌다. DD는앞paint에서밴드를이중증가시키면서그ledger를우연히유지한상태였다.
- EC는scan의`source_frame_band`로실제수용sourceFrame−내용소비양을기록한다. consumed/override는같은331.053px유지,emit는그밴드를paint에다시더하지않고straddlePageBands의physicalHeight에만전달한다. refit로다른컷을수용하면그refit의소비기록으로대체한다. 내용유닛과원점은동일하며기대값/fixture변경없이실행전후를검증한다.

### EC 검증과 ED 검사 계약 보정

- EC production patch `903199808ada96b440defc7f810f1c9a80d8b48712acad5c70336d7fee673568`, Native CLI `bc3c3e9328a37a0adcd57d491991947e14a751e2bdea734ec18df2e3e5e4b3fe`, Docker WASM `79e403721a126b036b50ddda5de3a3bd6e3226379f796a1252e910e26df76364`를 고정했다. 원본11문서 전393쪽 Native/fresh WASM 모두90%이상(최저90.85014%), 쪽수 불일치·누락0이다. TSV/해시/명령은 `source-frame-band-ledger/visual-summary.json`과각 manifest에 보존한다.
- Native393쪽과fresh WASM393쪽의최저값은동일하다. scattered45·46쪽의표외곽/분할/글줄을같은배율로직접대조했다. 실제Studio7794가새WASM79e40372를읽고18쪽의16쪽Canvas를표시한증거는 `studio-browser-proof.json`이다. 사용자의이전판정과새빌드의자동/직접검증은구분한다.
- EC 기존34경계PASS; 이전실패22재실행은13PASS/9FAIL이었다. runtime코드의Clippy3종/workspace build/manifest/unit-tier는전부PASS였다.
- ED는runtime함수·조건을바꾸지않고9개검사의잘못된전제를보정했다. CLI양성입력은이미정본과Native/fresh WASM전8쪽최저96.87527%가확인된`issue2004_cell_image_stack.hwpx`의우측본문경계넘침으로교체했다. 종료코드3/필터/배치순서검사강도는유지한다. 정상화된overfill에넘침을되살리지않는다.
- #3593의두문단은같은source table/cell owner의103·104쪽조각에각각1회표시된다. 전115쪽양backend최저91.14637%,fresh103/104review 최저91.14637%와독립Print직접판독후, 같은물리셀을찾던2검사를원본소유·순서·각물리셀내포함으로변경했다. 절대px나쪽수핀을추가하지않는다.
- #1156의정본37쪽은성능요구사항으로끝나고38쪽은데이터요구사항으로재개한다. 관련37/38의Native/fresh WASM는99.46714%/98.34296%이고기존행16핀을소유·순서·본문포함·반복머리행외중복없음·원본전체행보존으로교정했다. KPS-AI전77쪽의다른11쪽미달은고정base에서도13쪽미달이던기존범위이며새미달쪽0이다. 이부분을전쪽시각통과로보고하지않는다.
- 합성꼬리줄은저장4줄×1200HU=64px로52px예산을넘는다. 뒤spacer가추가공간을만들지않으므로수용3줄/예산내소비를검사한다. 수치허용치를늘리지않았다.
- ED 재검사22/22PASS(`logs/corrected22.log`,77.398s). 전체검사·최종lint/Skia는아직진행전이며원격반영승인을받았다고해석하지않는다.

### EE: 전체 회귀와 새 경계 보정

- ED 전체 nextest는10,304실행/10,276통과/28실패/45skip였다. 새 manifest를 준비한 전체 실행의 결과이며, 이전22개 PASS로 전체 PASS를 대체하지 않는다. 기본 글꼴 환경에서28개를 재검사했지만 실패가 유지돼 글꼴 공급만의 문제로 판단하지 않는다.
- fit용 높이는 마지막 줄/개체의 점유 끝이며 문단 뒤 여백은 다음 원점을 전진시킨다. format에서 spacing-after를fit에 합산해 #5524 마지막 문단을 다음 쪽으로 넘긴 실제차이(20→25.333px)를 확인했다. EE는fit에서후행밖여백을빼고flow advance에는같은여백을보존한다. Source capacity restart도독립쪽경계의증거가아니므로작은조각최소값을우회하는분기는확인된원본사다리컷만쓴다. 실행전후및정상대조군을검증중이며기대값은바꾸지않는다.
- 실행파일section추출을위해잘못사용한objcopy가EC보관binary를측정후재작성했다. provenance의artifact_status에원래측정hash bc3c3e93와재작성hash b947851a를분리해기록했고최종재캡처대상으로지정한다. PNG/TSV를지우거나새빌드증거로바꾸지않는다.

- EE 67경계재실행은43통과/24실패다. #5524의fitSA차이는해소됐지만PrEP전체는141쪽으로남아완료로판정하지않는다. PrEP첫변화37쪽Table376은정본첫프레임높이217.683px/마지막행21.098px,입력object높이218.0267px/원시행24.6133px다. 마지막셀원본LH1000HU/LS500HU와padding280HU를분리하면내용+padding의점유17.0667px/자연center정렬끝20.84px가source마지막행21.12px안에든다.
- EF는해당sourceFrame의닫는행을scan전물리높이override로전달한다. 완전한원본plain표·후속host양수rewind·걸침셀없음·마지막줄/패딩/정렬포함입증을요구하며본문/빈문단은제거하지않는다. 마지막줄전진간격은그SourceFrame의내용점유가아니므로다음framerow로넘기기위해한쪽추가를만들지않는다. rowbudget/consumed/endOverride/paint는같은source높이를쓴다. 실행전이고기대값은유지한다.

### EF 경계 결과와 EG 원점 소유 대조

- EF guarded Native CLI `9af781768984502f223d49474094fec34b287dde05952b0ea6a16fd92528633d`, dirty patch `0005eff645772b70af41b0698e67a6a4a1195a4f1ba58c61e66e2107b0a473ff`를 고정했다. 67개 경계 검사는 59 통과 / 8 실패다. PrEP는 140쪽이며 37쪽 행 0–8 / 38쪽 행 9부터 이어받지만, 67쪽 끝 줄과 68쪽 캡션 간격은 아직 실패한다. #2019 20쪽, #5543 compat, #5585 일반 행 밴드, #4514 하단 넘침과 Native 수식 글꼴 검사도 남았다. 로그는 `post-full-regression/ef-guarded/logs/focused.log`이다. 전체 성공으로 판정하지 않는다.
- #4514 원본의 37쪽 Table760 앞 본문은 독립 HWP Print PDF에서 y≈325px, 표 윗변≈354.3px이다. EF는 표 윗변335.9px, 하단1055.4px로 본문 하단1046.9px를 넘는다. typeset의 표 진입248.1px와 paint 진입258.45px가 다르며 빈749의 후행간격을 paint만 좌표 재역산에 쓰고 있다. 다만 그 간격을 단순 제거하면 정본의 윗변에서 더 멀어지므로, 원점 생성과 빈 줄 소유부터 대조한다.
- EG는 표가 여러 단을 생성한 뒤 finalization할 때 이전 단의 owner를 현재 단의 predecessor로 사용하지 않게 한다. `post_flow::current_origin_rewinds`에서 비교할 owner가 실제 수용된 현재 단의 PageItem인지 확인한다. 값 clamp나 절대 source0 가정은 추가하지 않는다. 이 보정의 실행 결과는 아직 미검증이다.

### EH–EJ: 빈 밴드·첫 본문 원점·물리 예산

- EH의 top-aligned 행 밴드 보존으로 #5585는 14쪽 행3에서 내용 컷[7,7], 15쪽 같은 행3의 잔여 밴드, 16쪽 행9 시작으로 복구됐다. 내용만 맞춘 통째 행 소비를 허용하지 않는다. 그러나 모든 정렬에 선언 높이−마지막 줄간격을 하한으로 적용한 후보는 PrEP Table304의 center alignment slack까지 고정해 140→142쪽 회귀를 만들었다. 해당 후보와 실패 증거는 `post-full-regression/eh/`, `ei/`에 보존한다.
- EJ는 source closing frame에서 top-aligned 셀의 선언 후행 밴드와 center/bottom alignment의 slack을 구분한다. 전자는 실제 물리 공간을 이어받고, 후자는 padding·정렬된 전체 내용의 점유 끝이 프레임 안에 든 경우에만 source frame이 닫는다. rowspan이 행을 덮는 새 fallback은 기각하며, 별도 strong source anchor 경로의 기존 완전 내용 컷 계약은 유지한다. 새 후보이며 실행 후 전쪽 확인이 필요하다.
- PrEP 67쪽 문단16의 실제 끝 줄은 본문 안에 든다. fit의 본문926px에서 임의4px를 빼 현재850.8+71.5px를922px 예산으로 기각했다. 고정 여유를 없애며 실제 점유/원점 차이는 공통 결과에서 보정한다. 원본11문서의 EH 쪽수는 기존52/18/18/48/8/8/5/5/1/115/115와 같지만 전쪽 시각 통과를 대신하지 않는다.
- #2019 4쪽의 첫 break owner49는 ColumnDef+Bookmark를 가진 빈 문단이다. 저장 LH/LS/vpos는0이지만 실제 구성 줄3.2px를 소비한다. 첫 본문50은 기존Sequential 배치가 정본과 가까우나, 다음51부터 잘못 확립한 source0축 때문에 본문 위 여백55px가 중복 가산된다(정본 y148.25px, 기존 TextLine y203.6px). 빈 쪽나누기의0은 successor 좌표축을 인증하지 않는다. 동일 `HeightCursor`에서 실제 빈 줄 소비와 첫 본문 앞 여백을 `corrected_line_metrics`와 확정 흐름 원점으로 연결해 source transform을 세우며 측정/paint가 함께 소비한다. 단정의·책갈피 이외 개체, 합성/편집/재작성 원본 사다리, 이미 다른 내용을 소비한 column은 해당하지 않는다. EJ 빌드/시각 검증 전이다.
- Native 수식 font integration의 기존 devel 실패를 실제 base worktree `bfa28a77`에서 확인했다. 동일 assertion `equations ignored supplied font outlines`가 실패하며, 현재 layout 변경으로 생긴 회귀로 분류하지 않는다. 이 검사를 제거하거나 완화하지 않았다. 로그: `post-full-regression/base-equation-font-check.log`. 원격 devel SHA도 다시 조회해 같은 bfa28a77임을 확인했다.


### EK–EM 검증과 기준 출처 정정 (2026-10-12)

- 고정 HEAD는 `1563c48b3a2981992c70e958892543fe4e033a93`, base는 `bfa28a77a2714482f647ee6a7b9aa7f7876d8404`다. `current-base/candidate/rhwp`는 순수 devel이 아니라 이전 통합 source `1990ae5c` 실행 파일이었다. 해당 경로에 의존한 위치 분석은 diagnostic으로 낮추고, pure bfa CLI `latest-pinned-base-controls/candidate/rhwp`(SHA-256 `c563fe28ffb2b7ee830b454a3185d2e5f9f09d25b1f1bee0acd02aca97daef49`)로 다시 대조했다. 원 증거는 삭제하지 않는다.
- EK/EL은 닫힌 본문 프레임의 저장 좌표를 인증한다. Table760의 저장 host vpos + 원 개체 높이 + 위/아래 바깥여백은 본문 HU 높이를 2HU 직렬화 오차 이내로 닫는다. 새 페이지로 돌아가는 후속 원본 줄을 함께 확인한다. 실제 표 진입 원점 → fragment budget → scan의 수용 높이 → paint를 같은 축으로 연결했고, EL에서 #4514의 8.52px 본문 넘침이 제거됐다. 이전 문단 위치의 base 차이는 별개로 남아 전쪽 fidelity 완료로 판정하지 않는다.
- EM은 Paper/Paper 고정·Through·비-TAC·pageBreak None 표를 기존 nonflow shape 경로로 분류한다. 절대 용지 원점의 표가 body cursor를 밀던 #2019 Table73의 원인을 제거했다. 실제 flow 본문과 고정 표의 소유를 분리하며 ID/형식/특정 수치 조건을 추가하지 않았다. 현재 #2019는 18쪽, PrEP는 140쪽이다.
- EM frozen Native CLI SHA-256은 `29598468e53216b0aba4bc23f00dfbb0f4ca1579217fc2ffc3fb957e2c88fb4b`, dirty patch는 `fc8fcc57c9965f747a517974f6e1c8a6c777a0ded80a679b37838079f6fabb1d`다. 집중 검사 67건은 65 PASS / 2 FAIL(#5543 compat2024 위치 기대값, pure base에서도 재현된 Native 수식 글꼴)이다. 원본 11문서 Native 전쪽 TSV 재산출에서 scattered HWP 20–24·48–52쪽 새 미달이 발견돼 최종 통과로 판정하지 않는다. 작은 조각의 수용 guard 변경 전후를 추적 중이다.

### 메인터너 수동 #5543 PDF

- 원본 `E:\\tmp\\pr7704-manual-review\\issue5543_carried_anchor_ladder.hwpx`는 저장소 샘플과 SHA-256 `6adca8d39bdcf51e8791d8afa1e584b9570eb8b15df901e74ce61cfd500cdb4a`가 같다. MCP는 매크로 감지로 자동 처리를 거절했고 우회하지 않았다.
- 메인터너가 제공한 `/mnt/e/tmp/pr7704-manual-review/issue5543_carried_anchor_ladder.pdf`는 한컴오피스 2020에서 생성됐다(사용자 확인). PDF SHA-256 `ae12f3f6ebdfe4b443001fdbb30901dc7b7c04d29768a60dc399eff730f2169d`, Creator `Hwp 2020 11.0.0.9136`, Producer `Hancom PDF 1.3.0.550`, PDF 1.4, 전체 4쪽. 실제 메뉴 인쇄 경로는 별도 명시되지 않았다. 생성 연도를 2024로 바꾸어 보고하지 않는다.
- 독립 PDF의 ‘기상이변…’ 문단(pi42)은 4쪽에 있다. EM compat2024 Native 전4쪽 최저 90.72298%, 같은 조건 pure base는 4쪽 30.25537%다. 4쪽에서 문단·두 표·아래 그림의 소유/순서/외곽 위치를 직접 대조했고 현재 쪽 배치가 PDF와 대응한다. 이것은 **2020 출력과의 비교**이지 한컴2024 동작 근거가 아니다. `post-full-regression/em/manual-5543/`에 원 해시·TSV·PNG·로그를 보존한다.
- 메인터너 환경에 2024가 설치되어 있지 않아 기존 ‘2024 pi42=3쪽’ 주장은 미검증이다. 해당 검사 기대값/fixture/baseline을 변경하지 않는다. fresh Docker WASM과 전쪽 비교도 현재 진행 중이며 완료 증거로 보고하지 않는다.

- 사용자 추가 판정: 2024의 3쪽은 직접 확인하지 못했지만 한컴편집기 관찰상 3쪽 배치는 버그로 보인다고 했다. 그 판단과 실제 2024 출력 확보 여부를 구분한다. EM compat2022도 같은 2020 PDF 전4쪽 최저90.72298%, pi42는4쪽이며 compat2024 결과와 동일하다. 기존3쪽 검사 전제를 재검토하며, 공통 실제 점유/포함 계약으로 검사할 증거를 먼저 확보한다.
- scattered 새 회귀는 EC→EE에서 변경한 scan의 작은 조각 가드에서 시작했다. Table18의19쪽 EC는 행146의capacity-selected [1,1] 컷을 포함해984.373px를986.4px예산 안에 소비한다. EE/EM은25px최소높이 때문에963.493px에서닫고 행146전체를20쪽으로넘겼다. 독립Print20쪽은앞부분이19쪽에서이미소비된동일셀의이어받기를보인다. source boundary 강제와 가용공간으로선택된컷의수용을동일한증명으로취급한것이잘못된가정이다. exact original-unit cut의수용은별도물리높이예산을지키며, 이전/later source rewind로한페이지를강제하는조건과분리한다. 추적은 `post-full-regression/em/scattered-isolation/{ec,em}-scan.log`와동일입력19/20쪽PageItem/PNG에연결한다.


### EN: 가용공간 컷 수용의 회귀 수정

- EM Native11문서393쪽 산출이 완료됐다. scattered10쪽 미달 외 나머지341쪽은90%이상이다. fresh Docker WASM `55a88f0ab5e0cbd197f60031591e7147ede1f2d381f81575d279d61cf24e6779`와Studio public해시는동일하다. #5543의compat2022/2024 freshWASM각전4쪽도Native와동일한최저90.72298%지만, 직접2쪽판독에서하단로고의기존위치차이를확인했으므로완료판정이아니다. PDF로고상단934.179px대현재981.3px, 다른로고942.968px대991.7px다. 두그림은Table25와같은host의별도Shape 글상자소유이며저장글줄원점과자기TopAndBottom프레임계상을추적중이다.
- EM 기본feature전체회귀는scattered보정전후정확한head검증을위해중단한진단실행이다. 총10,220중6,853실행/6,839PASS/13실제FAIL/1SIGINT,45skip,3,367미실행이다. SIGINT는실제결함으로세지않는다. body-overflow6, off-canvas3, #6448 1, injection-inspection2, #5543 1을별도분류하고다음후보에서재확인한다. 이결과를전체PASS로바꾸지않는다.
- EN은scan이이미선택한원본줄유닛컷만25px최소조각가드에서보존한다. `row_plain_restart_capacity_cut`의정확한cut과수용cut이같아야하며별도실제물리높이예산은그대로다. 저장경계를강제하는`row_stored_zero_origin_cut`이나후속confirmed rewind의수용조건은변경하지않았다. 독립Print19/20쪽에서작은첫조각/이어받기의소유와공간내포함을확인한근거로잘못된source-vs-capacity등치가정을제거했다. ENNative빌드/formatcheckPASS,CLI SHA-256`f74de32b506fedac17bb4e8be2ac1d5a76887b95a489c3cc3c1d2669473fc834`,dirtypatch`98eb0b5903e4576a855d5bb2ced63f5e51a5169458890d2d86028ba8aaab5849`이다. 52쪽Native시각과source-boundary반례재실행이진행중이며새WASM/최종lint/전체회귀/remote단계는완료되지않았다.

- EN scattered Native전52쪽 최저93.25935%,90%미만0으로복구됐다. 경계5검사중원본#6761의표첫쪽행머리/이어받기본문포함은2PASS다. 기존#5543 3쪽기대와수동으로저장사다리를변경한#6761 두반례는실패했다. 후자검사는‘source split 강제근거가없다→25px고아기준으로전체행이월’이라는기존정책을기대하며, EN의실제capacitycut수용과충돌한다. 실제회귀인지검사의정책가정인지독립수정본출력으로확인해야하므로아직판정하지않는다. 해당기대값을바꾸거나조건을덧붙여감추지않았으며, 수정본으로의전체검증/제출이준비됐다고보고하지않는다. EN상태는 `post-full-regression/en/status.json`에보존한다.


### 2026-10-12 기준 출력의 사용자 예외 승인

사용자는 #5543의 **한컴 2024 출력 미검증을 허용하고, 직접 제공한 한컴오피스 2020 PDF를 정답지로 진행**하도록 지시했다. 적용 범위는 이 샘플의 기준 제품 버전이다. 2024 동작의 확인으로 기록하지 않으며, 2020 기준의 전쪽 Native/fresh Docker WASM 비교·확대 판독·실제 점유와 소유 회귀 검증은 유지한다. 기존 ‘2024 pi42=3쪽’ 기대값은 독립 출력의 올바른 배치와 공통 조판 규칙으로 재검토한다.


### EO: #5543 글상자 프레임의 공통 소비

독립 2020 PDF의2쪽 하단글상자 로고위치와 EN을대조했다. paint의Table25/ci0은871.8→977.5px로흐름을진행시키고, 뒤Shape25/ci1이977.5→1027.3px로자기높이49.813px를다시소비했다. 저장된두번째hostLineSeg는그부동개체의배제밴드를이미포함하며, 이를다음개체의원점으로사용하면같은프레임이중계상된다.

- 생산: `shape_flow::accepted_shape_frame`은기존shape측정이수용한Para-relative/TopAndBottom/비-TAC/Top또는Inside/offset0도형의실제시작과끝을반환한다. 비영offset·center/bottom정렬은별도앵커계약이필요해이순차원점의비대상이다. 입력문서ID·글줄수·고정위치조건은추가하지않는다.
- 측정소비: `place_table_host_shape`의기존페이지fit→필요한단전환뒤현재origin으로frame생성→`commit_table_host_shape`가동일occupied_bottom을누적높이와paragraph_float_placements에기록한다.
- 실제배치소비: `layout_shape_item`은해당shape의시작과끝을사용한다. z-order뒤`layout_column_shapes_pass`도동일control별anchor를사용하여,다른TAC가갱신한문단map이나저장다음줄을shape원점으로다시선택하지않는다. 후속TopAndBottom역행방지guard와최종흐름도실제로대조한다.
- 그림(Picture)의별도stored frame·비영offset·center/bottom배치·table없는host는기존경로로남으며충족으로보고하지않는다. shape를모든저장줄의특정위치로맞추는clamp는추가하지않았다. source호출경로와원본대조군검증중이다.

현재2020정답지사용승인은2024재출력부족을해소하는제품버전예외이며,두개의수동사다리반례를자동PASS로바꾸는승인이아니다. #6761의변형입력은예전25px정책의기대와실제capacitycut의차이를분리해재생성/독립출력으로확인한다. #5543의새검사도2쪽알려진결함을점수로면제하지않고공통프레임보정후추가/변경한다.


### EP: TAC 후행 간격과 저장 기준선

- EO의공통Shape프레임으로2쪽미달위치는줄었지만,독립PDF대비약11.5px차이가남았다. `DIAG_TAC`는Table25/ci0의실제줄높이49.28+줄간격10.4=59.68px를이미소비해current837.0에이른뒤, `DIAG_TAC_END`가다시문단tail10.4px를더해847.4로전진함을보인다. 다중TAC의비마지막줄은이미LH+LS를소비하므로문단후행간격복원은마지막TAC에서만한번수행한다. terminal source frame의끝점과그사이Shape의수용원점을연결하며기대값을맞추기위해공간을버리지않는다.
- 저장drawText의원본LineSeg는LH3163HU/BL2689HU다. 두TAC그림높이는2388HU/3163HU이며한컴PDF의두상단차이는8.789px다. 같은저장기준선비율을사용하면(3163−2388)×2689/3163×96/7200≈8.785px다. 현재fallback하단정렬은10.333px를만들었다. EP는유효한원본줄상자가현재개체줄의최대높이와직렬화한단위이내로일치할때그기준선비율을소비한다. 임의0.85를새상수로추가하지않는다.
- `issue5820/156560092_ecard_meeting_press.hwpx`의유사로고문단은저장LineSeg가없다. 해당경로는종전재구성하단정렬로남으며원본한컴PDF전2쪽을대조군으로실행한다. 측정의최대개체줄높이는바뀌지않고같은기준선의작은그림원점만바뀐다. 최대그림의점유상자/후행흐름/글상자내포함을확인한다.
- 새예외의제품버전차이를주장하지않으며,2024는사용자허용에따라미검증으로명시한다. 실제2020PDF의쪽/문단/개체소유가검사정답이다. EP빌드와독립출력검증전상태다.

- EP 고정 Native CLI로 #5543 전4쪽 최저95.35217%, 2쪽97.16339%를 확인했다. 저장LineSeg가 없는 #5820 대조군은 전2쪽 최저98.10960%다. 2쪽의 표 하단/본문/로고 두개를 독립PDF와 동일 배율로 직접 대조했다. 로고의 상대 상단차이는 source 기준선8.785px와 대응하고, 표 뒤 글상자의 원점과 간격이 개선됐다. 작은 좌표 반올림 차이는 남지만 약49px의 중복 소비는 해소됐다. 첫 EP 빌드는 바깥 함수에서 존재하지 않는 변수를 참조해 실패했으며, 실제 control 순서의 후행 TAC 판정으로 고친 `ep-native-fixed.log`가 성공 빌드다. fresh Docker WASM과 원11문서 Native393쪽은 실행 중이며 이전 후보 증거를 최종 head 증거로 대체하지 않는다.

### EP 검증 완료 범위와 사용자 직접 판정

- #5543는 기본/compat2024 옵션 모두 Native와 fresh Docker WASM 전4쪽 최저95.35217%, 2쪽97.16339%다. 실제 한컴2024 출력 확인으로 해석하지 않는다. 2020 정본 PDF를 `pdf/issue5543_carried_anchor_ladder-2020.pdf`에 보존했다.
- 사용자는 실제 Docker WASM `c149f884…`를 읽은 Studio `?review=pr7704-carried-anchor-2020`에서 2쪽 로고 위치·간격과 4쪽 첫 문단의 **시각 판정 통과**를 회신했다. `post-full-regression/ep/studio-browser-proof.json`은 실제 응답 해시와4쪽을 고정한다.
- 원본11문서 Native393쪽의 최저90.85014%, 미달·누락·쪽수 불일치0이다. fresh WASM393쪽은 현재 실행 중이다. 이 실루엣 결과를 전체 회귀/최종lint/직접 판독의 완료로 대신하지 않는다.
- #5543의 기존3쪽 기대는 실제 기준과 달라4쪽 소유로 교정하고, 표→글상자 저장 간격의 단일 소비 및 두 로고의 저장 기준선·프레임 내부 포함을 관계형 검사로 보강한다. 검사는 현재 작성 중이며 수정 전 FAIL/후 PASS와 후속 검증을 기록한다.

- 새 #5543 관계형 검사2건은 실제 EP renderer에서 PASS다(`ep/logs/5543-tests-final.log`). 첫 실행은 fmt 전 생성한 harness와 fmt 후 source weight의 배정이 달라0건 실행이었으며 PASS로 세지 않았다. fmt 뒤 harness를 재준비한 최종 실행은 정확히2건을 실행했다. 테스트 작성 중 타입/variant 오류는 실행 결함 재현과 구분하고 교정했다.
- 수정 전 EN production patch를 별도 `/tmp/rhwp-pr7704-before-ep-20261012`에 재구성해 동일 검사로 FAIL 여부를 실행한다. 기존작업공간과WIP는 보존하고, Cargo target은 기존 공용경로 하나만 재사용한다. baseline·래칫 허용치를 변경하지 않았다.

- 동일한 새 회귀를 EN 코드에서 실행해 간격 assertion의 실제 FAIL을 확인했다. 표→글상자 간격60.213333px가 원본줄간격+바깥여백14.173333px를 중복 소비했다. EP에서는 동일검사 PASS이며 로고공통기준선/프레임내포함도 검사한다. EN의4쪽 소유검사는 이미PASS였으므로 쪽수검사를 새결함검출로 주장하지 않는다. 정확한test/source해시와로그는 `ep/5543-regression-evidence.json`에 연결한다.

### EP 원본 전쪽 비교와 현재 회귀 결과

- 원본11문서393쪽의 Native와 Docker WASM 비교가 모두 완료됐다. 양backend 최저90.85014%,90%미만·누락·쪽수 불일치0이다. 전쪽TSV/최저쪽/해시는 `post-full-regression/ep/visual-summary.json`에 연결한다. 실루엣 산출의 `not_evaluated`는 자동승인으로 해석하지 않는다.
- 수정 전EN 검증 후 공용target의EN 라이브러리가 suite-only 실행에 재사용됐다. `remaining-regressions.log`는현재source의증거에서제외했다(`remaining-regressions-invalid-cache.json`). target을삭제하거나추가하지않고현재 `src/lib.rs`의mtime만새로갱신하여실제라이브러리재빌드뒤 `remaining-regressions-current.log`를산출했다. source내용은변하지않았다.
- 현재 집중18검사는4PASS/14FAIL이다. #5543 두검사와 #6761 원본의행머리/이어받기본문내포함은PASS다. 남은실패는body-overflow6,off-canvas3,injection-inspection2,#6448 1,#6761 변형입력2다. 자동시각전쪽통과를전체회귀PASS로보고하지않는다.
- format check, 고정base `bfa28a77…`의manifest 및unit-tier정책은PASS다. 최종Clippy3종/workspace/Skia와전체회귀는완료되지않았다. 현재push/PR/merge단계의준비완료로판정하지않는다.
- #6761 독립수정본Print는control/shifted/later 모두행머리를39쪽에두므로기존40쪽기대의근거가없다. 다만later전체는Print104쪽대현재103쪽이어서별도실제차이가남는다. purebase도같은103쪽이다. 선택39/40쪽의높은점수만으로전쪽이나확대판독통과로대신하지않는다.
- #6448 2020PDF의표는2쪽94.297→1011.533px다. 현재표906.667px와그뒤문단은2쪽에수용된다. PDF에서앞행하단977.331px대현재둘째행글줄963.5px의차이가확인됐다. 작은행의내용최소높이와앞행선언높이를선언총높이로맞추는경로를의심하며실제호출경로를추적중이다. PDF의TAIL은3쪽이다. 저장host줄이작다는조건의정책가드와실제행기하를분리해추적중이며,3행/1열같은형상예외를더추가하거나기대쪽수를2쪽으로완화하지않는다.

### 최신 devel 병합 확인 후 전수 회귀 대조

- 사용자 지시에 따라 최신 `upstream/devel`을 fetch하고 merge했다. 기준 `bfa28a77a`는 이미 review HEAD `1563c48b3`의 조상이어서 `Already up to date`였다. 미커밋 보정과 PDF를 보존했고 production patch는 EP와 동일한 `0b0ead62…`다.
- 현재 작업본의 기본 feature 전체 nextest는10,221실행/10,199PASS/22FAIL/45skip이다. 중단·미실행 없이 끝났다. 앞선14건은부분18검사의결과이며이번전체에서추가8개실패를확인했다.
- 순수 고정devel을동일명령/환경/공용target으로재빌드하여대조했다.10,220실행/10,220PASS/0FAIL/45skip이며작업본실패22개모두대응하는기준검사는PASS다. 전체회귀상태는미충족이다. 실패검사수를서로다른버그수로세지않는다.
- 실패묶음은본문넘침6/용지밖배치3/HWP3 CLI·변환9/#6448 1/#6761변형2/#6812 1이다. 합성기대값과독립출력의충돌은별도재검토하며실행으로확인된배치결함과구분한다. 검사·baseline·허용치를바꾸지않았다.
- 정확한patch해시/전체명령/최종원문/전수PASS·FAIL대조는 `post-full-regression/devel-merge-check/{provenance.json,comparison.json,report-ko.md,logs/whole-current.log,logs/whole-base.log}`에보존한다. 공용캐시의cross-worktree혼용을막으려고내용변경없이진입source의mtime을갱신해각library/CLI를재빌드했다. 검증뒤공용산출물은작업본으로복원한다.

- 공용산출물의작업본복원을완료했다. #5543 두검사2PASS, 복원CLI의SHA-256은전체작업본검사에사용한CLI와동일하다(`5a060ebb…`). production source해시는변하지않았다. `devel-merge-check/restore-proof.json`에정확한해시·검사·base clean 상태를보존한다.

### EQ: HWP3 저장 페이지 경계 보존으로 회귀 보정

- 22실패를순차처리하라는사용자지시에따라HWP3의9개CLI실패부터추적했다. 동일 `hwp3-sample.hwp`의6쪽76문단은base가[0,9),작업본이[0,10)을수용한다. 현재10번째줄의잉크하단이본문보다15.413333px아래다. fit진입원점은양쪽686.933333px로같다. 본문876.853333px대pagination예산898.186667px의차이는IR의레거시하단허용치1600HU다.4px안전여유는원인이아니고이잘못된경계수용을우연히막았다.
- 원본입력은저장소의nativeHWP3이며parser는원본line_info의pgy낙폭을실제다음쪽시작으로이미읽어LineSeg.vpos=0을만든다.이정보가commonIR에서좌표로만남아,나중에놓는그림이있는본문은text-metadata-only reset 조회에서배제된다. 독립한컴2022 PDF는16쪽이며6쪽의해당꼬리와7쪽의재개를대조한다.
- 생산: HWP3 parser의기존pgy낙폭전환에서같은LineSeg에공식공통bit0(`TAG_FIRST_LINE_OF_PAGE`)를보존한다. renderer에HWP3분기를추가하지않는다.
- 소비: generic `stored_page_start_line`은원본줄소유/동일줄수/비dirty ladder의명시적page-start를선택한다. `prepare_forced_page_boundary`가whole-fit과line scan 모두에같은break_line을전달하고,`plan_fragment`의[시작,끝)과동일경계를paint가소비한다. 편집세션·재조판/dirty와문단첫줄은대상이아니다.레거시tolerance나빈줄의실제전진량을없애지않는다.
- 현재코드보정후검증전이다. 새회귀/fixture/기대값/baseline을추가·변경하지않았다. 원본/독립Print의전쪽Native/freshDockerWASM와기존9검사·경계대조군을검증한다.

- EQ 최종 Native 후보의 production patch SHA-256은 `a003efd6ac4743ea2f6015e16af54b3f569c3212939acb37b216cd322ac9102f`, CLI는 `9d031eefa764e0cc326465b01e8a6554c453b585ff139ec36b447c17ba4a9f28`이다. 명시적 쪽 시작은 첫 경계만이 아니라 fragment 시작 뒤 다음 경계를 매번 조회해 긴 문단의 후속 쪽 소유도 보존한다. HWP3/CLI/그림 앵커 기존283검사 모두PASS(`eq/logs/focused-tests.log`), 기존9실패도 모두PASS다. 원본16쪽 Native는 고정devel 출력과 PNG 해시가 전쪽 동일하며 영향6/7쪽94.24933/93.82895%다. 독립PDF/overlay 직접판독에서6쪽 꼬리의 소유·7쪽 재개/그림 원점을 확인했다. 7쪽 캡션 등의 기존간격차이와3/8쪽 최저56.61381/54.09223%는 base와동일하게 남는다. 새검사·fixture·기대값은 추가하지 않았다. fresh Docker WASM과 잔여 실패검사는 아직미완료다.
- #6448은 순수base도 표2쪽이65.05803%다. 기존3쪽 검사의PASS만으로 내부행기하의 정합을주장하지않는다. 같은Print 전3쪽을대조했고 앞행 높이가 총높이맞춤에서20.6px축소되는것은양쪽공통기존결함이다. 이번4px안전여유제거는뒤문단 이월의잠정가드를드러냈으며 표물리행계약부터검토한다.

### ER: 변환 여부와 무관한 저장 본문 조각 소유

- 원본 HWP5 변환본의 actual LineSeg는 sample5 문단189에서69120→70560→72000→0이며현재flow921.6px가첫저장줄과일치한다. 독립2022PDF8쪽은이첫3줄을소유하고다음쪽에서이어받는다. 현code는변환profile을 generic anchored-body owner조회에서제외하여6/7줄을같은쪽에수용한다. HWP5-nopassword의213문단도61888→63488→0이며14쪽2줄/다음쪽재개가PDF에대응한다. EQ에서실제로실행한flag수동진단은16/64/24쪽을그대로유지하고넘침0을보였으나합성입력진단이지원본수정/정답증거로세지않는다.
- 적용규칙: 원본비dirty/동일줄수/단일단/현재flow와첫저장줄원점일치/단조prefix/현재본문내마지막잉크/다음줄0reset의조합이소유근거다. 파일container나변환계보는이같은기하근거를거절할이유가아니다. 로컬cursor·편집·reflow는같은유효성검사로거절한다. reset뒤에소유된문단기준비TAC그림도앞쪽의control-free prefix소유를바꾸지않는다. 절대배치·TAC·표·앞쪽에소유된개체는비대상이다.
- 생산→소비: `prepare_forced_page_boundary`의anchored조회와`scan_lines`의prefix owner수용은같은`stored_body_reset_fragment_matches_current_flow`를호출하고선택한끝컷은`plan_fragment`→PageItem [start,end)→paint가소비한다. legacy21.3px허용치나본문좌표를clamp하지않는다. 원본Native/독립PDF의영향쪽과기존원장실패를선행검증한다.

- ER 고정 Native CLI의원문/해시는 `er/provenance.json`이다. 16/64/763/24쪽 원본변환군의넘침·off-canvas는모두0,쪽수불변이다. 기존실패22개중17개가EQ/ER에서PASS로복구됐다. body/off-canvas 모든32partition과주변경계등59검사를재실행한결과54PASS/5FAIL(`er/logs/focused-tests.log`)이다. 남은실패는#3751본문원장1/#6448 1/#6761수동변형2/#6812 1이다. 전체10,221재실행과ERfreshWASM은아직미완료다. EQ Docker WASM은`ac585323…`이고원본16쪽Native와PNG해시가전쪽같다.

### ES: 실패 fit 재시도의 계약과 저장 줄 증거 유효성

- #6448: whole-fit은strict-following 계약에따라TAIL의전체advance21.333px를요구하여914.667+21.333>933.6으로거절한다. 하지만split의`plan_fragment`는직전Table만보고trailing8px를빼13.333px만으로다시전체를수용한다. 동일문단의같은후행공간요구를분할재시도에도전달해야한다. 기존strict 계약의적용조건을추가/확장하지않고전체fit→split→실제예약의소비불일치를수정한다. 현재2쪽표내부의기존65%차이는별도로남기며새검사/fixture/baseline을추가하지않는다.
- #3751: 수동저장LS34개가실제재구성줄21개와다르다. 현재scan은li인덱스로원본LS를가리켜14·15번째구성줄도원본tail로오판한다. 실제줄/저장줄일대일대응없는경로에서저장slot을fit근거로쓰지않는다. #6812의기존multiple-pictures반례는소유줄수3개가같지만실제진입893.8px가원본첫줄855.43px에서벗어나있다. 앞그림의증가를소비한현재fragment와원점이맞지않는저장tail을fit허용근거로사용하지않는다. 기존source-anchor조회의validity를소비하며임의clamp나안전여유를추가하지않는다.

- ES 전체기본feature회귀는10,221실행/10,218PASS/3FAIL/45skip로완주했다(`es/logs/whole-tests.log`). 기존22중19복구,새FAIL없다. 남은것은#6761변형2와#6812다. #3751 raw저장줄34개대구성21개의잘못된인덱스참조와#6448 strict전체advance의split재시도누락은각기존검사에서PASS로확인했다. 원래시각게이트문서rowbreak HWP/HWPX,scattered Native는이전전쪽점수와같으며나머지원본검증진행중이다.

### ET: 저장 꼬리 끝점의 같은 페이지 기준 좌표 사용

- ES에서도#6812가FAIL하여실제`LineScanPage`→분할→paint를추가대조했다. 시작줄조회는`current_page_vpos_base`를빼지만`saved_line_range_fits_body_tail`은rawvpos로끝을잰다. 이문서에서는앞그림의추가60px를소비한뒤지연기준이바뀌므로raw끝939.43px는수용되고실제2번째줄끝1046.79px는단끝1043.15px를넘는다. 원점과끝점의같은좌표계가계약이다.
- helper가같은수용page-base를받아`line_seg_visible_bounds_px`로모든줄끝을계산하도록수정한다. source자체의단조성·비합성/유효성은그대로검사하고,본문내/각주위판정·suffix control owner·hwp-authoritative 조회도같은원점을소비한다. 페이지이월뒤에는Entry의옛기준을재사용하지않고새페이지의첫항목또는현재PartialParagraph의시작source줄기준을조회한다. 이선택은paint의첫PartialParagraph소유와대응한다. 원문/기존반례검사로그를연결하며baseline은바꾸지않는다.

- #6761의동일변형입력을ES에서다시실행했다. in-memory/HWPX roundtrip의control103/shifted103/later104쪽이같다(`es/logs/counterexample-generate-final.log`). EP에서저장한later HWP도ES에서는104쪽으로복구되어이전103쪽대Print104쪽차이가해소됐다(`es/logs/later-hwp-info.json`). 새HWPX변형을실제2020 Print하여같은103/103/104쪽을확인했다. `es/diagnostics/mcp/{control,shifted,later}-status.json`의engine2020/한컴11.0.0.9136/Print0/driver one-up출처를고정하고전쪽Native비교를진행한다. 단지쪽수가같다는것만으로기존40쪽정책기대를수정하지않는다.

- ET 집중59검사는56PASS/3FAIL이며#6812는남았다. page-base를맞추는수정은이입력의실제base가0이므로목표결함을검출/해소한증거가아니다. 실제 `FLOW_ORIGIN`/`DIAG_TACCAP` 대조에서typeset893.8px대paint의같은시작902.093px가갈린다. 앞TAC의cap은저장표줄의후행간격을절반으로독립재계산하고outer-top을다시더해이미수용한라인끝을회수한다. ET가목표를해결했다고보고하지않으며원인연결을다음EU로교정한다.

### EU: 자연스럽게 닫힌 TAC source 줄 끝의 공통 소비

- #6812의앞두표는원본LineSeg 높이가개체+상하바깥여백을정확히포함하고다음문단원점이LH+LS+문단간격의끝을닫는다. `stored_inline_table_successor_advance`의기존Table→다음줄공통producer가이계약을이미검증한다. cap의LH+LS/2+outerTop은같은줄을다르게해석해원점을되감았다.
- 기존single-table wrapper의적용범위는그대로두고,동일한single TAC line의owner를ctrl index로조회하는공통query를추출한다. 다른inline/문단기준TopAndBottom개체의독립advance,텍스트·여러줄·dirty/합성은비대상이다. Paper/Page 그림의어울림clearance는기존InlineBoxPlacement의geometry로별도소비한다. 자연source줄이완전히닫힌경로만cap에서도같은LH+LS+after를소비하여바깥여백을재가산하지않는다. 실제이월/paint뒤에서는그declared 원점을clamp하지않는다.
- 생산→조판cap→paint: float_placement의닫힌source-slot query→tac_reconcile이확정source 끝을소비→capped_bottom은기존side-wrap clearance를더함→layout은같은Table source box와successor-end를소비한다. 저장slot이현재측정본문을못담는경우실측성장floor를유지하며임의형상/문서조건을추가하지않는다. 독립Print1·2쪽과기존multiple-pictures검사를재실행한다.

### EV: 후행 선문자의 줄 원점과 테두리 원점 분리

- EU 전체 검사는 10,221 실행 / 10,218 PASS / 3 FAIL / 45 skip이다. 기존 #6812는 PASS이며 동일 multiple-pictures 11쪽 Native 최저 96.06939%다. 반면 #2020 표시선 검사 1건이 새로 실패했다. 이를 새 회귀로 기록하고 수정했다. #6761 수동 변형 2건은 계속 미해결이다.
- #2020의 F081C 후행 표시선은 저장 줄 기준선을 표의 테두리 원점에 더했다. 테두리 원점은 이미 위 바깥여백을 소비한 좌표라 이를 다시 줄 원점으로 변환해야 한다. `push_tac_post_f081c_line`에서 outer-top을 한 번 빼고 저장 기준선을 적용했다. PDF/원본/검사 범위에 맞추는 임의 오프셋이나 검사 공차 변경은 없다. 동일 API에서 선의 y는 1035.71333 → 1033.84667px이며 표·날인 텍스트는 그대로다. 기존 검사 재실행은 EV 전체 로그로 확인한다.
- 동일 원본을 MCP engine2020 / 한컴 11.0.0.9136 / Print0 / one-up으로 새 출력했다(job 8fd42fa0-5fad-43f4-bb97-03385c449d3b). 전체 Native는 EU 79.50870%로 기존 글꼴·배치 차이가 남는다. 이를 시각 통과로 보고하거나 검사 기대값을 바꾸지 않는다. EV의 표시선 판독과 fresh WASM은 별도로 이어간다.
- #6761 원본의 기존 피델리티 문제는 열린 #7739에 이미 등록되어 있다. 기존 정상 검사 동반 제외를 철회한 이력이 있으므로 원본과 정상 검사는 유지한다. 순수 devel과 현재 source를 동일 원본·Print PDF로 비교하고, 잘못된 40쪽 가정을 요구하는 두 변형 검사의 처리 범위만 확정한다. 아직 검사 제거/이관은 하지 않았다.
- EV 해시·로그는 ignored `post-full-regression/ev/provenance.json`, `ev/logs/whole-tests.log`, `ev/logs/receipt-probe.txt`에 보존한다. EU·ES의 시각 결과를 EV 최종 결과로 재인용하지 않는다.

- EV 최종 기본 회귀는10,219PASS/2FAIL/45skip(10,221실행)이다. Rust lint8단계와 NativeSkia 공식3종은 모두exit0이다. 원11문서393쪽 Native/DockerWASM 최저90.85014%/미달0이며 서로와 기존EP Native의전쪽 PNG가같다. #6812 11쪽96.06939%,#5543 4쪽95.35217%,#5820 2쪽98.10960%도 양backend에서동일하다. `ev/provenance.json`과 `ev/final-visual-summary.json`에최종source/CLI/WASM과수치를고정했다. remaining #6761 검사2개는#7739의독립Print/기존피델리티와연결할처리안을준비해승인대기중이며제거·기대값완화·원격게시를하지않았다. 전건해결/PR수용으로판정하지않는다.

### EW: 사용자 결정에 따른 원본 문서 전체 출력 보정

사용자는 2026-10-13 두 검사 이관보다 기존 화학제품 표시기준 연구 원본의 전체 출력 보정을 선택했다. 이관안은 실행하지 않는다. 정상/실패 검사2개와 원본·기대값·baseline을 유지하고, 원본103쪽을 독립 한컴2020 Print와 대조하여 첫 미달부터 실제 조판 원인을 순차 수정한다. 기존11문서393쪽 및 #6812/#5543/#5820의 EV 검증을 대조군으로 고정한다. 새 source에서는 영향 경계와 Native/fresh Docker WASM을 다시 확인하고 최종 전쪽·전체 회귀를 실행한다.

- EW 동기화: 최신 upstream/devel `552f5228d`를 작업 HEAD `e4a7964d6`에 충돌 없이 merge했다. 기존 WIP patch hash `41c44c63…`는 merge 전후 같다. 원본 HWP SHA `398d03a5…`를 MCP job `43fff0a9-6830-4ee0-8aa3-dac51b7f8f36`으로 새 Print하여 103쪽을 확인했다(2020 / 11.0.0.9136 / Print0 / one-up / preprocessing none). 새 PDF SHA `00d7f6f0…`이며 기존 PDF와 일부 페이지 raster가 달라 새 Print를 기준으로 사용한다.
- 15쪽: visible-text Picture host 뒤 실제 빈 줄의 before-space가 그림 하단에서 중복됐다. `stored_picture_successor_with_following_placement`가 source host 원점·실측 글줄 끝·그림 하단·후속 두 줄의 연결을 검증하고, section controls의 동일 placement를 typeset/paint가 소비한다. actual 빈 줄 565.04→557.04px, caption 이후도 8px 중복 해소. Native fresh Print 99.7098%; 직접 그림/빈 줄/캡션 경계를 대조했다. 기존 #7048 6건과 #6950/#6704 29건 PASS. frozen CLI `6d9b8e26…`, runtime patch `71613433…`의 전103쪽 미달은 55/78/79/80/82/85이며 EW 최종/전체 통과로 판정하지 않는다.
- 55쪽: 마지막 그림 셀 높이는 원본의 signed 음수(-638HU)다. 이를 INFINITY로 취급해 source empty anchor line 1100HU를 실제 그림 높이에 더한 뒤 TAC 공통높이로 다른 행까지 축소했다. row-sizing과 같이 유효 하한 없음(0)으로 읽어 실제 그림 점유+padding을 소비한다. Native 87.12434→99.08230%; 그림/행 경계/캡션 직접 판독, 새 검사·기대값 변경 없음.
- 78–79쪽 독립 대조군: original-derived control HWPX에서 첫 그림 높이만8000HU로 바꾸면 첫 그림이78쪽, 기존 작은 그림은79쪽에 배치된다(job `9cb7359b-b08a-4222-9a97-545b045643a4`). allowOverlap만 켜면78쪽의 작은 그림은 같고79쪽의 큰 두 그림이 같은 원점에 겹친다(job `d3bbdf51-1e8e-4acf-bb64-23cf75b9ec75`). 모두 103쪽, 한컴2020 Print0, 단일 속성 합성 대조군이며 원본의 통과 증거로 대신하지 않는다. source closing line vpos36386HU는 이월 두 그림의12379+24007HU와 일치하고, 원본 후속 문단38434HU는 그 줄+LH1200+padding282+outer566의 끝을 독립적으로 닫는다.
- 구현 중: `stored_partitioned_picture_cell_flow`가 첫 저장 물리 프레임의 first-fit과 위 closing source의 일치를 검증하고 그림 control ID별 atomic units/closing blank-line/physical slack을 공통 산출한다. measurement→cell_units→RowEntry split eligibility/컷→일반/partial paint의 owner와 원점에서 같은 계획을 사용한다. source dirty/수동 변형/겹침 허용/기타 앵커/수평 비겹침은 거절한다. 처음 실행은 RowEntry가 단일 text-line inventory로 셀을 통이월해78쪽 미달이 남았다. 그림 소유 unit을 실제 분할 진입부에 연결하여 재검증 중이다. 이 상태는 완료가 아니다.

- 그림별 atomic unit을 RowEntry의실제분할진입까지연결한 Native에서는79쪽99.11839%,80쪽99.93608%로누락/겹침/이월이해소됐다(정확값은 `ew/partitioned-split-native/chemical/overlay/overlay_metrics.json` 원문).78쪽은85.53236%이며앞표의4px상향이남는다. 전체103쪽/freshWASM/새head전체회귀는아직미완료다.
- 85쪽의167문단은양수4000HU가자기문단before53.3333px와정확히같고다음168문단도7740HU에서LH/LS/before를닫는다.앞166문단실제수용끝828.7067px도그저장줄끝과일치한다.기존 near-top2500HU 상한대신 `stored_margin_page_restart`의동일저장원점/실제전진/후속여백계약으로positive restart를판정한다. Native84쪽99.45177%,85쪽97.02406%;보정전85쪽20.40590%.
- 82쪽은새first-float-margin predicate만으로는미해결이다.실제PageItem목록은첫비TAC Table141→PartialParagraph141→empty142→Table143이며, `source_host_origin`이첫Table을TAC만수용하는것이원점배제원인이다.초기Shape가원인이라는코드추정은실제item목록과달라철회한다.비TAC의완전한첫부동프레임/후속source줄의닫힘을검증해같은단의시작으로수용하도록수정중이다.
- 78쪽Table123의negative-offset만0으로바꾼대조군job `1f4ad61b-d897-4617-834f-3541e6666bef`는한컴2020Print103쪽이며78쪽96dpi raster가control과완전히같다.반면rhwp는음수-300HU를table-body상단에적용해caption전체flow를4px되돌렸다.0대조군Native78쪽98.96341%다.캡션이위에있는문단기준TopAndBottom/flowWithText/allowOverlap=false 상자의앞공간은body와분리하지않고동일흐름값을소비한다.절대/배경/겹침허용/다른정렬은대상이아니다.기대값/baseline 변경없음.

- 최종 Native 후보(검증 중): 첫비TAC Table이실제수용된Exclusion이고첫before/부동외곽/다음source줄이닫힐때TAC와같은저장단시작으로수용했다. Shape 전용확장은실제호출경로가아니어서제거했다. source_host_origin→from_stored_host/head→record_paragraph_float_placement→paint에서같은원점을소비한다.143표 y471.6→463.6px,145표 y764.2→748.2px이며82쪽74.27482→99.93754%.78쪽은caption flow offset의공통producer를typeset get_table_vertical_offset과compute_table_y_position에서소비해98.48551%.
- source HEAD e4a7964d6/base552f5228d와최종runtime patch/CLI는 `ew/final-native-provenance.json`에고정한다.원본103쪽 Native전쪽TSV,동일source Docker WASM build,원PR11문서393쪽Native재검증을진행중이다.이전반복의점수를최종head전쪽통과로재인용하지않는다.새검사/fixture/기대값/원격변경은아직없다.

### EX: 마지막 그림 뒤 실제 빈 글줄의 소유와 끝점

- EW 전체 원본 Native 103쪽은 최저90.47526%(19쪽), 미달/누락/쪽수 불일치0이며 원PR11문서393쪽도 최저90.85454%, 미달0이다. 그러나 확대/self-review에서79쪽 마지막 그림 뒤 빈 TextLine의 실제 bbox가 첫 그림 위137.9px에 남아 있음을 발견했다. 가시 이미지 점수만으로 빈 줄의 실제 공간 소유를 완료로 판정하지 않는다.
- 공통 picture-cell units의 마지막 owner와 실제 fragment 높이에서 closing line의 원점·높이를 산출한다. 일반/partial paint의 문단 spacing과 커서끝도 그 결과를 소비한다. 첫 y_start 연결만으로는 `paragraph_layout`의 source-vpos 재선택이 원점을 덮어써 실패했다. 기존 stored_cell_content_origin 소비 지점까지 같은 원점을 전달해 조정한다. 이어받기 끝 컷의 usize::MAX는 열린 terminal owner sentinel이므로 실제 unit 목록의 끝으로 해석해야 하며, 그렇지 않으면 높이 조회가 None이 된다. 좌표 clamp/출력 숨김은 사용하지 않는다.
- 첫 float source-frame 수용은 저장 선언뿐 아니라 실제 Exclusion placement의 occupied_bottom이 후속 source 첫 줄을 닫는지 확인한다. 캡션이 있는 첫 표는 common.height만으로 전체 frame을 증명하지 못하므로 이 closure 대상이 아니다. RowEntry의 picture-plan 조회는 helper 자체의1열/마지막행 domain에서만 cell을 검색해 불필요한 전행 순회를 피한다.
- EX source로 Native/fresh Docker WASM과 전체 회귀를 재실행한다. EW의 완료된 비교/Docker 해시는 보정 전 증거로 보존하며 EX 최종 결과로 재사용하지 않는다. 기존 정상/실패 검사·기대값·baseline·원본을 유지했고 신규 fixture/검사/원격 변경은 없다.

- EX terminal-cut 보정 뒤79쪽 실제 빈 줄은 y623.1/h16.0으로 마지막 그림(303.0+320.1) 뒤에 위치하며 셀 끝641.0 안에 든다. 같은 probe의82쪽143/145표는463.6/748.2px를 유지한다. 최종 production patch98bb7da2…/CLI3fce5704…를 `ex/provenance.json`에 고정하고 전체496쪽 Native 및 Docker build를 시작했다. 실제 source 이후 덮어쓰기 경로까지 확인한 증거는 `ex/terminal-probe/render_tree_079.json`이다. 아직 전쪽freshWASM과전수회귀/최종판독은 미완료다.

- EX 원본 전103쪽 Native/fresh Docker WASM 모두 최저90.47526%, 미달/누락/쪽수 불일치0이다. Docker WASM b4668062…를 Studio가 실제 읽고103쪽을 표시했다(`ex/studio-browser-proof.json`). Native 원 PR11문서393쪽도 최저90.85454%,미달0이다. 이 수치만으로 다른 회귀나 전체 PR 통과를 선언하지 않는다. 전체103쪽 portable 비교는 `ex/chemical-full-review/index.html`이다.
- EX 기본 feature 전체는10,219실행/10,214PASS/5FAIL/45skip이다. 최신base#7861의검사구성에따른2건차이가있으므로 이전10,221분모와혼용하지않는다. 기존#6761수동기대2건 외에 #5701소유검사/같은입력의text-overlap partition0/#6778쪽수검사가추가실패했다. 새실패3검사는현재두배치원인과연결되며 baseline·기대값을변경하지않았다.

### EY: 원본 개선 뒤 새 회귀의 물리 프레임 유효성

- #5701의표는 common.height=17920HU(첫14행),실측16행=20480HU이다. 유효한글줄원점을전체표의완전한저장프레임으로간주해 paper-margin 예산을 부여하여,요구하단906.9067px가본문예산876.9067px를넘어도통째수용했다. 실제출력은후속문단5줄을1쪽표위/안에그려20건겹침이생겼다. 독립2020Print/기존관계형검사는14행+2행조각과후속1줄+4줄소유를요구한다. legacy paper-margin 수용은 original object frame이실측물리행전체를담을때만가능하고,첫조각만담는 common 높이는그권한이없다. 미수용은본문예산의기존row scanner로흘려같은컷을paint까지소비한다. 출력좌표 clamp나테스트완화는없다.
- #6778은저장문단57을이미새쪽에놓고source원점을재기준화한뒤58의positive margin reset으로한쪽을더열었다(12→13쪽). 새certificate의이전끝점에서frame_origin을빼면재배치된원점을독립sourcepage증거로오판한다. positive-margin certificate는이전/다음사다리가원래page-relative frame을공유할때만인정하며,이미rebased된페이지는일반fit/source수용경로를유지한다. 원본85쪽의unrebased source계약과대조한다.
- 기존정확한#5701/#6778검사와PDF를그대로사용하고focused Native/DockerWASM 및전쪽대조군을재확인한다. EX 프로덕션/artifact는증거로동결했으며 EY결과로재인용하지않는다. 임시 ANCHOR_TRACE는소비값로그를남긴뒤제거했다. 새fixture·golden·기대값변경·이관·원격변경은아직없다.

- EY 확정 검증: runtime patch `65f50cdc8a67eb1f0345f7feed4d62cc0b4afb060fe00198e6d12aee3db2b19b`, Native CLI `7165e9bf…`, Docker WASM `f9baf256…`의 원본 전103쪽 Native/fresh WASM 최저는 모두90.47526%, 미달/누락/쪽수 차이0이다. 전103쪽 PNG도 양 backend가 동일하다. 2026-10-13 메인테이너가 Studio `?review=pr7704-chemical-full-current`와 전쪽 비교를 확인하고 **시각 판정 통과**를 명시했다. 이 판정은 해당 원본·EY 산출물 범위이며 후속 source 보정이나 변형 문서의 시각 차이를 면제하지 않는다. 전체 nextest는10,219실행/10,217PASS/2FAIL/45skip으로 EX 새 회귀3건이 복구됐고 #6761 수동 변형 검사2건은 남는다(`ey/logs/whole-tests.log`).
- 남은 변형 `later.hwpx`의 Print104쪽과 현재104쪽은 같지만40쪽의 표 뒤 캡션·후속 문단이 약4px 위로 이동하여41–44쪽 소유가 달라진다. 실제 FLOW_END의 terminal flow/reserved는0이고 원본 outer bottom=283HU이다. 저장 LineSeg 존재만으로 종료 여백을 복원할 수 있다고 가정하는 HostSpacing producer를 검토한다. source 닫힘이 실제 성장 프레임과 맞지 않으면 이 여백이 사라진다. 예약→마지막 유닛 컷→실제 끝점이 같은 종료 밴드를 소비하도록 보정하며 기존 #5585의 실제 빈 Enter와 여백 중복 반례도 함께 확인한다. 기대값·baseline·검사 이관은 아직 수행하지 않는다.

### EZ/FA: 변형 입력의 실제 종료 여백·마지막 행 조각 정렬

- EY의 원본 시각 통과 이후 변형 출력 보정을 계속한다. `HostSpacing.terminal_outer_bottom_flow`는 저장 줄의 존재와 무관하게 개체의 종료 여백을 생산한다. budget은 빈 Enter가 이미 소유한 밴드와 겹치는 부분을 빼고 terminal overhead에 한 번 예약한다. scan은 같은 예약으로 마지막 유닛 수용을 결정하며 emit의 흐름 끝과 float placement도 이를 소비한다. terminal content cut이 모든 유닛을 소유하는 경우의 종료 판정은 FragmentInput 공통 query를 scan/emit에서 호출하여, 마지막 컷을 유지하면서 여백을 paint 뒤에 추가하지 않도록 연결했다. 기존 #5585 Enter 밴드와 정상 문서를 함께 검증하며, 기대값을 수정하지 않았다.
- EZ Native 실행에서 later 입력의 Table199 종료 occupied673.107→676.880px, outer283HU가 terminal reserved3.773px로 보존됐다(`ez/logs/later-flow.log`). 단, Cell(row3,col1)의 `함유 혼합물`은 bbox y158.7이고 독립Print는211.3576이다. 전체 내용이 늘어난 마지막 행 조각은 높이124.0267px인데, source 높이 override가 없다는 이유로 남은 내용을 모두 담아도 Top 정렬을 강제했다. 정상 remaining row의 수용 상자는 마지막 suffix의 정렬을 소유하므로, 모든 유닛을 담은 일반 행 조각은 원래 Center/Bottom을 소비하도록 FA에서 연결한다. 아직 새 source의 출력·전체 회귀·WASM은 미완료이며 이 문단은 시각 완료 주장이 아니다.

- **FA 후보 철회 및 EY 복원 완료**: 전수10,219실행/10,209PASS/10FAIL/45skip으로 기존2개 외 신규8개 실패(#554, #7336 2개, #7470, #7518 2개, overlap partition11/15)가 검출됐다. 종료 band max값과 전체 유닛 소비만으로 물리 공간의 중복 계상·정렬 권한을 판정한 확장을 철회했다. 이번 반복에서 수정한7개 source 파일만 복원하여 EY runtime patch `65f50cdc…`와 정확히 일치함을 확인했다(`fa/rejected-candidate-restoration.json`). 원본103쪽 보정, 기존 WIP, 모든 검사·기대값·baseline은 유지한다. FA 결과는 통합 후보가 아니다.
- 위첨자 제거 독립2020Print 대조군은 후속 문구의 y232.9783px를 유지해 위첨자가 간격 차이의 원인이라는 가설을 기각했다. 입력208줄을 synthetic으로 표시한 진단에서는 advance21.3→30.9로 바뀌어도 다음209의 current90.1/y227.7은 그대로다. 후단의 vpos 원점 재선택이 앞 흐름 결과를 덮어쓰는 경로를 추적한다. 합성 진단을 정상 생성본 근거로 사용하지 않는다.

- #6778 수동 기준 PDF: 메인테이너 제공 `/mnt/e/tmp/156757920-animal-welfare-husbandry-guidelines.pdf`는 Hancom2020 11.0.0.9136,12쪽이며 내용 배율이 새 MCP2024 출력의 약77%와 달리 정상100% 좌표에 대응한다. 원본 제공본은 수정하지 않고 `ey/square-manual-print/`에 SHA 고정 복사했다. EY source/Native/Docker WASM의 compat2022 전체12쪽 비교는 양 backend가 같고 미달1쪽85.60182%,4쪽66.27270%,6쪽59.78839%,7쪽40.29383%다. 이 추가 대조군은 통과로 판정하지 않는다. 인쇄 배율 차이와 남은 실제 배치 차이를 구분하며, 원본103쪽 통과나 #6778 쪽수 검사PASS로 해당 미달을 상쇄하지 않는다.

- **#6778 회귀 재분류**: 같은 수동2020PDF로 순수devel552f를 새 빌드하여 전12쪽을 비교했다.1/4쪽은 current와PNG동일(85.60182/66.27270%)이나6/7쪽은 devel99.37688/98.21700→EY59.78839/40.29383%로실제새회귀다. 빈 문단57/64가5/6쪽에서6/7쪽으로이월돼뒤제목·표를약26px밀었다. 다른10쪽PNG와 이전devel bfa의12쪽PNG는같다. EV에도같은소유변화가있으므로원본103쪽보정이최초원인이라고단정하지않는다. formal2FAIL 외에이두배치회귀를보정대상으로추가한다. 상세코드원인은아직미검증이며기대값을바꾸지않았다(`regression-attribution-20261013/{report-ko.md,comparison.json}`).

### 2026-10-13 남은 실패 보정: FB / FC Square

FB는 저장 줄 없는 TAC 객체가 새 단으로 이월될 때 이전 단의 문단 앞 간격을 반복 계상하던 측정 오류를 고쳤다. #6778의6·7쪽은 Native/fresh Docker WASM 모두99.37688/98.21700%이며 전12쪽 PNG가동일하다. 관련31검사는29PASS/기존 #6761 변형2FAIL이었다. 원 연구103쪽 Native 전쪽최저90.47526%, 미달0도 유지됐다. 정확한source patch·CLI·WASM해시는 ignored `post-full-regression/fb/provenance.json`에 보존했다.

FC Square의 첫 후보는 정상 저장 TAC 줄 다음의 무저장 줄에 후행 간격을 보존하고, 새 표의 문단 앞 간격·바깥 위 여백을 실제 paint에 적용했다. #6778의4쪽은66.27270→99.90861%로 개선됐고 관련7검사는7PASS다. 1쪽85.60182%는 남았다. 첫 후보는 실제 저장 문단 상자의48188HU 오른쪽을 픽셀 본문 상자의48190HU와 같은 것으로 요구해 인증에 실패했다. 후속 후보는 원본 줄 상자의 유효한 오른쪽 경계에서 같은 원본 후속 줄의 옆 레인을 인증하고, 물리 본문 포함 여부만 독립적으로 검사한다. 실제 표의 양수 오프셋·바깥여백을 포함한 한 후보를 whole-fit과 paint에 공유한다. 이 후속 후보의검증은 진행 중이며 통과로 보고하지 않는다.

#6761은 현재 변형104쪽 Native 전쪽을 다시산출했다.41–44쪽미달은33.05207/18.87016/41.87520/37.26536%로 남으며, 연구 원본103쪽의 통과와 구분한다. 검사 기대값이나baseline은 이번단계에서 아직변경하지 않았다.


## 2026-10-13 남은 실패 보정 — FM 검증 중

원래 devel 대비 회귀22건 중 실제 구현 실패20건은 복구했으며, 추가 #6778 시각 회귀6·7쪽과 기존1·4쪽도 보정했다. FK–FM 반례에서 #7336·#7470의 새 실패를 검출하고 고쳤다. FM 집중검사는53실행/51PASS/2FAIL이며 두 FAIL은 독립 Print와 충돌하는 #6761의40쪽 이월 기대값이다. FM 전체 기본검사는10,219실행/10,216PASS/3FAIL/45skip이다. 추가1FAIL은#6778의가로넘침이없어졌는데도103.9px넘침을요구하는#7051진단검사다. 실제새넘침으로분류하지않는다. 검사변경은아직적용하지않았다.

### 적용 규칙과 실제 소비 경로

- 저장 줄 없는 객체-only TAC가 새 단으로 이월되면 이전 단의 문단-before는 재예약하지 않고 표의 outer top은 보존한다. `table::host_spacing`의 formatted band → `typeset_tac_table`의 새 단 수용 → paint의 column-top 선행 간격과 같은 실제 원점을 소비한다. 뒤의 실제 빈 문단57/64는 삭제하지 않는다.
- 저장 TAC 행의 outgoing spacing과 다음 fresh 객체 행의 before/top은 별개 소유다. `composer::tac_next_line_full_spacing`의 같은 메트릭을 저장 계획과 layout이 소비하며 새 객체 줄은 자체 before/top을 한 번 계상한다.
- 빈 Square host의 원본 full-width 줄 상자와 다음 저장 side strip을 개체의 outer width/vertical band로 인증한다. `stored_empty_square_successor_strip` → block entry의 확정된 `ParagraphFloatPlacement` → `post_flow`의 strip 선택과 layout의 동일 strip/box를 연결했다. `StoredSquareWrap`은 host 흐름을 유지하고 strip 종료에서 실제 개체 하단을 닫는다. source offset의 host 원점과 displaced table top을 섞지 않는다. 전폭 꼬리의 fit은 실제 줄 점유를, 흐름은 말미 줄간격까지 포함한 advance를 쓴다.
- 표의 말미 outer bottom은 실제 조각의 물리 여백/캡션 예약과 이후 owner의 flow 전진을 구분한다. 마지막 행의 paint/ink fit에 flow margin을 일괄 더하지 않는다(#7336의독립7쪽Print). scan이확정한종료컷과emit은같은종료소비질의를사용한다. source closing query가여백을예약한뒤에남은flow margin을계산하며, terminal commit은유효source successor를지우지않는다. spacing 소비자는현재physical bottom과동일할때만앞간격을공유한다. 비끝컷은후속원점을폐기한다. 이경로는#7470의930.5px원점을다시11.4px늘리지않는다.
- 직접 텍스트의 완전한 suffix는 실제로 남은 행 상자에서 원래 center/bottom 정렬을 소유한다. 중첩 개체의 physical frame은 이 직접 텍스트 규칙으로 옮기지 않는다. 저장 줄 원점이 현재 carried frame 이전으로 돌아오면 plain near-top reset으로 새 단을 재종료하지 않는다. 인라인 도형/0 원점/명시적 쪽·단 경계는 별도 계약을 유지한다. HeightCursor는실제column source origin과지연역산원점을구분한다.

### 실행과 증적

FM runtime patch/CLI/Docker WASM은 기본 작업공간의 `output/pr-review/oksure-20261011/pr7704-maintainer/post-full-regression/fm-terminal-contract/provenance.json`에 고정했다. Native 원본103쪽, control103쪽, shifted103쪽, later104쪽은 각각 전쪽 최저90.47526%이다. #6778 전12쪽은98.21700%, #7336 nested6쪽은95.40457%, stored7쪽은96.73688%이며 누락/쪽수 차이0이다. 기존 승인11문서393쪽의 최신Native/freshWASM도 재실행중이다. Rustfmt/기본·WASM·workspace all-targets Clippy/workspace build 및 고정base manifest/unit-tier 검사는exit0이다. Native Skia3종은진행중이다.

실제Studio탭 `http://localhost:7794/?review=pr7704-fm-terminal-validated`에 연구원본103쪽을열었다. 브라우저가받은WASM해시와Docker/public해시의일치를확인했다(`studio-browser-proof.json`). 새WASM전체페이지비교가끝나기전에는최종통과/원격반영준비완료로보고하지않는다. 변경할두#6761검사는실제table/cell owner·원본셀텍스트의순서/보존·셀내줄포함을검사하도록ignored초안을준비했으며진단실행은103/104쪽에서PASS했다. #7051은정상출력의무넘침과의도적으로이동한child ink의검출을함께검사하도록준비했다. 아직정식검사에적용하거나원격댓글/push/PR/merge를하지않았다.


## 2026-10-13 로컬 보정 최종 판정

FM 최종 전체 기본회귀10,219PASS/0FAIL/45skip, Native Skia라이브러리4,046PASS/placeholder2PASS/directPDF4PASS, 최종Rustfmt·세Clippy·workspace build·고정base manifest/unit-tier 모두exit0. Native/freshDockerWASM은각각18문서831쪽전부90%이상(최저90.47526%)이고PNG831쪽이같다. 실제Studio새WASM/연구원본103쪽과대표7쪽review/overlay도확인했다. #6761 두검사는독립Print/전쪽출력보정후소속·포함·내용의순서/완전보존으로기대값을고쳤다. #7051은정상문서의무넘침과의도적child이탈의검출경계를검사한다. baseline/golden/허용치완화나검사이관은없다. 최종증적은기본작업공간 `output/pr-review/oksure-20261011/pr7704-maintainer/post-full-regression/fm-terminal-contract/report-ko.md`와 `provenance.json`이다. 원격반영/새PR/merge는아직수행하지않았다.
