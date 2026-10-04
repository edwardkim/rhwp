---
kind: investigation
status: active
canonical: mydocs/tech/typesetting_architecture.md
last_verified: 2026-10-04
---

# #7207 — RowBreak 선행 병합 뒤 저장 프레임 잔여 보정

기준 devel은 `731de9e1b4bb946d76f35108ed7e186ebe4ebecb`이며, #7567의
RowBreak·최소 공통 메트릭 보정을 포함한다. 원 후보 `bac75f50ee4e57839f4c0ac7a259165acf0e3509`는
별도 브랜치에 보존했다. 새 후보는 기준 devel에서 시작하며, 중복 공백 메트릭 변경을 제외한
6개 source 파일의 저장 표 프레임·물리 공간 처리만 재검증한다. #7544의 본문·source는 수정하지 않는다.

## 독립 근거와 주장 경계

실제 저장 원문 `samples/task2097/18095317_eogu_geumji.hwp`와 그 원문의
`pdf/18095317_eogu_geumji-2020.pdf`를 사용한다. 보존 후보의 과거 21쪽 통과 결과는
이번 head의 증거로 승계하지 않는다. 제출 직전 #7568 병합으로 base가 전진하여 실제 충돌이 생긴 3개 source를 통합하고 전체 검증을 다시 실행했다. 통합 중 정상 문서 86712의 병합 셀 높이 변화가 검출되어 문단 안 rewind와 경쟁 rowspan의 소유를 분리한 뒤 대조군 전쪽 동일성을 다시 확인했다. 교육과정의 기존 413/415쪽 차이는 별도 문제로 유지한다.

저장 셀 최소 높이는 내용 컷뿐 아니라 정렬에 필요한 빈 물리 공간을 소유한다.
원본 전폭 셀의 frame reset·원본 줄 정보·소유를 검증한 경로만 대상이며,
편집·합성/투영 줄·경쟁 rowspan 소유자는 적용 경계에서 제외한다.
문서 ID·임의 크기 조건을 추가하거나 baseline·허용치를 완화하지 않는다.

현재 후보의 실제 소비 연결은 `stored_full_width_row_source_height` →
`stored_full_width_row_declared_height` → `fragment/emit`의 실제 수용 높이와 누적
`stored_row_box_sum` → 다음 시작 행 override → `RowScanQuery::whole_row_height` →
`PartialTable` → `table_partial`의 저장 컷 정렬이다. emit의 높이 변경과 실제 예산 수용·후속 내용의 연결을 코드 및 실행 결과로 대조했다. helper 이름만으로 충족 판정하지 않는다.
SectionDef·ColumnDef는 본문 프레임 설정이며 단독 표 앵커에 경쟁하는 가시 객체가 아니다.
그림·추가 표·본문 텍스트는 단독 앵커 대조군에서 제외한다.

## 실행 순서

1. 최신 devel 및 잔여 후보의 작은 기존 컷·이어받기 검사와 영향 페이지를 먼저 실행한다.
2. 같은 원문의 Native/fresh WASM 전체 TSV와 대표 compare/review/overlay를 새로 만든다.
3. RowBreak 정상 대조군 전체 18쪽 및 다른 영향 문서의 내용·소속·쪽수·배치를 대조한다.
4. 시각 선행 조건 충족 뒤 독립 기대값의 정식 회귀를 추가하고 수정 전 FAIL/후 PASS를 확인한다.
5. 최종 source에서 전체 회귀·Native Skia·필수 세 Clippy·workspace build·고정 base 정책을 실행한다.
6. 실제 실행 결과와 잔여 범위를 기록하고 PR 본문 초안을 준비한다. 원격 push·Open PR 생성·GitHub CI는 사용자 승인을 받았다. PR 채번 뒤 self-review와 오늘할일을 trailing 문서 commit으로 추가한다. 병합은 별도 승인 대상이다.

사용자 제공 글꼴은 비공개 로컬 검증에만 사용한다. 글꼴 파일·식별 자료·글꼴 포함
SVG/HTML/로그는 공개 증적으로 포함하지 않는다. 허용할 raster와 수치만 별도로 검토한다.

## 최종 후보 실행 기록

최종 production/test source는 `4fc0862df1b1cb6155c3745052dfb3c76e432e49`.
아래는 이 source에서 새로 실행한 결과이며 과거 후보 결과와 구분한다.

- 기준 devel의 Native 어구 출력도 21쪽이지만 최저 14.61476%, 90% 미만 16쪽이었다.
  기준 실행은 고정 base binary 해시로 식별하며 쪽수만으로 배치 회복을 판정하지 않는다.
- Native: 어구 21쪽 최저 93.22327%, RowBreak 18쪽 최저 92.47763%. 90% 미만·누락 쪽 없음.
- 어구 2·4·7·10·14·17–21쪽과 RowBreak 7·8·11·12쪽의 새 review PNG를 직접 판독했다.
- 정상 대조군 10개·891쪽의 쪽수와 전쪽 render-tree JSON이 기준 devel 출력과 동일했다.
  교육과정은 기존 413쪽을 유지하며 독립 PDF의 415쪽과 다른 문제는 해결로 보고하지 않는다.
- 정식 회귀 `issue_7207_stored_frame_map_ownership`의 세 검사 모두 기준 binary에서 FAIL,
  최종 binary에서 PASS였다. 시작 프레임·업종 소속, 10쪽 주석 소속, 17–21쪽 6개 그림과
  캡션의 쪽·셀 포함·순서를 검사한다. 내용 존재만으로 위치를 판정하지 않는다.

### 적용 규칙과 소비 경로

아래 위치는 검증 source SHA 기준이다. `table_layout.rs`·`table_partial.rs`는
`src/renderer/layout/`, `float_placement.rs`는 `src/renderer/`,
`fragment/emit.rs`는 `src/renderer/typeset/table/continuation/`,
`scan/row.rs`는 `src/renderer/typeset/table/`의 파일을 가리킨다.

| 변경 주장 | 실제 생산·소비 경로 | 적용/비적용과 근거 |
|---|---|---|
| 내용 컷과 물리 공간을 별도로 보존 | `table_layout.rs:15703` 원본 셀 최소 높이 → `fragment/emit.rs:84` 원본 본문 높이로 소유 판정 → 실제 남은 예산에서 수용 높이 결정 → `:913` 실제 수용 높이를 cursor에 누적 → `scan/row.rs:62` 다음 조각의 내용과 잔여 물리 높이 → PartialTable → `table_partial.rs:1340` 같은 소유 조건으로 정렬 | 원본 Native HWP5의 전폭 단일 소유 행. 편집·투영·경쟁 rowspan·위 정렬은 제외. 각주/zone 예약 높이는 수용 예산에서 계산하고 원본 frame의 존재 판정과 섞지 않는다. |
| 시작 프레임의 빈 밴드와 다음 내용 보존 | `table_layout.rs:15386` 공통 원본 첫 frame → `fragment/emit.rs:220,353` 요구 높이·물리 점유 → `:994–1053` 남은 시작 행 override → scan → `table_partial.rs:5389` 동일 첫 frame 정렬 | 컷 앞에서 완결된 rowspan은 허용하고 컷을 가로지르는 소유자는 제외. 원본 문단 안의 양수→0 rewind는 경쟁 rowspan이 없는 시작 frame 경계에서만 사용한다. 원본 첫 frame 높이 생산은 최신 base의 저장 컷 계약을 유지하고, 일반 tail·sliver는 종전 cross-paragraph 규칙을 유지한다. |
| 프레임 설정이 동반된 zero-origin 표 앵커 | `float_placement.rs:209` 앵커 생산 → 표 앞 공간/분할 예산·배치의 기존 소비 지점 | 원본 zero-origin·폭0 단독 표 줄에서 SectionDef/ColumnDef를 설정으로 취급한다. 양수 host-origin이 소유한 문단 공간, 텍스트·추가 표·그림, 편집·합성 줄은 이 완화를 적용하지 않는다. |

실제 원문의 시작/끝 컷, 가운데/아래 정렬 빈 공간, 마지막 업종 뒤 주석과 마지막 지도 종료를
검사했다. 여러 rowspan의 동시 종료·별도 각주 예산을 조합한 새 합성 PDF 계약은 이번 범위에서
추가 검증하지 않았다. 기존 관련 컷·rowspan 검사를 전체 회귀에 포함하되 이를 독립 PDF 증거로
승격하지 않는다. HWPX 대조군은 무회귀를 확인한 것이며 Native 원본 frame 소유 보정의 적용 증거가 아니다.

### 잔여 차이와 완료 경계

표의 선 굵기·글자 획, 어구 1·4쪽 분할 표의 닫는 아래 경계선과 19쪽 범례선의 수직 차이는 남아 있다. 어구 7쪽 아래 경계선도 96dpi에서 PDF y1081px, 후보 y1066–1067px로 약 15px 차이가 남는다. 직접 판독한 페이지에서
업종·주석·지도·캡션의 누락·중복·잘못된 쪽 소속은 확인되지 않았다. 점수로 잔여 차이를 면제하지 않는다.
새 baseline/golden·허용치 완화·문서 ID 분기는 추가하지 않았다.

#7207의 다른 두 문서와 교육과정 기존 출력 결함은 별도 조사 대상으로 유지한다.
이 후보는 #7207 전체를 종료하지 않으며 #7544의 본문·source를 변경하지 않는다.

## 최종 로컬 검증 판정

| 검사 | source `4fc0862d` 결과 |
|---|---|
| release-test 전체 nextest | 10,291 PASS, 50 skipped; exit 0 |
| Native Skia 3종 | lib 및 placeholder·p37 모두 PASS |
| fmt·Native/WASM32/workspace all-target Clippy | 모두 PASS |
| workspace build·base 고정 manifest/unit policy | 모두 PASS |
| fresh WASM 전체 TSV | 어구 21쪽 최저 93.22327%(7쪽), RowBreak 18쪽 최저 92.47763%(12쪽); 90% 미만·누락 없음 |
| Native/fresh WASM 대표 review gate | 모두 `passed`; 글꼴 불일치 예외 미사용 |
| Native/WASM 영향 페이지 직접 판독 | 어구 2·4·7·10·14·17–21, RowBreak 7·8·11·12; 첫 frame 어구 1쪽과 최저 점수의 어구 9·RowBreak 2쪽도 실제 PNG/PDF 추가 대조 |

locked wrapper의 호스트 `--no-opt` 빌드와 실제 headless Chrome WASM export를 실행했다.
`pkg`와 Studio public의 JS/WASM SHA-256은 각각 같다. Docker daemon을 사용할 수 없어
표준 Docker 배포 빌드는 미실행이다. Studio UI 기능·일반 브라우저 성능은 이번 검증 주장이 아니다.

[검증 명령·입력/산출 해시·페이지별 수치](../pr/assets/issue7207_stored_frame_stage5/validation.json),
[Native 전쪽 TSV](../pr/assets/issue7207_stored_frame_stage5/native-eogu-silhouette.tsv),
[fresh WASM 전쪽 TSV](../pr/assets/issue7207_stored_frame_stage5/wasm-eogu-silhouette.tsv)에 연결했다.
대표 review/overlay는 같은 asset 디렉터리에 Native/fresh WASM으로 나누어 보존했다.

최저 점수 페이지 추가 판독에서 어구 9쪽 우상단 문구의 기존 수직 차이를 확인했다.
다열 분할 셀(row 62, col 4)의 해당 문구 잉크 상단은 같은 96dpi 영역에서 기준 PDF 107px,
기준 binary 80px, 최종 binary 82px였다. 문구는 동일 쪽·셀에 남지만 약 25px 위에 놓인다.
이 셀은 전폭 단일 소유 행 조건에 해당하지 않고 시작 frame의 row 0 경계도 아니다.
점수 통과를 완전한 위치 일치로 보고하지 않으며 이 기존 셀 정렬 차이는 후속 조사 대상으로 남긴다.

이번 후보의 시작 frame 공간·10쪽 주석·17–21쪽 지도/캡션 소속 검사는 충족,
기존의 세부 셀 정렬·선 차이는 미충족(잔여), 별도 합성/성능 경계는 미검증이다.
현재 상태: source `4fc0862d`의 최신 base 통합 뒤 전체 로컬 회귀·lint·Skia·fresh WASM·시각 검증을 모두 완료했다. 원격 push·Open PR 생성·GitHub CI 실행은 승인받았다. [PR #7574](https://github.com/edwardkim/rhwp/pull/7574)를 Open으로 생성했다. [self-review](../pr/archives/pr_7574_review.md)와 오늘할일을 같은 PR의 문서-only 후행 commit에 포함한다. 최종 head의 GitHub CI와 별도 병합 승인은 남아 있다.
