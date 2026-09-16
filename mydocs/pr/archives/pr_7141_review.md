---
kind: snapshot
status: active
canonical: mydocs/pr/archives/pr_7141_review.md
last_verified: 2026-09-16
---

# PR #7141 검토

## 메인터너 보정 재검토 (2026-09-16)

**상자 기하·예산의 부분 개선 범위에서 승인 — 초기 머지 보류 사유 해소.**

최신 원 head `d24030229`의 추가 증적·주석도 `4670dce74`로 반영했다.
실행 동작은 검증한 보정 `6cdca9464`와 동일하다.
[추가분 검증·최종 적용 기록](../../working/task_m100_7095_6946_maintainer_stage2.md).

- P1: `valign`·기존 페이지별 내용 이동까지 해결한 것으로 주장하지 않는다. #7095를 OPEN으로
  유지하고 통합 PR 본문·최종 squash 모두 `Refs #7095`를 사용한다. 원 commit의 closing
  키워드를 최종 메시지에 자동 복사하지 않는다.
- P2: 확정 host 원점에 outer-top을 한 번만 계상하고, replacement budget도 paint와 같은
  물리 하단을 사용한다. 실제 23.76→21.88px 이중 여백 반례와 빈/텍스트 host 62개 경계에서
  유닛 1회 표시·셀 내부·계속 원점·후속 본문을 검증했다. 저장 컷 advance의 뒤 줄간격을
  잘못 빼는 초안은 기각하고 80168 157쪽·rowbreak 18쪽을 보호했다.
- 추가 시각 회귀: 파생 그림 줄을 저장 쪽 프레임으로 오인하던 확대를 제거했다. 같은 컷의
  원 문단/줄 출처를 소비하며 issue2004 p5 실제 테두리가 약 93px 늘던 문제를 해결했다.
  [보정 전](../assets/pr7141_maintainer_2004_p005_before_compare.png) /
  [보정 후](../assets/pr7141_maintainer_2004_p005_after_compare.png).
- 일반성/공통 결과/분할 계약은 위 구현·음성 대조·62개 경계·실물 PDF로 보완했다.
  합성 IR을 한컴 정답지로 승격하지 않으며 기존 golden/래칫은 변경하지 않았다.
- 남은 fidelity: 7062의 `valign`, p3/p10 내용 이동, 저자 증적의 기존 이탈 크기 변화는
  #7095에 남긴다. 전체 PDF 시각 일치 승인이 아니다.

대표 fresh WASM 증적: [7062 p2](../assets/pr7141_maintainer_7062_p002_wasm_review.png),
[issue2004 p5](../assets/pr7141_maintainer_2004_p005_wasm_review.png).

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
| PR | [#7141](https://github.com/edwardkim/rhwp/pull/7141) |
| 제목 | 수정(typeset·renderer): 쪽 넘김 1×1 표 조각의 상자·예산을 쪽이 정한다 — 끝 조각은 내용 맞춤 (#7095) |
| 작성자 / reviewer | planet6897, 기존 기여자 / jangster77 요청 및 API 확인 |
| 원 head | `23d507caa8c73e617e480fac7a704920fab2b704` |
| base / 규모 | devel / 10 files, +403/-15 |
| 통합 base | `263b61a64a77a0679e9d8679c5be2e1d180cee1a` |
| 통합 code head | `21164e71a8a84c6204edcad58723f154256587da` |
| branch | `codex/pr7141-7175-7178-20260916` |
| 원 PR 상태 | OPEN / non-draft / MERGEABLE / CLEAN |
| 원 head CI | [Full CI success](https://github.com/edwardkim/rhwp/actions/runs/34988489883) — run head SHA 일치 확인 |

순서는 #7141 → #7175 → #7178이며 기능·test·문서 12개 commit을 `-x`로 체리픽했다.
#7141의 devel merge commit은 제외했으며 원 head와 최신 base의 merge tree가 해당 체리픽 결과와
바이트 동일함을 확인했다. 세 PR 모두 텍스트 충돌은 없었다. [실행 기록](pr_7141_review_impl.md)에
source/local SHA를 남겼다. 각 원 PR 최신 head의 CI 성공은 통합 head의 CI 성공을 대신하지 않는다.

base route: `collaborator_external_pr.md` (기본 작업공간의 devel 기반 누적 체리픽).
loaded: `pr_review_workflow.md`, `pr_review/README.md`, `collaborator_external_pr.md`,
`intake_and_review.md`, `multi_pr_update_branch.md`, `local_validation.md`, `visual_fixture_evidence.md`.
시각 판정에는 [Visual Sweep 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment)을 적용했다.

## 발견 사항과 해제 조건

### P1 — 부분 구현을 전체 이슈 종료로 처리하지 않는다

PR은 `Closes #7095`를 포함하지만 본문에서 축 3인 칸 `valign`을 미해결로 명시한다.
[원 이슈](https://github.com/edwardkim/rhwp/issues/7095)는 상단·하단·valign과 같은 조각 높이 소비를 요구한다.
직접 캡처한 7062 p2의 내부 제목 상단은 약 49.1px로, PDF 약 57.06px보다 위다.
외곽은 y=47.2, bottom=1043.9로 개선됐으나 내부 배치까지 일치한 것은 아니다.
p3도 본문 흐름이 다르다. p10의 외곽 높이는 238.9px이며 PDF의 약 873px와 크게 다르다.
`issue_7095_last_fragment_and_downstream_contracts_hold`는 마지막 하단이 1000px 미만인지만 검사하므로
이 차이를 검출하지 않는다. PDF p10의 ‘Ⅲ. 향후 계획’이 현재 출력에서는 이전 쪽에 있다.

p10의 큰 내용 이동은 기준 devel 바이너리에도 있었다. **신규 회귀로 분류하지 않는다.**
다만 쪽수 10 일치나 마지막 조각 축소만으로 이슈 전체를 해결했다고 판단할 수 없다.
해제: (a) 남은 valign·페이지별 내용 배치를 구현·검증하거나, (b) 상자 기하의 부분 개선으로 범위를
고정하고 원 이슈를 OPEN으로 유지한다. 통합 PR 본문과 최종 squash 메시지 모두 `Refs #7095`를
사용해야 하며, 체리픽 원문에 남은 `Closes #7095`를 최종 메시지에 그대로 복사하지 않는다.

### P2 — 분기별 실제 예산과 paint 상자가 아직 같은 결과로 입증되지 않았다

[typeset](../../../src/renderer/typeset.rs)의 27048행 부근은
`is_continuation || current_height < 0.5`로 여백·100HU 차감을 선택한다.
27274행 부근의 `fragment_placement.map_or`는 앞에서 계산한 `page_avail`을 새로 계산하며
100HU 차감 항을 그대로 소비하지 않는다. 이는 `host_placement`가 있는 비어 있지 않은 host 경로다.
[paint](../../../src/renderer/layout/table_partial.rs)의 3768·4092행 부근은
최상위 여부와 실제 y 좌표로 쪽 상단을 판정하고, 쪽 중간 조각에는 `min(pinned_height)`를 적용한다.
공통 형상 helper를 호출했다는 사실만으로 이 서로 다른 분기가 같은 컷·점유 높이를 쓴다고 보증할 수 없다.

실물 7062의 빈 host 계속 조각은 진단 로그에서 가용 996.5px, paint 상자 높이 996.7px로 확인했다
(표시 좌표 반올림 포함). 비어 있지 않은 host, 쪽 중간 첫 조각, 마지막 유닛 경계의 새 예산을
독립 입력으로 확인한 증거는 이번 검토에 없다. **새 회귀 검출이 아닌 코드상 우려·필수 증거 부족**이다.
해제: 적용 분기별로 실제 host 원점·시작/끝 컷·소유 유닛·요구/예약/paint 높이를 연결하고,
여백을 제외한 높이만 fit하는 경계와 terminal 뒤 본문을 집중 테스트한다. 필요하면 최종 조각
기하를 공유하도록 현재 범위만 보정한다. 전체 엔진 재작성이나 baseline 완화를 요구하지 않는다.

## 기준값 변경 판정

- `issue_4771`은 HWP 그림 y 기대값에 outer-top 283HU를 더하며 HWPX는 유지한다.
  원 PR의 한컴 표 괘선 좌표 근거는 확인했고 관련 6개 테스트는 통과했다.
  이번 직접 sweep은 그 그림 문서의 해당 페이지를 포함하지 않아 독립 시각 확정은 미검증이다.
- `issue_7086`의 제목·빈 줄·그림 y는 +1.9px로 변했다. 7062 sweep과 실제 트리에서 확인했다.
- `issue_6924`는 탐색 범위를 114쪽 한 장에서 114·115쪽으로 넓힌다. 소유 글줄 존재 테스트는 통과하나
  대응 한컴 PDF 없이 페이지 이동의 정답성까지 검증한 것으로 간주하지 않는다.

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
| 구현 근거와 일반성 | 미검증 | 한컴 원본·PDF 근거 존재. 다만 위 P2의 분기/소유 추정 경계 증거 부족 |
| 측정·배치 일관성 | 미검증 | 원본 집중 경로는 확인했지만 위 P2의 혼재 경로까지 공통 결과로 입증되지 않음 |
| 분할·이어받기 계약 | 미검증 | 7141은 cut 예산·paint·terminal 경계 보완 필요 |
| 줄 소속과 점유 높이 | 미검증 | 실물·이웃 focused 통과. 통합 51개 성공을 미실행 경계의 증거로 확대하지 않음 |
| 사례와 증거의 독립성 | 충족 | 원본/한컴 PDF와 통합 head의 Native/fresh WASM 직접 대조. 합성·탐색 제외 |
| 기준값 변경 | 미검증 | 위 기준값 변경 절 참조 |
| 주장과 검증 범위 | 미충족 | 원본 시각 개선·잔여 fidelity·코드 우려·미검증 경계를 별도로 기록 |

## 검증 입력 커밋 확인

**충족** — 아래 실행 파일은 통합 code commit의 blob과 byte 단위 동일하다.
기존 입력을 재사용했고 같은 내용의 HWP/HWPX/PDF 사본을 추가하지 않았다.
테스트가 메모리/임시 파일로 생성한 계약 입력은 해당 커밋의 test source로 재현한다.

| 경로 | SHA-256 |
| --- | --- |
| [samples/issue7062/tac_object_host_line_height.hwp](../../../samples/issue7062/tac_object_host_line_height.hwp) | `2cf764c89943a23eff17fb8ac5ccaa1958711216b15d5eb29a9a469b97d23abb` |
| [pdf/tac_object_host_line_height-2020.pdf](../../../pdf/tac_object_host_line_height-2020.pdf) | `f90ea6915a842ac2266f4dd737b2829bbb3b72b927b1658577f6ca8c8b9b6051` |
| [samples/issue6023/30269_reform_recommendation.hwp](../../../samples/issue6023/30269_reform_recommendation.hwp) | `9e7cc9a3ea8c9d67a9df33fe49921ec87e6f5c38d5cc53b9885a37ef5b285ef7` |
| [pdf/30269-anticorruption-recommendation-toc-2020.pdf](../../../pdf/30269-anticorruption-recommendation-toc-2020.pdf) | `d01f9ff59664a9c5b4e1a67e38f0d1c295aac8e22e3fbc07017f938f197822c3` |
| [samples/issue2004_cell_image_stack.hwp](../../../samples/issue2004_cell_image_stack.hwp) | `c633fd086323075ab25777d8ee10357869d037bd17c9e74d444d5b8297c40d50` |
| [samples/issue2004_cell_image_stack.hwpx](../../../samples/issue2004_cell_image_stack.hwpx) | `e91cf067de71c6c567e27974415bc67462f3e777997d41853db949fea4265fa7` |
| [samples/누름틀-2024.hwpx](../../../samples/누름틀-2024.hwpx) | `a762e165576050cf491cca6b00c941db97cd6730e3a8a34944a7cc15e72e1012` |
| [samples/issue5714/1490000-200800034_vietnam_labor_report.hwp](../../../samples/issue5714/1490000-200800034_vietnam_labor_report.hwp) | `da3550d9f370b52823bd63eae0431d1b86fae46ae211b9cb2cd987165a8c0904` |

## 직접 Visual Sweep

명령: `venv/bin/python scripts/visual_sweep.py --file-target <key> <위 원본> <위 PDF> --rhwp-bin target/pr7141-7175-7178-20260916/release-test/rhwp --pages <아래 쪽> --dpi 96 --out <아래 root>`; WASM은 `--wasm-pkg /private/tmp/pr7141-7175-7178-wasm`을 추가했다.

Native root `/private/tmp/pr7141-7175-7178-native`, WASM root `/private/tmp/pr7141-7175-7178-wasm-sweep`. 각 `<root>/<key>` 아래 `compare/compare_NNN.png`, `overlay/overlay_NNN.png`, `review/review_NNN.png`가 있다.

| key | 쪽 | pixel_match % | visual_accuracy_proxy_percent | flags |
| --- | ---: | ---: | ---: | --- |
| pr7141-7062 | 1 | 81.30603 | 24.68643 | 0 |
| pr7141-7062 | 2 | 76.69072 | 27.87140 | 0 |
| pr7141-7062 | 3 | 80.24184 | 10.23245 | 0 |
| pr7141-7062 | 10 | 87.47743 | 44.38240 | 0 |
| pr7141-30269 | 9 | 91.23053 | 9.81292 | 0 |
| pr7141-30269 | 10 | 86.34225 | 11.53815 | 0 |
| pr7141-30269 | 11 | 94.96480 | 13.52991 | 0 |

Native/WASM의 위 지표는 동일하다. 11쪽 전체의 트리는 내부 usize sentinel의 32/64bit 차이를 제외하면 동일하다. 7062 4쪽분에서만 sentinel 차이가 있으며 나머지 7쪽은 raw JSON도 동일하다. 전체 fidelity 통과가 아니고 위 직접 판정 범위만 인정한다.

`fidelity_compare.py --text-only --export-all-svg --layout-ledger`도 각 원본의 전 페이지(10/22/13)를 실행했다. 원장은 `/private/tmp/pr7141-7062-ledger`, `/private/tmp/pr7141-30269-ledger`, `/private/tmp/pr7178-44529-ledger`에 있다. 7062 p3→4·p8→9에 표 내용 owner 이동 후보 2건, 나머지 두 문서는 page-boundary 후보 0건이다. 후보 0을 시각 일치로 해석하지 않는다.

기준 바이너리는 직전 PR #7179에서 검증한 `60a3ad32e` code 산출을 재사용했다. 이 SHA와 현재 base 사이 `src`, `crates`, Cargo.toml/lock diff가 없음을 확인했다. 위치는 `/Users/tsjang/.Trash/rhwp-pr7179-target-20260916-014502/release-test/rhwp`이며 복구·재이름변경하지 않았다. 기준 sweep은 `/private/tmp/pr7141-7175-7178-base`에 보존했다.

PDF provenance: 모두 PDF 1.6, 595×841pt이다. 7062(10쪽)·30269(22쪽)는 Creator `Hwp 2022 0.0.0.0`, Producer `Hancom PDF 1.3.0.550`; 44529(13쪽)는 Creator/Producer `Hancom PDF 1.3.0.550`. `rhwp info`의 저장 제품은 7062·44529 `hancom-office-2010`, 30269는 product null/version `7.5.12.614`로 2020 bucket에 해당한다. 기존 파일을 재사용했으며 이번에 MCP 재변환하지 않았다.

## Merge 후 contributor PR comment 계획

현재는 머지 보류다. 해제 후 최신 보정 head의 수치·판정을 갱신하고, 아래 asset이 merge SHA로 devel에 존재할 때만 comment를 게시한다. 지금 수치를 보정 후 통과 증거로 재사용하지 않는다.

- [Visual Sweep 정본](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)을 직접 링크한다.
- 위 실제 페이지·flags·pixel_match·내용 픽셀 보조값과 사람 판정/잔여 차이를 함께 적는다.
- 대표 [WASM pr7141-7062 p2](../assets/pr_7141_7062_wasm_p002.png); raw URL 형식 `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/pr_7141_7062_wasm_p002.png`.
- 대표 [WASM pr7141-7062 p10](../assets/pr_7141_7062_wasm_p010.png); raw URL 형식 `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/pr_7141_7062_wasm_p010.png`.
- 대표 [WASM pr7141-30269 p10](../assets/pr_7141_30269_wasm_p010.png); raw URL 형식 `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/pr_7141_30269_wasm_p010.png`.

대표 이미지의 본문·라벨·한글·보조 수치를 직접 확인했다. 보조값은 사람 판정 정확도가 아니다.

원격 comment는 merge 승인과 실제 병합 후 UTF-8 본문 파일을 `--body-file`로 게시하고 API로 재조회한다. 이번 검토에서는 reviewer 지정 외 원격 변경·push·comment·close·merge를 수행하지 않았다.
