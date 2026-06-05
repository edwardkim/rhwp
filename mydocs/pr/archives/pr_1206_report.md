# PR #1206 처리 보고서 — Task #1134: HWPX serializer 문단 id 충돌 수정

- **작성일**: 2026-06-01
- **PR**: #1206 → **CLOSED** (머지 안 함)
- **컨트리뷰터**: @Mireutale (**rhwp 첫 기여자**)
- **연결 이슈**: #1134 "HWPX serializer의 정보 누락 관련 제안" → **OPEN 유지** (미해결)
- **판단**: **닫음** (미완성 모듈 + 무관 변경 혼재 + 정합 미검증)

## PR 내용 (5 파일, +792/−43)

| 파일 | 변경 |
|------|------|
| `serializer/hwpx/context.rs` (+9) | `para_id_counter` + `next_para_id()` — 전역 문단 id 카운터 |
| `serializer/hwpx/section.rs` (+116/−7) | 본문 문단이 `next_para_id()` 사용 |
| `serializer/hwpx/table.rs` (+143/−25) | 셀 문단이 같은 카운터 공유 |
| `serializer/hwpx/header.rs` (+229/−11) | **fillBrush 직렬화 완성** (winBrush/gradation/imgBrush) |
| `tests/hwpx_roundtrip_integration.rs` (+295) | id 유니크 + solid brush 회귀 테스트 |

## 검토 결과

### 1. 문단 id 충돌 수정 (PR 목적) — 적절
- `SerializeContext::next_para_id()` 로 본문·셀 문단이 전역 카운터 공유 → `<hp:p id>` 중복 방지.
- 설계 깔끔, 회귀 테스트(id=0 정확히 1회 등) 의도 명확.
- 트러블슈팅 `cell_split_save_corruption`(id/문단 불일치 → 한컴 손상) 방향과 부합.

### 2. fillBrush 직렬화 완성 (header.rs) — 본문 미설명 + 정합 이슈
- 기존 "Stage 1 빈 fillBrush 래퍼" 를 winBrush/gradation/imgBrush 실제 직렬화로 **완성**.
  PR 목적(id 충돌)과 별개 작업인데 **PR 제목·본문에 미언급** (두 작업 혼재).
- 같은 IR(`Fill`/`SolidFill`/`GradientFill`/`ImageFill`)을 쓰는 HWP 직렬화
  (`control.rs::serialize_shape_fill`)와 방향 정합. brush 는 IR→HWP/HWPX 양쪽에서
  도형·셀 배경 채우기를 직렬화하는 실제 역할이 있음(작업지시자 질문 답).
- **정합 비대칭**: `image_fill_mode_str` 출력 토큰 일부(`LEFT_BOTTOM`/`LEFT_CENTER`/
  `RIGHT_CENTER`/`RIGHT_TOP`/`RIGHT_BOTTOM`/`CENTER_TOP` 등)를 현재 HWPX 파서
  (`parser/hwpx/header.rs:1397`, `section.rs:2900`)가 역으로 못 읽어 `TileAll` 폴백.
  → rhwp 자기 라운드트립 일부 깨짐. 한컴 HWPX 스펙 값인지 미확인.
  (feedback_self_verification_not_hancom 정확히 해당 — 자기 검증조차 일부 불일치.)
- PR 테스트는 id·solid brush 만 검증, image_fill_mode 비대칭 토큰 미검증.

## 닫는 사유

1. **대상이 미완성 모듈**: rhwp 는 HWPX→HWP 직렬화(`exportHwp`, #178 어댑터)만 완성·검증.
   HWPX 쓰기(`serialize_hwpx`/`exportHwpx`)는 미완성. 이 PR 은 그 미완성 모듈을 확장.
   → 모듈 방향·완성 범위는 메인테이너 정비 후 진행이 적절.
2. **무관 변경 혼재**: id 충돌 수정 + fillBrush 완성이 한 PR(본문은 id 만 언급).
3. **정합 미검증**: image_fill_mode 토큰 라운드트립 비대칭 + 한컴 스펙 미확인.

## 컨트리뷰터 제안 (코멘트)

- 첫 기여 환영 + 차분한 사실 중심 피드백(feedback_pr_comment_tone).
- **문단 id 충돌 수정만 분리**한 PR 재제출 요청 (그 자체는 가치 있음).
- fillBrush 직렬화는 HWPX 쓰기 모듈 정비 일정에서 별도 처리(토큰 정합 + 한컴 스펙 대조 포함).

## 비고

- 이슈 #1134 는 미해결 → OPEN 유지 (feedback_no_close_without_approval).
- 검토 과정에서 드러난 두 후속 작업: (a) HWPX 문단 id 전역 유니크, (b) fillBrush 직렬화 +
  image_fill_mode 토큰 ↔ 파서 정합. HWPX 쓰기 모듈 정비 시 반영 대상.
- CI 미실행(serializer/hwpx 변경이 CI paths 트리거 밖) — 닫음으로 무관.
