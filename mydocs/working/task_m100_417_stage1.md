---
타스크: #417 HWP 3.0 정식 파서 구현
단계: Stage 1 — debug println! 제거 + untracked 파일 커밋
브랜치: local/task417
작성일: 2026-04-28
상태: 완료
---

# Stage 1 완료 보고서

## 1. 작업 내용

### 1.1 debug println! 4건 제거

`src/parser/hwp3/mod.rs`:
- 617번: `println!("TABLE caption_pos byte: {}", caption_pos);` — 제거
- 625번: `println!("TABLE mapped caption_direction: {:?}", caption_direction);` — 제거
- 768번: `println!("PICTURE caption_pos byte: {}", caption_pos);` — 제거
- 776번: `println!("PICTURE mapped caption_direction: {:?}", caption_direction);` — 제거
- 1518-1519번: 불필요 주석 2줄 — 제거

유지: 760번 `eprintln!` (그리기 객체 파싱 실패 로그, 정당한 사용)

### 1.2 clippy 오류 3건 수정

- `drawing.rs:758`: `.len() > 0` → `!is_empty()` — E0308 clippy 오류 수정
- `mod.rs:1044`: `.get(0).copied()` → `.first().copied()` — clippy 권장 수정
- `mod.rs:1254`: 비어있는 `if para.char_count > 0 {}` 블록 제거

## 2. 검증 결과

| 항목 | 결과 |
|------|------|
| `cargo test --lib` | 1016 passed / 0 failed / 1 ignored |
| `cargo clippy -- -D warnings` | clean |
| `rhwp export-svg samples/hwp3-sample.hwp` | 20 SVG 생성 성공 |
| debug println! 출력 | 0건 (완전 제거) |

## 3. 렌더링 상태 (export-svg 검증)

파싱→렌더러 전달: **성공** — "문서 로드 완료: samples/hwp3-sample.hwp (20페이지)"

### 알려진 LAYOUT_OVERFLOW 이슈 (이번 타스크 범위 외)

LAYOUT_OVERFLOW 경고가 발생하나 SVG 생성 자체는 정상 완료됨.

**원인**: HWP3 파일은 pre-computed vpos (줄 배치 위치)를 저장하나, HWP5 레이아웃 엔진은 이를 무시하고 font metrics 기반으로 reflow. 재계산 줄 높이가 HWP3 원본과 다르면 페이지마다 높이 차이가 누적되어 overflow 발생.

- 꼬리말 overflow: 36px — 꼬리말 fixed_line_spacing(24.5mm)이 footer 높이(15mm)를 초과
- 본문 overflow: 일부 페이지 136-376px — HWP3 vpos↔reflow 불일치 누적

이 이슈는 레이아웃 엔진 개선 범주로 별도 이슈 등록 필요.

## 4. 커밋 대상 파일

- `src/parser/hwp3/mod.rs` — println! 제거, clippy 수정
- `src/parser/hwp3/drawing.rs` — clippy 수정  
- `src/parser/hwp3/` (전체 9개 파일) — git add (untracked → tracked)
- `src/bin/dump_pictures.rs` — git add (untracked → tracked)
- `src/parser/mod.rs` — 변경분 커밋
- `mydocs/` — 문서 파일들 (orders, plans, working)

## 5. 다음 단계

Stage 2: 하이퍼링크 URL 추출 구현 (추가정보블록 #1 TagID 3)
