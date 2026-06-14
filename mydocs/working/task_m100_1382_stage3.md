# Task M100 #1382 — 3단계 완료 보고서 (xfail·테스트 승격)

- 브랜치: `local/task1382`
- 작성일: 2026-06-13
- 수정 파일: `tests/hwpx_roundtrip_baseline.rs`, `src/serializer/hwpx/table.rs`

## 1. baseline 승격 (3.1)

- `XFAIL`에서 143E433F503322BD33 제거 — `xfail_entries_still_fail` 가드가 강제한
  설계대로 동시 처리. doc 주석에 승격 사유(#1382 해소) 기록.
- 잔존 xfail 4건 = 전부 #1384(borderFillIDRef) 귀속 — **#1382 귀속 xfail 0**.
- `cargo test --test hwpx_roundtrip_baseline` — **4 passed** (143E가 A등급 전수
  대상으로 합류).

## 2. #1387 캡션 테스트 승격 (3.2)

- `task1387_ta_pic_001_r_roundtrip_preserves_caption`: trim_end 완화 제거 →
  **텍스트 완전 일치** + char_offsets(8-jump 포함) 보존 단정 추가.
- 신규 `task1382_ta_pic_caption_autonum_slot_at_midtext`: RT 재파싱 IR에서
  placeholder 위치 4 + 직후 offset 12(8-jump)를 고정 — 1차 한컴 판정의
  "번호 문장 끝 밀림" 재발 방지. (XML 표면 패턴은 2단계
  `task1382_autonum_slot_emitted_at_placeholder`가 고정)

## 3. 검증

- `cargo test --lib serializer::hwpx::table` — 35 passed
- `cargo test --test hwpx_roundtrip_baseline` — 4 passed (신규 xfail 0)
- `cargo fmt --check` 통과

## 4. 다음 단계

4단계 — 전수 배치 + ta-pic SVG(캡션 행 3.5px 시프트 해소) + 143E SVG 회귀 확인 +
매뉴얼·최종 보고서 + CI(release-test) + 한컴 판정 요청.

승인 요청드립니다.
