---
kind: snapshot
status: active
canonical: mydocs/pr/archives/pr_7178_review.md
last_verified: 2026-09-16
---

# PR #7178 검토

## 메인터너 보정 재검토 (2026-09-16)

**승인 — lane 소유 혼재 경계의 머지 보류 사유 해소.**

- x 교차를 예약 소유로 간주하던 추정을 제거하고 `FloatLane.control_index`를 두 배치 경로에서
  기록·소비한다. 형제 표의 실제 예약 여부로 lane/block 경로를 구별한다.
- 작은 lane 표 → block 표 → 후속 표를 같은 x에 놓은 편집 IR 계약에서 보정 전
  `601.05 × 384.92px` 겹침이 재현됐고 보정 후 해소됐다. 각 표 1회 배치와 쌍별 점유를 검사한다.
- 원본 44529 p6~9에서 p7/p8 분리와 이웃 순서를 확인했다. #6795 및 #6950 이웃 계약도 포함했다.
  도형선·줄바꿈 등 기존 fidelity 차이는 원본 이슈 전체 해결로 확대하지 않는다.
- 구현 일반성, 점유·배치 일관성, 혼재 경계의 증거 부족을 해소했다. cut/rowspan 계산은
  이 수정의 적용 범위가 아니며 기존 golden/래칫은 변경하지 않았다.

대표 fresh WASM 증적: [44529 p7](../assets/pr7178_maintainer_44529_p007_wasm_review.png),
[44529 p8](../assets/pr7178_maintainer_44529_p008_wasm_review.png).

최종 보정 source에서 focused **80개**, 전체 nextest **9,916개**, Native Skia lib **4,112개**,
그림 **2개**·직접 PDF **4개**가 통과했다. fmt·Rust Clippy 3종·workspace build·suite 정책도 통과했다.
7개 문서 선택 **25쪽**을 Native/fresh WASM Visual Sweep으로 직접 대조했다.
[분석·재현·컷/공간 대조와 검증 기록](../../working/task_m100_7095_6946_maintainer_stage1.md),
[입력/소스 hash·실행 요약](../assets/pr7141_7178_maintainer_validation.json)을 함께 확인한다.
원 PR CI나 초기 검토를 새 통합 head CI 성공으로 표기하지 않는다. 통합 PR CI는 아직 미실행이다.

판정은 **메인터너 보정을 포함한 로컬 누적 통합**에 대한 것이다. 보정 전 원 PR head 자체의
GitHub approve·merge 완료를 뜻하지 않는다. 아래 초기 검토는 당시 증거를 보존한 기록이다.

## 초기 검토 판정 (`a85189909`)

**머지 보류**. 검토 대상은 아래 원 PR 및 누적 통합 code head다.
이 판정은 GitHub approve·push·통합 PR 생성·merge 완료를 뜻하지 않는다.
아래 차단 사항을 해결하고 보정 head의 관련 검증을 완료한 뒤 재판정한다.

## 대상과 적용

| 항목 | 작성 시점 확인값 |
| --- | --- |
| PR | [#7178](https://github.com/edwardkim/rhwp/pull/7178) |
| 제목 | 수정(typeset): block 으로 앉은 자리차지 표 형제가 lane 에 없어 뒤 형제가 같은 쪽에 겹친다 (#6946) |
| 작성자 / reviewer | planet6897, 기존 기여자 / jangster77 요청 및 API 확인 |
| 원 head | `05acbd6f63308b37a8ce365dd09e68aea52a5e52` |
| base / 규모 | devel / 10 files, +259/-22 |
| 통합 base | `263b61a64a77a0679e9d8679c5be2e1d180cee1a` |
| 통합 code head | `21164e71a8a84c6204edcad58723f154256587da` |
| branch | `codex/pr7141-7175-7178-20260916` |
| 원 PR 상태 | OPEN / non-draft / MERGEABLE / CLEAN |
| 원 head CI | [Full CI success](https://github.com/edwardkim/rhwp/actions/runs/34988745897) — run head SHA 일치 확인 |

순서는 #7141 → #7175 → #7178이며 기능·test·문서 12개 commit을 `-x`로 체리픽했다.
#7141의 devel merge commit은 제외했으며 원 head와 최신 base의 merge tree가 해당 체리픽 결과와
바이트 동일함을 확인했다. 세 PR 모두 텍스트 충돌은 없었다. [실행 기록](pr_7178_review_impl.md)에
source/local SHA를 남겼다. 각 원 PR 최신 head의 CI 성공은 통합 head의 CI 성공을 대신하지 않는다.

base route: `collaborator_external_pr.md` (기본 작업공간의 devel 기반 누적 체리픽).
loaded: `pr_review_workflow.md`, `pr_review/README.md`, `collaborator_external_pr.md`,
`intake_and_review.md`, `multi_pr_update_branch.md`, `local_validation.md`, `visual_fixture_evidence.md`.
시각 판정에는 [Visual Sweep 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment)을 적용했다.

## 발견 사항과 해제 조건

### 원본 실물 결함은 개선됐다

44529는 Native/fresh WASM에서 모두 13쪽이다. p7에는 앞쪽 신고서가, p8에는 뒤쪽 동의서가
독립적으로 표시된다. p6·p9의 다음/앞 서식도 확인했다. 원래 879.4px 겹침을 원본 sweep에서
다시 확인한 후 후보와 대조했다. p9의 약 26.1px 본문 넘침은 기존 현상이며 새 baseline 행의
근거와 구분한다. 아래 수치가 낮으므로 선 굵기·글꼴·일부 줄바꿈까지 일치한다고 주장하지 않는다.

### P2 — lane 등록을 소유 정보 없이 가로 겹침으로 추정한다

[typeset.rs](../../../src/renderer/typeset.rs) 21236–21244행의 `lane_registered`는
`whole && lanes.lanes().iter().any(ranges_overlap(...))`이다. 그 lane이 앞의 `PageItem::Table`을
실제로 예약한 것인지 확인하지 않는다. 같은 x 범위에 먼저 놓인 작은 lane 표 A와, 이후 block으로
배치된 표 B가 함께 있으면 A의 lane만으로 B도 등록됐다고 판단할 수 있다.
그러면 `overlaps && !lane_registered`가 false여서 B의 `current_height`를 raw_top에 반영하지 않는다.
원본에는 이 혼재 반례가 없어 원본 4개 및 이웃 5개 테스트 성공으로 해당 경계를 보증할 수 없다.

이것은 **원본 fixture의 새 회귀 검출이 아니라 코드 검토상 잘못된 소유 추정과 경계 증거 부족**이다.
탐색용으로 원본 IR에 작은 표와 축소한 뒤 표를 추가한 경우 큰 겹침을 관찰했지만,
수동 수정한 저장 메타데이터·한컴 생성본 및 수정 전 대조가 확정되지 않아 새 회귀 증거로 채택하지 않았다.
합성 입력을 정상 한컴 저장본으로 격상하지 않는다.

해제: 실제 lane 예약/소유를 연결하거나 block 배치 상태를 직접 전달한다. 같은 x 범위의
lane 표 + block 표 + 뒤 표가 있는 혼재 경계를 실제 제품 경로에서 확인하고,
나란히 놓인 Square 형제·조각 형제·뒤의 짧은 표를 함께 보호한다. 정상 원본 44529의 p7–p9
직접 sweep과 관련 focused tests를 보정 head에서 다시 확인한다. 단순히 원본 속성 조건을
추가하거나 x 겹침 허용치를 바꾸는 방식으로 소유 추정을 덮지 않는다.

## baseline 추가

`oracle_page_count_baseline.tsv`에 새 fixture의 `13/13`, `body_overflow_baseline.tsv`에 `1`을
처음 기록한다. 기존 행의 허용치를 늘리지 않았다. 같은 입력의 기준 바이너리와 후보에서 p9에
해당하는 다음 서식의 기존 26.1px 넘침을 구분한다. 신규 fixture 보안 검사는 실제 경로를
`RHWP_SECURITY_SWEEP_SAMPLES_JSON`에 넣어 실행했고 security 묶음 6개가 통과했다.

## 실제 검증

- 전용 target: `target/pr7141-7175-7178-20260916`. 기존 target과 실행 중 Cargo/Rust 작업을
  확인하고 별도 경로를 사용했다. Cargo 작업은 순차 실행했다.
- `cargo build --locked --profile release-test --target-dir target/pr7141-7175-7178-20260916`: exit 0.
- 관련 9개 case를 manifest에서 실제 suite로 해석해 `cargo nextest run --locked --cargo-profile
  release-test --target-dir target/pr7141-7175-7178-20260916 --test <해석된 suite> --no-fail-fast
  -E <case별 OR 필터>`로 실행: **51 passed / 1522 filtered skips**, exit 0.
  #7095 3, #4771 6, #6924 1, #7086 4, #6946 4, #6795 5, #6981 8, layout_anomaly_contract 14,
  security_corpus_regression 6이다. security 입력은 새 44529 fixture 1개다.
- 첫 실행의 #6981 terminal-cut 테스트에 LEAK 표시 1건이 있었다. 해당 테스트만 같은 target에서
  재실행하여 **1 passed / leak 표시 없음 / exit 0**을 확인했다. 첫 표시를 숨기거나 실패로 바꾸지 않는다.
- `cargo fmt --all -- --check`, manifest 및 unit-tier `--check --base-ref 263b61a64a77a0679e9d8679c5be2e1d180cee1a`: 모두 exit 0.
- Docker daemon 연결 불가를 확인했다. `CARGO_TARGET_DIR=target/pr7141-7175-7178-20260916
  scripts/wasm-pack-locked.sh --target web --out-dir /private/tmp/pr7141-7175-7178-wasm --no-opt`: exit 0.
  Chrome에서 새 패키지의 SVG와 render tree를 직접 얻었다. 최적화된 Docker 배포 빌드 통과로 표기하지 않는다.
- 전체 Rust/Native Skia 회귀는 사용자의 CI 중복 실행 생략 지시를 따랐다. 원 head의 정확한
  Full CI·별도 checks 성공을 확인했으며 **새 통합 head CI는 아직 실행하지 않았다**.
  이번 단계는 PR 검토다. 통합 PR 제출 전 Rust lint 3종 및 적용 게이트를 통과해야 한다.
- code/test/fixture/baseline 메인터너 보정은 추가하지 않았다. 탐색용 임시 IR probe는 정상 한컴
  입력 계약·수정 전 음성 대조가 확정되지 않아 수용 또는 신규 회귀 증거에서 제외했다.

## 초기 공통 조판 원칙 검토

| 항목 | 판정 | 근거 |
| --- | --- | --- |
| 구현 근거와 일반성 | 미충족 | 한컴 원본·PDF 근거 존재. 다만 위 P2의 분기/소유 추정 경계 증거 부족 |
| 측정·배치 일관성 | 미검증 | 원본 집중 경로는 확인했지만 위 P2의 혼재 경로까지 공통 결과로 입증되지 않음 |
| 분할·이어받기 계약 | 비해당 | 7178은 통째 Table 형제의 lane 진입을 바꾸며 cut/rowspan 계산은 변경하지 않음 |
| 줄 소속과 점유 높이 | 미검증 | 실물·이웃 focused 통과. 통합 51개 성공을 미실행 경계의 증거로 확대하지 않음 |
| 사례와 증거의 독립성 | 충족 | 원본/한컴 PDF와 통합 head의 Native/fresh WASM 직접 대조. 합성·탐색 제외 |
| 기준값 변경 | 충족 | 위 기준값 변경 절 참조 |
| 주장과 검증 범위 | 미검증 | 원본 시각 개선·잔여 fidelity·코드 우려·미검증 경계를 별도로 기록 |

## 검증 입력 커밋 확인

**충족** — 아래 실행 파일은 통합 code commit의 blob과 byte 단위 동일하다.
기존 입력을 재사용했고 같은 내용의 HWP/HWPX/PDF 사본을 추가하지 않았다.
테스트가 메모리/임시 파일로 생성한 계약 입력은 해당 커밋의 test source로 재현한다.

| 경로 | SHA-256 |
| --- | --- |
| [samples/issue6946/44529-logistics-policy-rule-amendment.hwp](../../../samples/issue6946/44529-logistics-policy-rule-amendment.hwp) | `e7225ebb6d97981ffb10c98fbb6a639ef2efe77e43ca30d11d4fd172c3c2c1d6` |
| [pdf/issue6946/44529-logistics-policy-rule-amendment-hwp-2020.pdf](../../../pdf/issue6946/44529-logistics-policy-rule-amendment-hwp-2020.pdf) | `a1b36d69d544cb68f0d9ba84391fcb366e2dacf8887e62a1eaf71ed3c2719eb1` |
| [samples/issue6795/1341000-201100013-cyber-university-application.hwp](../../../samples/issue6795/1341000-201100013-cyber-university-application.hwp) | `3202819ec9712c49b189ecb0e1b4a2d46aba37d01e1654c6917438e8134f42d8` |
| [samples/issue2813/dangjik_dutylog.hwpx](../../../samples/issue2813/dangjik_dutylog.hwpx) | `d785a49b62c43ed5b4602509657a112742d916782828022a355b916bcc19a4c5` |
| [samples/task2287/1342000_edu_curriculum_map.hwp](../../../samples/task2287/1342000_edu_curriculum_map.hwp) | `623b00d56beffc45d27c5bf23911bdc49d3a541ded8aecbb323d0716a2bc9f4e` |
| [samples/issue6803/1376496-neighborhood-facility-land-table.hwp](../../../samples/issue6803/1376496-neighborhood-facility-land-table.hwp) | `2482b695bc92c0cafe661af63c54acea834b6927bf9c666bdf47299910e3801d` |

## 직접 Visual Sweep

명령: `venv/bin/python scripts/visual_sweep.py --file-target <key> <위 원본> <위 PDF> --rhwp-bin target/pr7141-7175-7178-20260916/release-test/rhwp --pages <아래 쪽> --dpi 96 --out <아래 root>`; WASM은 `--wasm-pkg /private/tmp/pr7141-7175-7178-wasm`을 추가했다.

Native root `/private/tmp/pr7141-7175-7178-native`, WASM root `/private/tmp/pr7141-7175-7178-wasm-sweep`. 각 `<root>/<key>` 아래 `compare/compare_NNN.png`, `overlay/overlay_NNN.png`, `review/review_NNN.png`가 있다.

| key | 쪽 | pixel_match % | visual_accuracy_proxy_percent | flags |
| --- | ---: | ---: | ---: | --- |
| pr7178-44529 | 6 | 91.56317 | 39.52441 | 0 |
| pr7178-44529 | 7 | 86.33765 | 18.17107 | 0 |
| pr7178-44529 | 8 | 95.83250 | 56.06163 | 0 |
| pr7178-44529 | 9 | 94.86756 | 7.41468 | 0 |

Native/WASM의 위 지표는 동일하다. 11쪽 전체의 트리는 내부 usize sentinel의 32/64bit 차이를 제외하면 동일하다. 7062 4쪽분에서만 sentinel 차이가 있으며 나머지 7쪽은 raw JSON도 동일하다. 전체 fidelity 통과가 아니고 위 직접 판정 범위만 인정한다.

`fidelity_compare.py --text-only --export-all-svg --layout-ledger`도 각 원본의 전 페이지(10/22/13)를 실행했다. 원장은 `/private/tmp/pr7141-7062-ledger`, `/private/tmp/pr7141-30269-ledger`, `/private/tmp/pr7178-44529-ledger`에 있다. 7062 p3→4·p8→9에 표 내용 owner 이동 후보 2건, 나머지 두 문서는 page-boundary 후보 0건이다. 후보 0을 시각 일치로 해석하지 않는다.

기준 바이너리는 직전 PR #7179에서 검증한 `60a3ad32e` code 산출을 재사용했다. 이 SHA와 현재 base 사이 `src`, `crates`, Cargo.toml/lock diff가 없음을 확인했다. 위치는 `/Users/tsjang/.Trash/rhwp-pr7179-target-20260916-014502/release-test/rhwp`이며 복구·재이름변경하지 않았다. 기준 sweep은 `/private/tmp/pr7141-7175-7178-base`에 보존했다.

PDF provenance: 모두 PDF 1.6, 595×841pt이다. 7062(10쪽)·30269(22쪽)는 Creator `Hwp 2022 0.0.0.0`, Producer `Hancom PDF 1.3.0.550`; 44529(13쪽)는 Creator/Producer `Hancom PDF 1.3.0.550`. `rhwp info`의 저장 제품은 7062·44529 `hancom-office-2010`, 30269는 product null/version `7.5.12.614`로 2020 bucket에 해당한다. 기존 파일을 재사용했으며 이번에 MCP 재변환하지 않았다.

## Merge 후 contributor PR comment 계획

현재는 머지 보류다. 해제 후 최신 보정 head의 수치·판정을 갱신하고, 아래 asset이 merge SHA로 devel에 존재할 때만 comment를 게시한다. 지금 수치를 보정 후 통과 증거로 재사용하지 않는다.

- [Visual Sweep 정본](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)을 직접 링크한다.
- 위 실제 페이지·flags·pixel_match·내용 픽셀 보조값과 사람 판정/잔여 차이를 함께 적는다.
- 대표 [WASM pr7178-44529 p7](../assets/pr_7178_44529_wasm_p007.png); raw URL 형식 `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/pr_7178_44529_wasm_p007.png`.
- 대표 [WASM pr7178-44529 p8](../assets/pr_7178_44529_wasm_p008.png); raw URL 형식 `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/pr_7178_44529_wasm_p008.png`.
- 대표 [WASM pr7178-44529 p9](../assets/pr_7178_44529_wasm_p009.png); raw URL 형식 `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/pr_7178_44529_wasm_p009.png`.

대표 이미지의 본문·라벨·한글·보조 수치를 직접 확인했다. 보조값은 사람 판정 정확도가 아니다.

원격 comment는 merge 승인과 실제 병합 후 UTF-8 본문 파일을 `--body-file`로 게시하고 API로 재조회한다. 이번 검토에서는 reviewer 지정 외 원격 변경·push·comment·close·merge를 수행하지 않았다.
