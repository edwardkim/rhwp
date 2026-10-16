---
kind: evidence
status: active
canonical: mydocs/working/task_m100_7445_rebase_repair_stage5.md
last_verified: 2026-10-16
---

# 분할한 부모 셀과 자식 표의 저장 총높이 출처

Issue: #7445. 이 기록은 #4138의 중첩 표 높이와 그 뒤 빈 문단의 쪽 소유 보정이다.
브랜치 전체 PR 준비·전체 회귀·최신 devel 대비 무회귀 완료를 뜻하지 않는다.

## 독립 근거와 구현

원본 HWP/HWPX의 같은 자식 표81은 독립 Print에서 약101.65CSSpx 외곽을 유지한다.
부모 셀을 나눠 저장한 HWP에서는 같은 자식의 끝 행이 자연 높이를 유지해 약105.16CSSpx가 된다.
같은 원본 HWPX의 한컴2020/2024 출력 및 같은 분할 HWP의2020/2024 출력도 각각 같은 괘선 좌표다.
따라서 포맷·한컴 세대 차이로 나누지 않는다. 수동으로 저장 정보를 조작한 입력도 아니다.

`RenderNormalizationOverlay`는 기존의 폭 변경·원문 셀/복제 빈 peer 출처 판정으로
부모 행 재조판 여부를 확인하고 직접 자식의 상속 총높이 프레임에만 무효화 출처를 보관한다.
자식 자체의 셀 선언·글줄 저장 정보·텍스트는 바꾸지 않는다.
`resolve_row_heights_with_common_fit`의 캐시/비캐시 총높이 축소와 뒤의 common-height 맞춤이
같은 출처를 확인한다. 개별 셀 선언 신뢰(`trust_declared_row_heights`)는 유지한다.

실제 소비 사슬은 원문 부모 폭 변경 출처 → overlay의 직접 자식 표시 → 공통 행 높이 질의 →
중첩 body/flow 높이와 CellUnit 점유·컷 예산 → 동일 행 높이의 실제 표/셀 paint다.
`nested_table_body_height`와 중첩 조각 행·반복 머리글 경로도 같은 resolver를 소비한다.
최상위 `typeset/table.rs::fit_measured_for_host`는 이 자식의 측정 경로가 아니며, 바깥 표는
끝 셀에 중첩 표가 있어 단순 declared-tail helper 대상이 아니다. 뒤의 common-height 경로도
같이 차단해 무효한 총높이로 재확대하지 않는다. 좌표 clamp·문서 ID·글꼴/허용치 변경은 없다.

## 입력·기준 Print

| 조건 | 입력 SHA-256 | 독립 PDF SHA-256 |
| --- | --- | --- |
| `mydocs/pr/assets/issue7445/completed-20261009/regression-policy-20261015/issue4138-current-production/saved-split-1x2.hwp` | `4a96f1fc838eb7239d8c85b7698b9e04f83fc6b2f271c948ccae038a1c6c89b9` | `5a82290de1571c692c7246fe6b57ae136602ae52c2068f2568fffc6c91bfd571` |
| `/Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-sync-20261015/tab-period-split.hwp` | `d9bc6b13d54f5179c2cf57e6a2310a28efce4f8f6d6e74137a5828c9878a24b6` | `687f1404e56a764258c93fb0e59d9fb1dd870b506d36451fbfa833fd207f8294` |
| `samples/issue1949_giant_cell_nested_tables_perf.hwp` | `ef10261cd29325116028e4f4f3e6be1a72c675eb771bddfd8484e7fe5aa94b4e` | `9fb08dea604813eed90a0449f29682d400be54dacd413352cc4f680f37cd4a44` |
| `samples/issue1949_giant_cell_nested_tables_perf.hwpx` | `fc6e5f156de470dfbb14aab392389491720ee7fb1bf6f03fe9a018e93b420c65` | `ba065afdd3ace7fd6b87fe82848e6cae3d01915a1606e4d8a70e4c2153d57d73` |

기준 PDF:
- [issue4138-ascii-split-20261016-2024.pdf](../../../../../../../pdf/issue4138-ascii-split-20261016-2024.pdf)
- [issue4138-tab-period-split-20261015-2024.pdf](../../../../../../../pdf/issue4138-tab-period-split-20261015-2024.pdf)
- [issue1949_giant_cell_nested_tables_perf-hwp-2024.pdf](../../../../../../../pdf/issue1949_giant_cell_nested_tables_perf-hwp-2024.pdf)
- [issue1949_giant_cell_nested_tables_perf-hwpx-2020.pdf](../../../../../../../pdf/issue1949_giant_cell_nested_tables_perf-hwpx-2020.pdf)

## 현재 후보의 전쪽 결과

기준 `cdef0073bb24512daf0622dec5051ac62cb8d57e`, 보정 직전 부모 `cab7f21ab90e760ea5b3d1f1088e82370d2f10bd`와 본 커밋의 Rust 변경이다.
Native SHA-256 `293817ae2f03c9838d0d59697e9382740660c37699d6c94440d7707552c17ac8`.
fresh WASM SHA-256 `89b8a81c4ea899eb40908c0af2dcc0364a859f239f9e64bb9b67463aea84568e`.
Mac locked wrapper `--target web --out-dir pkg --no-opt` 로컬 대체 빌드이며 Docker 결과가 아니다.
root pkg·Studio public·보존 pkg의 JS/WASM hash는 일치한다. 실제 Studio 초기 응답과 새 후보
전체 소비 경로 검증은 아직 미검증이다.

| 조건 | Native/WASM/PDF 쪽수 | Native 최저 | fresh WASM 최저 | 90% 미만 |
| --- | --- | --- | --- | --- |
| giant4138-current-ascii | 197/197/197 | 90.27729% | 90.21662% | 양쪽0쪽 |
| giant4138-tab-period | 197/197/197 | 90.27729% | 90.21662% | 양쪽0쪽 |
| giant4138-original | 115/115/115 | 90.62416% | 90.50161% | 양쪽0쪽 |
| giant4138-hwpx115 | 115/115/115 | 97.13453% | 97.13430% | 양쪽0쪽 |

각 backend4조건624쪽, 합1248쪽을 새로 산출했다. 정본 Visual Sweep, print profile,
192DPI(HWPX96DPI), 해당 compat2024/2022,2px 이웃 관용 실루엣을 사용했다.
기존 공통 글꼴 공급·명시적 공통 빈 FontEnvironment를 유지했다. 기본 프로필과의 동등성
및 최신 devel과의 전체 비교는 별도 잔여이며 이 수치로 대신하지 않는다.

| 주요 쪽 | Native 보정 전 → 후 | fresh WASM 후 |
| --- | --- | --- |
| 분할7 | 89.10530 →97.68947% | 97.68959% |
| 분할10 | 14.06920 →96.61078% | 96.55518% |
| 원본 HWPX4 대조군 | 99.98878% | 99.98877% |

## 회귀와 직접 판독

정식 검사 `split_parent_keeps_child_row_occupancy_and_the_following_blank_owner`는
마지막 행의 원본 점유 하한,7→8쪽 빈 문단86과 원문85의 누락·중복,9→10쪽 빈 문단110과
수식111의 앞뒤 관계를 보호한다. 원본 HWP/HWPX115쪽과 두 분할197쪽을 유지한다.
원본 대조군은 유효 총높이와 셀별 내용·순서를 함께 검사한다.
7~10쪽은98% 미만이므로 절대 px를 추가하지 않는다. 같은 자식81의 편집 전 프레임을
검증하는 HWPX4쪽은 Native99.98878/WASM99.98877%여서 독립 Print의 시작969.658691·
끝1071.308024CSSpx를 ±2CSSpx로 함께 보호한다. 더 높은 다른 쪽은 같은 자식 표를
검증하지 않으며, 기존 분할177쪽의99%대 좌표·내용 관계 검사도 유지한다.

수정 직전 정확한 부모 소스의 실제 실행은1FAIL(0.70초, 끝 행67.613333 vs 원본5331HU)이다.
처음 archive 빌드가 공유 target의 후보를 재사용한 것은 서로 다른 소스인데 binary hash가
같아 사용 전에 발견했다. 해당 producer를 INVALID로 보존하고 archive의 source mtime만
갱신해 실제2분53초 재컴파일 후 서로 다른 binary·실제 FAIL을 확인했다.
현재 후보의 source bytes도 보존한 채 mtime을 갱신해 정식 Cargo 검증을 진행했다.
최종 정식 nextest는 `regression_suite_016`에서 실제4PASS/0FAIL/220skip,2.802초다.
그 중 새 검사는2.800초 PASS이며 기존 세 검사의 쪽수·내용·그림 경계도 유지했다.

Native/fresh WASM의7·10쪽과 원본 대조군4쪽 review·overlay를 각각 함께 직접 열었다.
중첩 표의 마지막 괘선과 뒤 본문, 다음 쪽의 빈 줄 점유·첫 수식이 복원됐다.
작은 글리프·수식 간격 차이는 남아 있으며 완전 픽셀 일치로 보고하지 않는다.

## 대표 review·overlay

| 조건 | review | overlay |
| --- | --- | --- |
| before/native/7 | [review](giant4138-current-ascii-p007-before-native-review.png) | [overlay](giant4138-current-ascii-p007-before-native-overlay.png) |
| after/native/7 | [review](giant4138-current-ascii-p007-after-native-review.png) | [overlay](giant4138-current-ascii-p007-after-native-overlay.png) |
| after/wasm/7 | [review](giant4138-current-ascii-p007-after-wasm-review.png) | [overlay](giant4138-current-ascii-p007-after-wasm-overlay.png) |
| before/native/10 | [review](giant4138-current-ascii-p010-before-native-review.png) | [overlay](giant4138-current-ascii-p010-before-native-overlay.png) |
| after/native/10 | [review](giant4138-current-ascii-p010-after-native-review.png) | [overlay](giant4138-current-ascii-p010-after-native-overlay.png) |
| after/wasm/10 | [review](giant4138-current-ascii-p010-after-wasm-review.png) | [overlay](giant4138-current-ascii-p010-after-wasm-overlay.png) |
| control/native/4 | [review](giant4138-hwpx115-p004-control-native-review.png) | [overlay](giant4138-hwpx115-p004-control-native-overlay.png) |
| control/wasm/4 | [review](giant4138-hwpx115-p004-control-wasm-review.png) | [overlay](giant4138-hwpx115-p004-control-wasm-overlay.png) |

## 로컬 원본 증적과 잔여

로컬 정본: `/Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/regression-policy-audit-20261015/rebase-cdef0073-20261016` 아래 `c71-whole-native/`, `c71-whole-wasm/`,
`candidate-c71/`, `c71-exact-parent-build/`, `c71-canonical-fixture.log`다.
전쪽 TSV·실행 로그·중간 JSON은 ignored output에만 보존한다.
전체48조건의 새 base/head 비교, 전체 nextest·lint/Skia·실제 소비 경로 및 남은 제목·따옴표
시각 보정은 아직 완료하지 않았다. 이 단계 결과로 PR을 생성하거나 준비 완료로 표시하지 않는다.
