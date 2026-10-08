---
kind: review
status: active
last_verified: 2026-10-08
---

# PR #7644 검토 — 통째로 들어가는 RowBreak 빈 앵커 표의 바깥 여백

## 최종 판정

**머지 보류.** 쪽 경계 나눔을 허용한 표도 실제로 분할되지 않으면 바깥 여백을 포함한 상자를 점유한다는 수정 근거는 확인했다. 생성 재현의 Native 출력은 개선됐다. 그러나 기존 실물 문서의 본문 넘침이 0→1건으로 증가했고, 이를 baseline에 추가했다. 다른 실물 문서 20쪽은 90% 시각 게이트 미달을 재현했다. 구현의 일반적인 규칙 개선과 이 두 미충족을 구분한다.

작업지시자의 이번 지시는 검토 시작이다. 별도의 merge·시각 예외 승인은 받지 않았다. reviewer 지정 외에는 원격 변경을 하지 않았고, 보완 의견은 로컬 초안으로만 보존했다. 코드·테스트·baseline을 메인터너가 수정하지 않았다. [#7620](https://github.com/edwardkim/rhwp/issues/7620)은 부분 해결 범위의 `Refs`로 유지하며 종료하지 않는다.

## 접수와 고정한 대상

| 항목 | 확인 결과 |
| --- | --- |
| PR / 작성자 | [#7644](https://github.com/edwardkim/rhwp/pull/7644), sacru2red; 기존 기여자 |
| head | `5cd52f83aaed82bbaad1d4331587002831c27392`, `sacru2red/rhwp:fix/issue-7620-rowbreak-float-margins` |
| API base / 최신 로컬 base | devel `b3c3047db575d146dff9d39098e6de98c4630b3e` / `f0e7228f6dd2ea1437724e53ad640c40c56d204b` |
| 규모 | PR 고유 5 commits, 21 files; production 2 files, 회귀 1 source·2 tests, 기존 baseline 1 row, 입력·PDF·PNG·생성 절차 |
| reviewer / 경로 | edwardkim 지정 확인; [maintainer_general](../manual/pr_review/maintainer_general.md), [intake_and_review](../manual/pr_review/intake_and_review.md), [local_validation](../manual/pr_review/local_validation.md), [visual_fixture_evidence](../manual/pr_review/visual_fixture_evidence.md) |
| merge 상태 | open / non-draft / mergeable=true / unstable; CI Impact Policy pending |
| 최신 base 충돌 simulation | `git merge-tree --write-tree f0e7228f6dd2ea1437724e53ad640c40c56d204b 5cd52f83aaed82bbaad1d4331587002831c27392` exit 0, tree `91af755e67005c8fece9750e7b473ec5dbfd9feb` |
| worktree | `/tmp/rhwp-pr7644-review-20261008`, `review/pr7644-20261008` |

증적 루트는 기본 작업공간의 ignored `output/pr-review/pr7644-20261008/`다. 아래 상대 증적 경로는 모두 이 디렉터리 기준이며 Git에 포함하지 않는다. 기존 Studio 서버와 다른 review worktree, 공용 `target/pr-review`는 보존했다.

## 조판 규칙과 실제 소비 경로

입력은 비TAC `TopAndBottom`, Para 기준, 빈 저장 앵커 문단의 표다. 기존 None 경로와 달리 RowBreak whole-fit 경로는 저장 원점과 위·아래 바깥 여백을 포함한 공통 상자를 기록하지 않았다. 수정은 쪽 나눔 속성 자체를 실제 분할 여부의 대용으로 삼던 차이를 제거한다. 문서 ID·특정 픽셀 값으로 적용 대상을 고르는 production 조건은 추가하지 않았다.

| 단계 | 정확한 head의 코드와 검토 |
| --- | --- |
| 저장 원점의 유효성 | `src/renderer/typeset/table/block/entry.rs:1524`의 단일 단·편집되지 않은 저장 사다리에서 `source_host_origin`을 얻는다. 폭 0만으로 실제 흐름과 원점이 같다고 판단하지 않는다. |
| 공통 상자 생산 | `src/renderer/float_placement.rs:2349`의 `from_saved_whole_empty_host` → `empty_host_box:2373`. 적용된 문단 위 간격을 저장 원점에서 빼고 바깥 위 여백·signed offset·effective height·tail을 반영한다. `saved_empty_table_anchor_props:3264`는 저장 정보 dirty·implementation tag·음수 offset 등을 제외한다. |
| 예산과 whole-fit | `entry.rs:1792–1827`에서 legacy whole-fit·비None·다른 배치 없음·strict following text 계약 제외를 확인한다. exclusion 제약을 적용한 뒤 같은 `whole_frame_budget`으로 `occupied_bottom`을 검사한다. |
| 요구 높이와 누적 예약 | whole-fit일 때 결과를 기록해 `place_table_with_text`로 전달한다. `src/renderer/typeset.rs:5018`의 빈 host 흐름은 `placement.occupied_bottom`까지 전진한다. |
| 실제 표 원점 / 최종 하단 | `src/renderer/layout.rs:12125`는 `col_area.y + placement.table_top`, `:13337`은 같은 anchor, `:13836`은 `col_area.y + placement.occupied_bottom`을 소비한다. 해당 경로는 뒤의 legacy 원점 추정으로 공통 결과를 덮어쓰지 않는다. |
| 예산 실패 경계의 우려 | `.filter`가 바깥 여백까지 안 들어가는 상자를 버린 뒤 `map_or(legacy_whole_fits, …)`가 종전 whole-fit을 다시 수용할 수 있다. 이 fallback에 대한 독립적인 아래 여백 면제 근거·경계 출력은 없다. 실행으로 검출한 새 결함이라고 단정하지 않으며 **코드 검토상 미검증**으로 남긴다. |
| 실제 분할·이어받기 | 새 결과는 whole-fit에만 기록된다. 행 컷·rowspan 소유·first fragment의 기존 준비 경로는 변경하지 않았다. whole/split 선택에 영향은 있으므로 원래 높이만 fit하는 예산, CellBreak, 실제 이월 출력의 검증이 필요하다. 공통 helper가 있다는 이유로 분할 경로까지 같은 결과를 소비한다고 판정하지 않는다. |

문단 아래 간격과 바깥 아래 여백의 구분은 간격 있는 `hwpctl_API_v2.4.hwp` 60쪽 대조군으로 검사했다. RowBreak 생성본은 아래 빈 문단과 뒤 텍스트의 실제 위치도 비교했다. 음수 offset은 속성 계약상 비적용이지만 그 대조군을 이번에 실행하지 않았다.

## 입력 출처와 독립 기준

8개 입력·PDF 파일 모두 `git show <head>:<path>`와 로컬 바이트의 SHA-256 일치를 확인했다(`input-provenance.json`).

| 입력 / 기준 PDF | SHA-256과 출처 |
| --- | --- |
| `samples/issue7620/a-rowbreak-float.hwp` / `pdf/issue7620/a-rowbreak-float-hancomdocs.pdf` | 입력 `44c065e5a770e88903e314b53e1b199f2ab1385e50a79eebb5e8ae8716ec64a3`, PDF `901c26db10c91b3e2284f5506af5c18e7e5c78525ad1b6b9dcd3ea83e73e08e7` |
| `samples/issue6465/press_release_footer_logos.hwpx` / `pdf/pr6485-visual/pr6485-issue6465-press-release-footer-logos-2020.pdf` | 입력 `89ec9065432547ae141328dfb957981a26f9dd1b17b9bba9d721e5ae4fbbf5fa`, PDF `d5a3db9320675907e92a3407d75ce0726fbe605e5bf19fe9bdc46befd07d1487` |
| `samples/issue6111/56345_regulatory_impact_analysis.hwp` / `pdf/issue6111/56345_regulatory_impact_analysis-hwp-2020.pdf` | 입력 `58013017c3a3dc7e2d278b99c5b4fa1c61de0aa861f913a2c41a49145baadafc`, PDF `c66ea20c3b8d6b31af73752e170372bc7af839c38812991125bdde4dfba35abb` |
| `samples/hwpctl_API_v2.4.hwp` / `pdf/hwpctl_API_v2.4-hwp-2020.pdf` | 입력 `d11dd1331083be4e8c989dfbd587777626b3d77686d3436c35a2c20da9494603`, PDF `1d289727dd40ed35e48135bf16df06fe4cd080d967441ff464fb0e0b205fae74` |

신규 RowBreak 파일은 `d9749c5a15b95206455953494f6fc871c115cef1`의 rhwp API 생성본이다. 한컴 저장본으로 승격하지 않는다. [입력 설명](../../samples/issue7620/README.md)과 [manifest](../../samples/issue7620/MANIFEST.json)는 생성 절차와 한컴독스 웹서비스 6.3.6 / Web v2 빌드 `20260812021742`의 서버 Print PDF를 구분한다. LineSeg를 수동 수정한 입력이 아니며, 데스크톱 Print 출력과 CellBreak의 독립 PDF는 미검증이다.

footer 문서는 PR 본문이 기준 PDF 부재로 분류했지만 저장소에 동일 입력에 대한 독립 PDF가 있다. [#6481 기록](archives/pr_6481_planet6897_visual_sweep.md)의 MCP job `1bbb1b6c-04ea-4922-9a5c-a1b6e7da891f`와 [#6485 기록](archives/pr_6485_planet6897_visual_sweep.md)이 위 두 해시를 고정한다. 기존 PDF를 그대로 재사용했으며 새 출력으로 가장하지 않는다. PDF의 Creator 연도만으로 대응이 확인된 기준을 배제하지 않았다.

## 실행으로 확인한 결과

비교 전 source는 최신 devel `f0e7228f6dd2ea1437724e53ad640c40c56d204b`, 비교 후 source는 PR head `5cd52f83aaed82bbaad1d4331587002831c27392`다. 각 source에서 `release-test` Native CLI를 실제 빌드하고 별도 ignored 바이너리로 복사했다. 공용 target은 `/home/edward/mygithub/rhwp/target/pr-review` 하나만 사용했다.

- `rhwp-before` SHA-256: `0e6adc559a0aa3c46d115054fc81fe9a543223542f0ec2339e811f6ea6fddf3f`
- `rhwp-after` SHA-256: `7f987a55bfd0da3b32453c40f7049b90414214c4d673b0edb4782d83b2f456ae`

| 검사 | 실제 결과 |
| --- | --- |
| Native build 전·후 | 둘 다 성공; `base-build.log`, `focused.log` 및 각 sweep 로그 |
| PR 신규 회귀 | wrapper로 `issue_7620_rowbreak_float_table_outer_margins` 실행: 2 PASS / 0 FAIL / 222 filtered skip; `focused.log` |
| 관련 회귀 | #2439 4건 + #7571 saved empty anchor 1건: 5 PASS / 0 FAIL / 452 filtered skip; `related-focused.log` |
| integration manifest | `--prepare` 뒤 `--check --base-ref f0e7228f6…` PASS; 1487 sources / 6395 test attributes / 28 suites / 20 exceptions / 48 targets; `manifest-check.log` |
| 본문 넘침 검사 | 동일 입력 11쪽, tolerance 2px: 수정 전 0 / 수정 후 1, +4.52px; `overflow-before-tol2.json`, `overflow-after-tol2.json` |
| 충돌 / whitespace | 최신 base merge-tree exit 0; PR diff check 통과 |

### Native Visual Sweep

print profile / 96dpi / webfont rasterizer / `RHWP_FONT_PATH=/mnt/c/Windows/Fonts`, 동일 입력과 독립 PDF를 사용했다. 현재 head에서 PNG를 다시 만들었고 compare·standalone overlay·review를 직접 판독했다. 아래 수치는 **2px 이웃 관용 내용 실루엣 일치율**이며 전체 문서 평균이 아니다.

| 입력·쪽 | 수정 전 | 수정 후 | 전체 쪽수 Native / PDF | 직접 판독 |
| --- | ---: | ---: | --- | --- |
| RowBreak 생성본 1쪽 | 31.62913% | 100.00000% | 1 / 1 | 표 시작·아래 여백·빈 문단·뒤 텍스트가 기준과 정렬 |
| footer 문서 11쪽 | 87.22658% | 96.85232% | 13 / 13 | 표 위·중간 괘선은 개선; 하단은 PDF보다 약 9px 아래 |
| 56345 문서 20쪽 | 90.12195% | **89.75606%** | 21 / 21 | 첫 표 원점은 개선; 아래 표 행 높이·줄바꿈 차이가 남음 |
| hwpctl API 50쪽 | 이번 비교 전 실행 없음 | 98.81055% | 105 / 105 | 표·뒤 Return/Remarks 관계 유지 |
| hwpctl API 60쪽 | 이번 비교 전 실행 없음 | 99.72769% | 105 / 105 | 문단 간격 있는 표와 뒤 문단의 배치 유지 |

출력 경로는 다음과 같다. 각 디렉터리에 `run_manifest.json`, `native-export.json`, `overlay/overlay_metrics.json`, render tree와 compare도 보존했다.

- 재현: `native-before-repro/rowbreak-before/review/review_001.png`, `native-rowbreak-after/rowbreak-after/review/review_001.png`
- footer: `native-before/footer-before/review/review_011.png`, `native-footer-after/footer-after/review/review_011.png`
- 56345: `native-before-regulatory/regulatory-before/review/review_020.png`, `native-regulatory-after/regulatory-after/review/review_020.png`
- API: `native-hwpctl-after/hwpctl-after/review/review_050.png`, `review/review_060.png`

56345 수정 후 summary의 gate는 `re_review_required`, 해당 sweep 종료 코드는 1이다. 다른 수정 후 3개 sweep은 exit 0이다. 환경·빌드 실패를 시각 미달로 세지 않았다. 기여자 첨부 `issue7620-native-tsv.zip`도 해시 `2e0d1125dc20d874bff047b283a817fd09cf4fa575f25499bbd2068c7483b5e3`를 확인하고 원본 경로로 짝을 찾았다. 첨부의 같은 20쪽은 90.00239→89.67579%로 본문에 이미 보고됐으며 로컬 재실행도 같은 게이트 경계 미달을 재현했다.

### baseline에 추가한 넘침의 독립 대조

`tests/fixtures/body_overflow_baseline.tsv`에 `issue6465/press_release_footer_logos.hwpx` 1건을 추가했다. 이 행은 문서별 허용 건수의 예외이며 production의 바깥 여백 규칙과 성격이 다르다. 아래처럼 실제 오류 크기가 증가했다.

| 값 | 수정 전 | 수정 후 / 독립 기준 |
| --- | ---: | --- |
| Native 표 상단 y | 122.88px | 126.65333px |
| Native 표 높이 | 924.8px | 924.8px; 선언 높이 916.48px보다 약 8.32px 큼 |
| Body 하단 | 1046.93333px | 동일 |
| 본문 밖 초과 | 약 0.75px, tolerance 미만 | 4.52px, tolerance 초과 |
| Raster 괘선 하단 | Native 약 1047px / PDF 약 1042px | Native 약 1051px / PDF 약 1042px |

한컴 PDF는 위 여백 복원 방향을 지지하지만 표 높이 초과와 baseline 증가를 정당화하지 않는다. 위·중간 괘선 정렬 개선이 표 하단의 무회귀를 입증하지 않는다. 동일 입력의 측정·행 높이·실제 paint 하단의 차이를 해결하거나 독립 출력에 부합하는 기준 변경 근거가 필요하다. 16개 전체 baseline partition을 로컬에서 다시 실행한 것으로 보고하지 않는다. 이번 실행은 baseline과 같은 2px 경계의 layout-anomaly 전후 대조다.

## CI 상태와 재사용 범위

[최신 CI 37637641075](https://github.com/edwardkim/rhwp/actions/runs/37637641075)는 success다. preflight job `112848041070`은 원 candidate `7777562775e17707e1c936cc3f6d41b61c188418`의 [Full CI 37621970490](https://github.com/edwardkim/rhwp/actions/runs/37621970490)를 신뢰 재사용했다. 원 Full의 4 archive worker·Native Skia·Lint·Frontend가 성공했으며 현재 candidate와의 production 2파일·신규 회귀 source·baseline diff는 0이다. 최신 preflight 로그는 `latest-preflight.log`에 있다.

현재 head commit status의 **CI Impact Policy는 pending**이다. 연결된 [controller run 37638385801](https://github.com/edwardkim/rhwp/actions/runs/37638385801)은 completed/success지만 commit status를 성공으로 갱신한 증거가 없다. 실제 상태와 CI aggregate의 success를 구분한다. 승인 직전에는 최신 head의 required status를 다시 확인해야 한다.

메인터너 source 변경이 없어 local Full nextest·Native Skia·세 Clippy 전체 묶음의 중복 실행은 하지 않았다. 원 Full CI 재사용을 새 source에 대한 실행으로 과장하지 않는다. focused와 영향 페이지 직접 Sweep은 별도로 수행했다. fresh WASM은 이번에 빌드·실행하지 않았다. 필수 시각 게이트 미달과 넘침 증가를 먼저 재현해 보류 근거를 확정했으며, 현재 상태를 Native/fresh WASM 최종 승인으로 보고하지 않는다.

PR 본문은 아직 `777756277…`를 대상 head로 적고 PNG raw URL도 그 SHA에 고정돼 있다. source가 같아도 현재 review head와 증적의 표시는 구분해야 한다. 보완 뒤 새 code head에서 재출력하고 본문 이미지 URL도 그 exact head로 갱신해야 한다.

## 원칙별 판정과 보류 해제 조건

| 항목 | 판정 | 근거 / 필요한 보완 |
| --- | --- | --- |
| 일반 조판 규칙의 근거 | 충족, 확인 범위 한정 | RowBreak whole-fit의 독립 Print PDF와 저장 간격 있는 대조군; 생성본을 일반 한컴 저장본으로 승격하지 않음 |
| whole-fit 공통 결과의 소비 | 충족, 적용 경로 한정 | table_top·occupied_bottom의 예약→paint 소비 추적; 실제 splitter의 공통 결과라고 주장하지 않음 |
| 실제 기존 문서의 무회귀 | 미충족 — 실행 결함 | footer 본문 넘침 0→1, 실제 초과 증가; baseline 추가로 숨기지 말고 높이/배치 원인을 처리 |
| Native 90% 시각 게이트 | 미충족 — 실행 결과 | 56345 20쪽 89.75606%, 구조 차이 직접 확인; 현 head 재출력으로 해소 필요 |
| budget 실패 fallback / CellBreak / 실제 이월 | 미검증 — 코드 경계·증거 부족 | 바깥 여백 포함 예산 실패와 아래 여백 면제의 독립 근거, 비적용 대조군·내용 보존 출력 필요 |
| 수정 전 FAIL / 수정 후 PASS | Native 관측 충족 / Rust negative 미검증 | 생성본 위치·뒤 문단의 전후 실패·개선 직접 관측; 신규 Rust 2검사를 수정 전 source에서 실행하지 않음 |
| fresh WASM·전체 영향 범위 최저값 | 미검증 | 이번 Native 5쪽은 whole-corpus나 fresh WASM 최종 게이트를 대신하지 않음 |
| 새 회귀·baseline 추가 선행 조건 | 미충족 | 관련 실물 출력 중 90% 미달·fresh WASM 미실행이 남음; 기존 검사 삭제나 허용치 완화로 해결하지 않음 |
| 원격 승인·통합 | 미충족 | CI Impact Policy pending, 사용자 merge 승인 없음, 본문 증적 head 갱신 필요 |

기여자에게 권고할 순서는 (1) footer 표 높이·점유 경계 해결과 임의 baseline 추가 철회, (2) 56345 20쪽 구조 차이 처리, (3) budget 실패 경계·CellBreak 주장 범위의 독립 근거와 실행 증거 보완, (4) 새 head의 Native/fresh WASM 영향 페이지 및 전체 필요한 범위 게이트 통과, (5) 정확한 head로 본문 PNG·CI 상태 갱신이다. 전체 검증은 작은 경계와 직접 이미지 확인 뒤 수행한다.

보완 의견 초안은 ignored `review-comment-ko.md`에 준비했다. 게시·approve·push·close·merge는 하지 않았다. 원본 증적과 review worktree를 보존한 채 작업지시자의 다음 지시를 기다린다.

## 후속 — Docker WASM과 Studio 확인 준비

2026-10-08 작업지시자가 Studio 확인 준비와 Docker 빌드 지침 준수를 요청했다. 처음 선택한 native `--no-opt` 진단 경로는 이 작업의 표준 경로와 맞지 않아 최종 확인에 사용하지 않았다. 아래 Docker 서비스로 최적화까지 다시 빌드했고 exit 0 / 7m35s 완료를 확인했다. 초기 검토의 fresh WASM 미실행 기록과 이 후속 준비를 구분한다.

```bash
docker compose -p rhwp --env-file .env.docker run --rm --no-deps wasm
```

기본 `rhwp` 프로젝트의 `rhwp_wasm-target`, cargo/cache volume을 재사용했다. `/app`은 이 review worktree다. 이미지 `rhwp-wasm:latest`의 ID는 `4824b312625eba7b8146aa1c7eaca95710618beb75c4cec15470cdb76bc2d99b`, wasm-pack 0.15.0 / Rust 1.93.1이다. Docker 서비스가 저장소 루트의 locked wrapper를 실행하고 `pkg/`와 Studio `public/`을 동기화했다. production source는 검토 head `5cd52f83aaed82bbaad1d4331587002831c27392`와 동일하고 추가 변경은 로컬 review 문서뿐이다.

| 준비·실행 | 확인 결과 |
| --- | --- |
| JS SHA-256 | pkg / public 모두 `70cde06a369fa7fd4fc8bc8f3d6acaee158596ba1a116a2c72159002b0b5654e` |
| WASM SHA-256 | pkg / public / 두 Chrome 탭의 실제 instantiateStreaming 입력·network response 모두 `e357f3a63c83fff7f4e3f172ae9a4f7f0248567d42fa3ddf0d07cfa14fd410c3` |
| 전용 서버 | `http://localhost:7799/`, 이 worktree의 Vite `--host 0.0.0.0 --port 7799 --strictPort`; 기존 7700 / 7798 서버 유지 |
| footer | 실제 파일 입력으로 로드, 13쪽, `11 / 13 쪽`으로 이동; `studio-footer-page11.png` |
| 56345 | 실제 파일 입력으로 로드, 21쪽, `20 / 21 쪽`으로 이동; `studio-regulatory-page20.png` |
| 브라우저 | CDP 19222, print profile, Local Font Access 518 faces / 함초롬바탕 확인, font status loaded, 두 탭 pageerror 0 |
| 실행 증적 | `studio-docker-wasm-build.log`, `prepare-studio.cjs`, `studio-browser.log`, `studio-session.json`, `studio-vite.log` |

두 검증 탭을 Chrome에 남겨 메인터너가 직접 판정하도록 했다. screenshot도 열어 문서·대상 쪽의 표시를 확인했다. 이 준비는 fresh WASM Studio의 실제 로딩 확인이며 independent PDF와의 fresh WASM Visual Sweep 전체 게이트를 수행한 것은 아니다. 기존 넘침·Native 20쪽 게이트 미달과 보류 판정은 별도 승인·보완 전까지 유지한다. 초기 response-body 수집 도구 오류는 `studio-browser-attempt1.log`에 보존했고, 재시도에서 실제 인스턴스 입력 해시와 network response를 모두 확인했다.
