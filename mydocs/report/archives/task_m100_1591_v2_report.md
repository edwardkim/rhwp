# Task #1591 v2 최종 보고서 — first-para hidden 슬롯 정합 (#1593 통합)

## 요약

fidelity 잔여 IR_DIFF Class C 3건(#1591 char_shape +8 ×2, #1593 fieldEnd 드롭
−16/−8)을 **단일 근원 수정**으로 전부 해소했다. 근원은 첫 문단(para0)의 hidden
슬롯(secPr/템플릿 흡수 colPr)이 cc 축의 8유닛을 점유하는데 방출 슬롯 목록에서
빠져 **첫 문단이 구조적으로 mismatch 폴백(위치추정 포기)에 진입**하는 것.
1라운드(#1591, 불채택·롤백)가 북마크 hoist 를 원인으로 오인했던 케이스의 재규명 완성.

- 통제 비교(samples 전수 28,844건): **개선 4 / 악화 0 / 순효과 +4**
  (타깃 3건 + 동형 보너스 1건(36401777) — 1라운드 순효과 0 과 대조).
- 이슈: #1591, #1593 / 브랜치: `local/task1591-v2`
  (upstream/devel + 제출 열린 PR 20건 선적용 베이스)
- 수정: `src/serializer/hwpx/section.rs` 단일 파일 (3점)

## 근본 원인 (1단계, `mydocs/working/task_m100_1591_v2_stage1.md`)

원본 XML·IR 위치 축 대조로 확정 (36384689: cc=33 = [secd 0..8][cold 8..16]
[tbl 16..24][pageNum 24..32] + 1):

- `inferred_control_slot_count`(cc 증거) = 4 는 **물리 축** — secPr/colPr 포함.
- 방출 `slots` 는 SectionDef(템플릿)·첫 ColumnDef(#1584 흡수)를 제외 → 항상
  `slot_count != slots.len()` → mismatch 폴백.
- mismatch 폴백의 결함 2종이 Class C 로 발현:
  - **C1 (+8)**: 슬롯을 char-offset 없이 일괄 방출 → 후위 슬롯(pageNum)이
    char_shape 경계 앞으로 이동.
  - **C2 (cc −8)**: 균형 `field_ranges` 의 닫는 fieldEnd 방출 코드 부재 → 소실.
  - 부수: 비-empty 문단 북마크 hoist(문단 시작 강제) → 순서 파괴.

## 수정 (2단계, `mydocs/working/task_m100_1591_v2_stage2.md`)

1. **hidden 슬롯 정합** (핵심): cc 증거가 hidden 의 점유를 보여주면
   (`slot_count == slots.len() + hidden.len()`) SectionDef·첫 ColumnDef 를 위치
   슬롯으로 편입 — XML 은 방출되지 않고(SectionDef: arm 없음, 첫 ColumnDef:
   consume-once) 위치 축만 8유닛씩 전진. 첫 문단이 **메인 경로(UTF-16 위치 축)**
   로 진입해 후위 슬롯·fieldEnd 가 정위치에 방출된다. 증거 불일치 시 종전 동작
   (자가 검증 — 오진입 없음).
2. **북마크 in-order 통일**: 슬롯 있는 문단의 hoist 제거(1라운드 교정 편입) +
   메인 경로에 in-order 방출 지점 추가. 1라운드 RED
   `task1591_bookmark_not_hoisted_before_slot` **`#[ignore]` 해제 → GREEN**.
3. **mismatch 폴백 보강**: 잔존 폴백에도 균형 fieldEnd 말미 복원(#1556 동형).

## 검증

### 통제 비교 (3단계, 채택 게이트 — `mydocs/working/task_m100_1591_v2_stage3.md`)

- BEFORE(수정 직전 커밋 빌드) vs AFTER, `hwpdocs/samples` 전수 28,844 hwpx:
  **공통 28,855건 집계 — 개선 4 / 악화 0 / 순효과 +4** (`tools/roundtrip_control_compare.py`).
- 비-PASS 원시 레코드 교차검증: BEFORE 8건 → AFTER 4건, 소거분 = 개선 4건과 일치,
  잔존 4건은 양측 동일(별개 클래스) — **악화 0 재확정**.

### 회귀 게이트

| 게이트 | 결과 |
|--------|------|
| 타깃 3건 `hwpx-roundtrip` | PASS (diff=0, r2=0) ✅ |
| opengov 스냅샷 (20건, 타깃 3건 신규 편입) | IR_DIFF→PASS 승격 후 통과 ✅ |
| `hwpx_roundtrip_baseline` | 통과 ✅ |
| `cargo test --lib` | 2054+ passed / 0 failed ✅ |
| `cargo test --release` 전체 | **2751 passed / 실질 실패 0** (7건 "실패"는 svg_snapshot Windows autocrlf CRLF 노이즈 — #1786 동일 목록, 내용 diff 0) ✅ |
| 합성 가드 테스트 | `task1591v2_first_para_hidden_slot_char_shape_position`, `task1593_first_para_same_para_field_end_preserved` 신규 GREEN ✅ |

## 산출물

- 소스: `src/serializer/hwpx/section.rs`
- 테스트: 합성 2건 신규 + 1라운드 RED 해제 1건 + opengov 실파일 가드 3건
  (`samples/hwpx/opengov/` 36384689·36385445·36388711, 공개 결재문서 14~51KB)
- 검증 도구: `tools/roundtrip_control_compare.py` (통제 비교 집계, 악화 시 exit 1)
- 문서: 수행·구현 계획서, stage1~3 보고, 본 보고서

## 한계·후속

- 통제 비교 잔존 IR_DIFF 4건(36372352·36401553·36380743·36399822)은 본 수정과
  무관한 별개 클래스 — 필요 시 후속 이슈로 분류.
- 비-empty·무슬롯 문단의 북마크는 종전(문단 시작) 유지 — IR 에 북마크 char-offset
  이 없어 순서 증거가 없는 영역(현행 무회귀 우선).

## 이슈 처리 제안

- #1591: 수용 기준 1·3·4 충족 → close 요청.
- #1593: fieldEnd 드롭·북마크 hoist 결합 해소(36388711 PASS) → close 요청.
