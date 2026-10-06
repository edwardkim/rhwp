---
kind: snapshot
status: active
canonical: mydocs/working/task_m100_7095_6946_maintainer_stage1.md
last_verified: 2026-09-16
---

# PR #7141·#7178 메인터너 보정 — 1회차

Issue: #7095, #6946

## 분석 (수정 전)

- 기준: 통합 code `21164e71a`, 검토 기록 `a85189909`, branch `codex/pr7141-7175-7178-20260916`.
- #7178: lane의 x 교차는 앞 표가 그 lane을 예약했다는 증거가 아니다. 문단 내 control 소유를 예약 시 기록하고 형제 판정에서 그 소유를 사용한다. 같은 x에 작은 lane 표, block 표, 후속 표가 섞이는 경우를 제품 경로의 합성 계약으로 검증한다. 합성 입력을 한컴 저장본/출력 증거로 주장하지 않는다. 원본 44529와 #6795 조각·나란한 표의 기존 계약을 함께 확인한다.
- #7141: 빈/비어 있지 않은 host 경로의 최종 원점과 여백·inset 예산을 대조한다. paint가 사용하는 쪽 상자와 요구 높이·컷 경계를 일치시키고 terminal 뒤 내용 계약을 검증한다. 독립 기준은 이미 Git에 있는 7062·30269 HWP와 한컴 PDF다. 새 사본을 만들지 않는다.
- 범위: #7141은 상자 기하와 그 예산의 부분 개선이다. 미완성 valign·기존 페이지별 내용 이동까지 #7095 해결로 선언하지 않는다. 통합 PR/최종 squash에는 `Refs #7095`를 사용하고 이슈를 OPEN으로 유지한다.
- 검증: 관련 focused tests, 변경 전 음성 대조, Native/fresh WASM Visual Sweep, Rust lint 3종과 manifest/unit-tier. 보정 전 결과를 보정 후 통과로 재사용하지 않는다.

## 결과보고

머지 보류의 코드·증거 사유를 보정했다. #7141은 #7095 부분 개선 범위, #7178은 예약 소유 수정 범위에서 승인한다. 아래 최종 실행 결과와 남은 범위를 함께 보고한 뒤 커밋한다.

### 원인과 경계 재현

- #7178 예약 데이터에 문단 내 `control_index`를 기록했다. x 겹침을 소유로 간주하던 추정을 제거했다.
  기존 44529 원본을 유지한 채 앞·뒤에 일관된 치수의 빈 표를 추가한 합성 IR 계약에서,
  보정 전 두 표가 `601.05 × 384.92px` 겹쳤다. 수정 전 실패·수정 후 통과를 확인한다.
- #7141 텍스트 host의 계속 조각은 예산에 넣은 outer-top을 paint에서 다시 열었다.
  수정 전 경계 테스트에서 본문 위 `20px`, outer-top `141HU = 1.88px`인데 조각 상단은
  `23.76px`였다. 확정 원점에 여백을 한 번만 넣고 paint는 그 원점을 소비하도록 보정했다.
  최종 가용 높이를 paint와 같은 `body-bottom − outer-bottom − 100HU`로 제한한다.
- 입력 계약은 ASCII 문자 offset·표 control의 text-tail 위치·편집 무효화 상태를 명시한다.
  기존 한컴 저장 줄을 임의로 위조하지 않으며 한컴 정답지로 쓰지 않는다. 쪽 높이
  21000~24000HU를 100HU 간격으로 바꾼 빈/텍스트 host 62개 조합에서 내용 40개 유닛의
  1회 표시, 자기 셀 내부 위치, 계속 조각 outer-top 1회 적용, 후속 본문 순서를 검사한다.
- 보정 전 코드(`a85189909`의 source)로 새 테스트 2개를 실행했을 때 **2 failed / exit 100**.
  대조 후 보정 source 4파일을 원래 byte로 복원했다. 로그 `/private/tmp/pr-maint-negative.log`.
- 초기 표본은 내용만 검사해 #7141의 두 번 열린 여백을 검출하지 못했다. 계속 조각의 실제
  상단 계약을 추가한 뒤에야 위 결함을 검출했다. 통과만 하는 초기 표본을 회귀 증거로 쓰지 않는다.
- 초기 build는 Xcode 라이선스 미동의로 link exit 69였다. 시스템 설정/약관 수락 없이
  `DEVELOPER_DIR=/Library/Developer/CommandLineTools`를 이 작업의 빌드 환경에만 지정했다.
  CLT SDK 경로를 `xcrun --sdk macosx --show-sdk-path`로 확인했다.
- rustfmt 뒤 원본 파일 weight가 바뀌면 suite 자동 배정도 달라진다. 원본 수정·fmt 뒤
  `--prepare`를 다시 실행한 manifest로 실제 suite를 해석한다. 초기 0-test 실행은 검증으로 세지 않는다.

### 기각한 확대 적용과 보정 범위

최종 상자 하단 cap을 모든 빈 host의 중간 조각 advance 예산에도 적용한 초안은 기각했다.
`80168_regulatory_analysis.hwp`가 157→158, `rowbreak-problem-pages.hwp`가 18→19쪽으로 늘었다.
기존 주석에 기록된 반례를 현재 바이너리로 재확인한 것이며 baseline을 갱신하지 않았다.
전체 검증은 이 결함을 발견해 빌드 단계에서 중단(exit 130)했고 성공 결과에 포함하지 않는다.

- 빈 host의 기존 저장 컷은 마지막 줄 뒤 비가시 간격을 포함하는 advance 계약을 유지한다.
  쪽 중간 조각의 paint 하단 제한을 그 advance에 다시 빼지 않는다. 본문 상단 조각의 기존
  여백·100HU 예약은 그대로이며 해당 실물과 경계 테스트에서 직접 대조한다.
- 비어 있지 않은 host가 **확정 원점으로 기존 예산을 대체하는 경로**에는 빠졌던 물리 하단
  cap을 넣는다. 첫 원점에서 outer-top을 한 번 계상하고 계속 원점은 이미 계상된 값을 쓴다.
  paint가 확정 원점 위에 outer-top을 재가산하지 않는다.
- 반례 2건을 보호하는 focused test를 추가했다. 독립 PDF는
  `pdf/80168_regulatory_analysis-2022.pdf`(Hwp 2022 12.0.0.4547, 157쪽),
  `pdf/rowbreak-problem-pages-hwp-2024.pdf`(Hwp 2024, 18쪽)다. 후자를 2020 PDF로 표기하지 않는다.
- 보정 후 동일 후보에서 관련 tests·전체 회귀·lint·WASM을 다시 실행한다. 확대 적용 초안의
  78 passed와 Native sweep을 최종 후보의 검증 완료로 옮기지 않는다.

Visual Sweep의 80168 기준은 현재 원본의 `lastSavedWith=hancom-office-2024`에 맞는 기존
`pdf/80168_regulatory_analysis-hwp-2024.pdf`(157쪽)를 사용한다. rowbreak 원본도 2024 저장본이다.
KTX는 2018 저장본과 기존 Hwp 2022 12.0.0.4426 PDF(27쪽), issue2004는 2022 저장본과
기존 hwp-2020 기준 PDF를 재사용한다. 파일명만으로 변환 engine을 추정하지 않았다.

### 실제 호출 경로·컷·공간 대조

| 경로 | 컷/소유와 요구 높이 | 예약·배치·종료 |
| --- | --- | --- |
| 빈 host 저장 RowBreak | continuation step → `scan_block_table_split_rows` → 절대 unit cut. 기존 advance에는 끝 줄 뒤 간격이 포함된다. | 쪽 상단에서는 기존 outer-top/bottom·100HU 예산, 중간에서는 저장 advance 예산. 비끝은 같은 column을 닫고 다음 쪽으로 전진한다. `usedHeight` 진단값은 내용 advance이며 늘어난 paint 상자 높이와 동일한 수치라고 주장하지 않는다. |
| 텍스트 host의 확정 원점 | `for_first_fragment` → fragment 원점 → 최종 replacement budget. 첫 outer-top을 이 원점에 한 번 계상하고, 같은 원점과 물리 하단 cap으로 컷을 고른다. | `commit_fragment`가 page placement를 저장하고 `layout_partial_table`이 resolved 원점을 소비한다. 계속 조각은 새 frame에서 열린 여백을 paint가 다시 더하지 않는다. |
| terminal·후속 본문 | end cut 없음/최종 unit 소진 확인 뒤 continuation을 종료. terminal의 기존 내용 높이와 후행 간격 계약을 보존한다. | 62개 편집 IR 경계에서 40개 unit 각각 1회, 셀 내부, outer-top 1회, `AFTER`가 마지막 표 뒤인 것을 확인한다. 일반 terminal cut 경계는 기존 #6981 묶음도 재실행한다. |
| 중첩/다행/일반 HWPX | 이번 추가 cap은 HWP5 저장 pagination 계보의 단일 셀 원래 row domain에서 확정된 host placement에 적용한다. 호출자는 `hwp5_stored_pagination_layout()`이며 원 HWP5와 marker HWPX를 포함하고 일반 HWPX는 제외한다. | `row_geometry_table`이 다른 중첩 owner인 경우 확장하지 않는다. 원래 nested/rowspan 컷 계약을 보정하지 않는다. 원 HWP5 실물·합성 경계, 기존 HWPX 이웃·roundtrip 전체 회귀의 증거 범위를 구분한다. |

최종 후보 7062 `pi=8, ci=0, row 0..1`의 `dump-pages --json`에서 컷은
`[]→21→27→67→88→131→162→199→253→296→[]`로 연속된다.
p1 요구 advance 397.1 / 가용 508.1, p2 980.4 / 996.5, p3 961.1 / 996.5,
terminal p10 238.7 / 996.5px다. 가시 소유를 나타내는 컷과 쪽을 닫으면서 남기는 물리 공간을
혼동하지 않는다. p2 paint 높이 약 996.7px와 p2 내용 advance 980.4px의 차이는 남는 빈 밴드다.
이 값은 #7095의 아직 미완성인 valign 배치를 해결했다는 근거가 아니다.
로그 `/private/tmp/pr-maint-7062-budget.log`, `/private/tmp/pr-maint-7062-pages.json`.

### Visual Sweep이 발견한 추가 회귀: 파생 그림 조각의 외곽

- issue2004 p5의 긴 회색 선은 편집 안내선이 아니라 SVG의 실제 검정 인쇄 테두리였다.
  원래 devel의 표 y/height는 83.1/842.9px, #7141 적용 후 86.9/936.2px였다.
  outer-top 보정은 PDF와 맞지만 본문 끝까지 늘린 하단은 PDF보다 약 93px 길었다.
- 기존 `pdf/issue2004_cell_image_stack-hwp-2020.pdf` p5를 `pdftocairo -svg`로 읽으면
  bottom path는 y=143.121094pt, transform은 `(1,0,0,-1,0,841)`이다.
  원본 84188HU 쪽 높이로 정규화한 독립 하단은 931.48px다.
- 원인: `reclassify_cell_floating_stacks`가 생성한 그림별 inline 문단에는
  `TAG_IMPLEMENTATION_PROPERTY` 출처가 있다. 이 내용 단위의 높이를 저장된 쪽 프레임으로
  간주하면 안 된다. 1×1/RowBreak 형상만으로 쪽 전체 소유를 추론한 가정을 제거했다.
- 실제 컷의 `CellUnit` → 원 문단/줄 범위 → 파생 줄 출처를 읽는
  `cell_cut_has_projected_lines`를 사용한다. 파생 내용 조각은 계산된 내용 높이를 보존하고
  물리 하단 제한만 적용한다. 원점/바깥여백과 기존 컷의 예약은 바꾸지 않는다.
- 새 실물 테스트는 PDF 하단 931.48px ±2px를 검사한다. 허용 차이는 기존 그림 projection의
  약 1.7px 오차를 명시한 것이며, 기존 golden/래칫은 바꾸지 않았다. 수정 전 하단 1023px는
  이 계약을 위반한다. 그림 identity/위치와 8쪽 종료는 기존 #4771 계약도 함께 검사한다.
- 직전 후보의 전체 nextest는 9915 passed / 51 skipped (393.807s, 5 slow, 1 leaky)였다.
  위 시각 회귀를 고친 최종 source 검증으로 재사용하지 않고 별도 최종 실행을 시작했다.
  로그 `/private/tmp/pr-maint-full.log`와 `/private/tmp/pr-final-full.log`를 구분한다.

### 최종 Native 시각 대조

7개 문서의 선택 25쪽을 직접 대조했다. 7062 p1/2/3/10, 30269 p9/10/11,
44529 p6/7/8/9, issue2004 p4/5/6/7/8, KTX p24/25/26, 80168 p28/29/30,
rowbreak p13/14/15다. 전체 SVG 255쪽을 직전 후보와 비교하면 issue2004 p5/6/7만 달라졌고,
나머지는 동일하다. 해시만으로 시각 판정을 대체하지 않고 선택 페이지 비교 이미지를 확인했다.

- issue2004: [보정 전](../pr/assets/pr7141_maintainer_2004_p005_before_compare.png)과
  [보정 후](../pr/assets/pr7141_maintainer_2004_p005_after_compare.png). p5의 실제 인쇄 테두리
  하단을 약 93px 줄였고, PDF 정규화 하단과 약 1.7px 차이를 유지한다. p4/8은 변경 없으며
  p6/7도 불필요한 빈 밴드가 사라졌다. 이미지 5개가 각 쪽에 1개씩 표시된다.
- 7062: p2 표 외곽은 개선돼 있으나 제목/내부 valign과 p3/p10 내용 이동은 여전히 남는다.
  #7095 부분 개선으로만 승인하며 전체 이슈 종료 근거로 사용하지 않는다.
- 44529: p7/8 표 분리, 이웃 p6/9 순서 유지. 기존 도형선·줄바꿈 차이는 남아 있다.
- 30269/KTX: 이어받는 표 외곽·뒤 본문과 선택 이웃 페이지 연결 유지.
- 80168/rowbreak: base 바이너리로 같은 PDF와 추가 대조했다. 주요 줄바꿈·내용 차이는
  기존 현상이며, 이번 컷/외곽 보정이 페이지 수를 158/19로 늘리는 초안 회귀는 제거됐다.

Native root: `/private/tmp/pr-final-native`; 추가 base 대조 root: `/private/tmp/pr-final-base`.
각 `maint-<key>`의 compare/overlay/review와 metrics를 보존한다. 최종 fresh WASM 대조는
아래 최종 검증 결과에서 별도로 확인한다.

Base 대조 바이너리는 기존 PR #7179의 `60a3ad32e` 빌드이며 현재 base `263b61a64`와
`src`, `crates`, `Cargo.toml`, `Cargo.lock` diff가 없음을 재확인했다. SHA-256:
`f3358c56345a45a08e1e5d713cb250c4077a1ab43d91659572d37d75a16208b8`.
최종 Native 캡처 바이너리 SHA-256:
`bc95a54c2ed4e5864e38ed7f08876260d020551e72c310171159a304a7295760`.

### 최종 실행 결과

- 최종 source patch SHA-256: `837efe31cbdbe954346ebc85c886ab34077e9a793f16435ab0b2f8cfacb8c488`.
- focused: **80 passed / 1318 skipped**, test 6.010s, exit 0. 62개 host/높이 조합은
  한 test 내부의 계약 행렬이며 62개 독립 nextest 항목으로 중복 집계하지 않는다.
- 전체 nextest: **9916 passed / 51 skipped**, test 370.437s, exit 0.
  compile 포함 896.55s. 5 slow / 2 leaky 표시는 숨기지 않고 별도 재실행 결과와 구분한다.
- fmt 및 native Clippy `-D warnings`: exit 0.

- WASM lib Clippy `--target wasm32-unknown-unknown -- -D warnings`: exit 0.
- `docker info`는 daemon 연결 실패(exit 1). 이번 브라우저 증적은 로컬
  `scripts/wasm-pack-locked.sh --target web --out-dir /private/tmp/pr-maint-wasm --no-opt`
  경로다. Docker 최적화 배포 빌드 검증으로 표기하지 않는다.

- Workspace build 및 all-target Clippy: exit 0. manifest/unit-tier는 고정 base
  `263b61a64a77a0679e9d8679c5be2e1d180cee1a` 대비 exit 0.
- Native Skia lib: **4112 passed / 13 ignored**, exit 0 (root 3930 + 내부 crate 15/165/2).

- Native Skia 그림 누락 gate: **2 passed / 201 skipped**, exit 0.
- Native Skia 직접 PDF export gate: **4 passed / 191 skipped**, exit 0.

- fresh WASM build exit 0. Native/fresh WASM의 7개 문서·25쪽을 직접 비교했다.
  새 입력 사본 없이 기존 tracked HWP/PDF 14개를 재사용하며 Git blob과 byte 동일함을 확인했다.
- 최종 검토 판정: #7141 부분 개선 범위 승인, #7178 승인, #7175 승인 유지.
  원격 push/통합 PR/merge는 이번 단계에 포함하지 않는다. #7095는 OPEN이며 `Refs`만 사용한다.

[기계 검증 요약과 입력 SHA-256](../pr/assets/pr7141_7178_maintainer_validation.json).

LEAK 표시 2건은 `body_overflow_does_not_grow_partition_12`와
`issue_2007_nested_cell_content_paginates`였다. 같은 최종 소스로 해당 2개만 재실행하여
**2 passed / 398 skipped / LEAK 표시 없음 / exit 0**을 확인했다(1.362s).
원 전체 실행의 2 leaky 기록은 유지한다. `/private/tmp/pr-final-leak-recheck.log`.

코드 위치(이 회차 검증 source): `float_placement.rs:1375` 물리 하단,
`typeset.rs:21238` 예약 소유, `typeset.rs:27274/27306` 확정 원점과 최종 예산,
`table_partial.rs:3772/4110` 여백 중복 방지와 파생 조각 분기,
`table_layout.rs:14531` 같은 컷의 줄 출처다. 뒤의 주석 전용 추가 커밋으로 줄 번호는 이동할 수 있다.

### 검토 중 추가된 원 PR 증적

원격 #7141은 `23d507caa` 이후 `d2403022946af730d2110b17f7d03d750fb75ca1`이 추가됐다.
추가 내용은 `split-continuation-proof.md` 141줄과 Rust 주석 14개 행 추가/삭제이며
동작 변경은 없다. 원 diff의 모든 Rust 변경 행이 전체 줄 주석임을 확인했고 현재 작업에
`git apply --check`도 통과했다. 이 보정 회차를 결과보고·커밋한 뒤 원 저자/출처를 보존해
추가 체리픽하고 최종 검토 기록에 반영한다.

저자 증적은 이전 코드 `23d507caa`의 빈 host 체인을 보강하지만 이 회차에서 발견한 텍스트
host 이중 여백·파생 그림 프레임·lane 혼재 반례를 대신 검증하지 않는다. 저자가 기록한
7062 p3/p6의 기존 이탈 크기 +1.88px와 미완성 valign도 #7095의 잔여 범위로 남긴다.
이를 전체 시각 일치 또는 #7095 종료 승인으로 바꾸지 않는다.
