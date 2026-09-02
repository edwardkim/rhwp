# Task M100 #6641 Stage 3 — 기존 source 회귀 계약 강화

- **실행일**: 2026-09-03 KST
- **제품 commit**: `9bc475bfb`
- **integration 배치**: 기존 `tests/issue_838_field_set_value.rs`만 강화
- **파생 source**: 제출 대상 0건

## 1. 강화한 계약

기존 #838 테스트는 값과 일부 본문 offset만 검사했다. 같은 source 안에서 다음 소유자 행렬을
공개 sample 기반으로 고정했다.

- 본문 HWP5 ClickHere 두 필드의 연속 by-id 편집
- HWPX 표 셀 가상 필드의 공개 fieldId 편집
- HWP5 표 셀 ClickHere
- HWP5 표 셀 안 글상자 ClickHere
- 깊이 2 HWPX 표 셀 ClickHere

각 저장 왕복은 필드 값, 문자모양 경계, `char_count`, `char_offsets`, `field_ranges`를 비교한다.
HWP5는 LineSeg 수·시작·vpos의 저장 보존을 요구한다. HWPX는 편집 직후 LineSeg가 모두 bit 31
합성임을 확인하고, 재적재 뒤에도 빈 배열 또는 모두 합성인 비권위 상태만 허용한다.

## 2. 기존 의미 보존

`batch_fill_contract` 25건으로 다음을 함께 확인했다.

- 정상 `samples/field-01.hwp` 채움: exit 0, `identical=true`, `diffCount=0`
- 미지정 필드: 문서 생성은 유지하는 partial success
- 깨진 입력 행: 레코드를 누락하지 않고 전체 종료 1
- 인자 오류: 기존 usage 실패
- dry-run: 파일 무작성
- 병렬 실행: 행 순서·파일명 결정론 유지

#3380·#3545는 안내문과 같은 실제 값을 채웠을 때 dirty bit와 값이 저장 왕복에서 살아남는지
확인했다. #4402와 field-begin emission 계약은 형제 필드의 안내문·range가 section 재직렬화 뒤에도
보존되는지 확인했다. #2724 passthrough 가드는 by-id 직접 무효화에 맞춰 stale 면제가 제거됐음을
확인했다.

## 3. focused 결과

| suite/source | 결과 |
| --- | ---: |
| `issue_838_field_set_value` | 6/6 PASS |
| `batch_fill_contract` | 25/25 PASS |
| `issue_3380_field_value_equals_guide` | 3/3 PASS |
| `issue_3545_clickhere_dirty_roundtrip` | 8/8 PASS |
| `issue_2724_passthrough_invalidation_guard` | 5/5 PASS |
| `issue_4402_hwp5_guide_residue_roundtrip` | 7/7 PASS |
| `field_begin_emission_order` | 1/1 PASS |
| `issue_1893` | 1/1 PASS |
| **합계** | **56/56 PASS** |

manifest check 기준은 `1120 sources / 4808 static test attrs / 48/48 integration targets /
nextest 최소 6559 cases`다. `cargo fmt --all`과 `git diff --check`도 통과했다.

## 4. Stage 4 인계

Stage 0~3의 로컬 구현·집중 회귀는 끝났다. 다음 단계는 AGENTS.md가 요구하는 native/WASM/workspace
Clippy와 전체 integration을 같은 exact head에서 순차 실행하는 장시간 게이트다. 해당 실행은 수행계획의
별도 승인 전 시작하지 않는다. 이후에만 #6628 BO05·BO15 canary와 Gym 전수를 분리해 진행한다.
