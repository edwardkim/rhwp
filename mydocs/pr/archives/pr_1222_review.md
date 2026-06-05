# PR #1222 검토 — HWPX 본문·표 셀 문단 id 전역 유니크 (PR #1206 분리 재제출)

- **작성일**: 2026-06-01
- **PR**: #1222 (OPEN)
- **컨트리뷰터**: @Mireutale (#1206 컨트리뷰터 — id 충돌만 분리 재제출)
- **연결 이슈**: #1134 → assignee=Mireutale, OPEN
- **base/head**: `devel` ← `fix/issue-1134-para-id-only` (cross-repo)
- **규모**: 4 파일, +159/−7 (context/section/table + 통합테스트)
- **mergeable**: **CONFLICTING / DIRTY** → 로컬 충돌 해소(아래 4절)
- **CI**: 미실행 ("no checks reported") → 로컬 검증 필수
- **라벨**: enhancement / 마일스톤 v1.0.0

## 0. 배경 — PR #1206 분리 재제출

PR #1206(2026-06-01 close)을 검토 시 우리가 명시 요청한 **"문단 id 충돌 수정만 분리 재제출"**에
정확히 응한 PR. #1206 close 3사유 중:
- ② 무관 변경 혼재(fillBrush) → **해소**: header.rs 제외(파일 4개, header.rs 0). 확인 완료.
- ③ image_fill_mode 라운드트립 비대칭 → **해소**: fillBrush 자체가 빠져 무관.
- ① 미완성 HWPX 쓰기 모듈 → 잔존 우려이나, **#1206 검토 시 "id 충돌 수정 그 자체는 적절·가치
  있음"**으로 평가했고 분리 재제출을 요청했으므로 본 PR 은 그 합의의 이행.

## 1. 문제와 원인 (#1206 검토에서 확인)

HWPX 직렬화 시 본문 문단과 표 셀 내부 문단의 `<hp:p id>` 가 같은 번호 공간을 공유하지 않아
전역 중복 가능. 트러블슈팅 `cell_split_save_corruption`(표 바깥 문단 수/ id 불일치 → 한컴 손상)
방향과 부합.

## 2. 수정 내용 검토

| 파일 | 변경 |
|------|------|
| `context.rs` (+9) | `para_id_counter` + `next_para_id()` — 문서 전역 단조 증가 카운터 |
| `section.rs` (+) | 본문 문단·각주 sublist 가 `ctx.next_para_id()` 사용(루프 인덱스 id 제거) |
| `table.rs` (+) | 셀 문단이 같은 ctx 카운터 공유 |
| `tests/hwpx_roundtrip_integration.rs` (+) | basic-table-01 라운드트립 본문+셀 id 전역 유니크 |

설계: 전역 카운터 1개를 본문→셀이 공유 → id 단조 증가·중복 없음. #1206 검토 평가("적절")와 동일.
순수 추가(루프 인덱스 id → 카운터 발급). fillBrush 무관 변경 없음.

## 3. 위험 평가

- **낮음.** HWPX 직렬화 id 발급 로직만. 렌더/파싱 무관. fillBrush 제외로 #1206 우려 ②③ 해소.
- 미완성 HWPX 쓰기 모듈이나 id 유니크는 손상 방지 방향(cell_split 트러블슈팅 부합).

## 4. 충돌 해소 (CONFLICTING)

`table.rs` tests 모듈 단일 충돌. **방금 머지한 #1213(textFlow) 테스트 4건과 #1222 id 테스트 4건이
같은 라인 영역에 추가된 텍스트 인접 충돌**(본문 write_table 등은 자동 머지). 의미 충돌 아님.
해소: 양쪽 테스트 **전부 보존**(textFlow 4 + id 4). 마커 0 확인.

## 5. 검증 결과 (로컬 머지 시뮬 `pr1222-verify`)

| 단계 | 결과 |
|------|------|
| merge | ✅ (table.rs 텍스트 인접 충돌 해소, 본문 자동 머지) |
| fmt / build | ✅ |
| clippy `--lib` | ✅ clean |
| 전체 테스트 `cargo test --tests` | ✅ **1921 passed, 0 failed** |
| id 충돌 단위 4건 | ✅ (셀 유니크 / 카운터 오프셋 / 연속 표 무충돌 / 셀당 복수 문단) |
| **통합 테스트** | ✅ `para_ids_unique_across_body_and_table` — basic-table-01 실제 parse→serialize_hwpx→section XML id 전역 유니크 |
| fillBrush 제외 | ✅ header.rs 미포함(파일 4개) |
| 시각 | (해당 없음 — XML id 변경, 렌더 무관) |

## 6. 판단 — 머지 권고

- #1206 분리 요청 이행(fillBrush 제외 확인), id 충돌 수정은 #1206 검토에서 이미 "적절" 평가.
- 전역 카운터 설계 깔끔, 통합 테스트로 실효 검증, 회귀 0(1921 passed), clippy clean.
- 시각 무관(XML id) — 시각 판정 불필요. 단 HWPX 쓰기는 자체검증 ≠ 한컴 호환
  (`feedback_self_verification_not_hancom`)이나, id 유니크는 손상 방지 방향이고 fillBrush 류
  한컴 스펙 미검증 토큰이 빠져 위험 낮음.
- 승인 시 메인테이너 로컬 `--no-ff` 머지(충돌 해소 포함) + push. 이슈 #1134 클로즈.

## 7. 비고

- CI 미실행(serializer/hwpx 가 CI paths 트리거 밖) → 로컬 직접 검증으로 대체.
- fillBrush 직렬화 + image_fill_mode 토큰 정합은 HWPX 쓰기 모듈 정비 시 별도(#1206 후속 (b)).
- @Mireutale 의 #1206 피드백 반영 재제출 — 차분한 감사 코멘트(feedback_pr_comment_tone).
