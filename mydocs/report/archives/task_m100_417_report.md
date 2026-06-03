---
타스크: #417 HWP 3.0 정식 파서 구현
마일스톤: M100
브랜치: local/task417
작성일: 2026-04-29
상태: 완료
---

# 최종 결과 보고서

## 1. 타스크 개요

HWP 3.0 형식 파일(`.hwp`)을 파싱하여 HWP5 Document IR로 변환하는
`parse_hwp3()` 파이프라인을 구현하고, 렌더링 오류를 수정한다.

GitHub Issue: #417

## 2. 구현 단계 요약

### Stage 1 — HWP3 파서 초기 구현 + 코드 정리

- `src/parser/hwp3/` 9개 파일 (mod, records, paragraph, encoding, johab, johab_map, special_char, drawing, ole) 신규 추가
- `parse_hwp3()` → Document IR 변환 파이프라인 완성
- debug `println!` 4건 제거 (TABLE/PICTURE caption_pos, caption_direction 로그)
- `src/parser/mod.rs`: HWP3 파일 식별 및 라우팅 (매직 바이트 확인)
- `src/bin/dump_pictures.rs`: 그림/캡션 덤프 유틸

### Stage 2 — 그림 겹침 수정 + 캡션 렌더링 복원 + AutoNumber 번호 수정

**수정 1**: 후속 단락 겹침 (picture_footnote.rs)
- Para-relative 그림 반환 y를 `y_offset+h` → `base_y+h`로 수정
- vert_offset이 적용된 실제 그림 상단(base_y)을 기준으로 후속 단락 배치

**수정 2**: 캡션 렌더링 복원 (hwp3/mod.rs)
- HWP3 파서에서 모든 그림의 `caption.width = 0` → `pic.common.width`로 보정
- 0px 캡션 박스 문제 해결

**수정 3**: AutoNumber U+FFFC 처리 (paragraph_layout.rs)
- HWP3 AutoNumber 위치를 U+FFFC(OBJECT REPLACEMENT CHARACTER)로 저장
- `apply_auto_numbers_to_composed`에 `'\u{fffc}'` 패턴 탐색 추가

**수정 4**: HWP3 그림 번호 수정 (hwp3/mod.rs + parser/mod.rs)
- HWP3에서는 `Control::Picture` 개체 자체(캡션 유무 무관)가 그림 카운터를 올림
- `doc.header.version.major = 3` 설정으로 HWP3 판별
- `assign_auto_numbers_in_controls(is_hwp3=true)` 분기 추가
- 꼬리말 로고(tac=true)가 카운터 1 선점 → 본문 그림이 2~5로 정상 채번
- 결과: 그림2./그림3./그림4./그림5. (한컴과 일치)

**알려진 한계** (별도 이슈 #425 등록):
- HWP3 혼합 단락(텍스트+Para-relative 그림) 내 텍스트/그림 겹침
- vpos 기반 라인 배치 미구현으로 인한 reflow 불일치

### Stage 3 — 하이퍼링크 URL 추출

- 추가정보블록 #1 TagID 3 (하이퍼텍스트 정보, 스펙 §8.3) 파싱
- 각 항목 617바이트: `data[0..256]` = URL (kchar, null 종료)
- 등장 순서 기준으로 `Control::Hyperlink.url`에 순차 매핑
- 샘플 파일에 하이퍼링크 없어 직접 검증 불가 (구현은 스펙 기준)

### Stage 4 — 최종 검증 + 보고서

- debug println! 재확인: Stage 1에서 이미 제거 완료
- 최종 테스트 통과 확인
- 보고서 작성 및 커밋

## 3. 최종 검증 결과

| 항목 | 결과 |
|------|------|
| `cargo test --lib` | 1016 passed / 0 failed / 1 ignored |
| `cargo clippy -- -D warnings` | clean |
| `rhwp export-svg samples/hwp3-sample.hwp` | 20 SVG 생성 성공 |
| WASM 빌드 + rhwp-studio 확인 | 정상 (작업지시자 검증) |

### 렌더링 결과 (hwp3-sample.hwp)

| 항목 | Stage 1 전 | 최종 |
|------|-----------|------|
| 캡션 렌더링 | 불가 (width=0) | ✓ |
| 그림 번호 표시 | 미표시 (U+FFFC 미처리) | ✓ |
| 그림 번호 값 | — | 그림2./3./4./5. (한컴 일치) |
| 후속 단락 겹침 | 발생 (pi=77↔pi=76) | ✓ 수정 |
| 하이퍼링크 URL | String::new() | 추가정보블록에서 추출 |

### 알려진 미해결 LAYOUT_OVERFLOW (Stage 1 이래 동일)

| 위치 | overflow | 원인 |
|------|---------|------|
| 꼬리말 (전 페이지) | 36px | footer height(15mm) < fixed_line_spacing(24.5mm) |
| page 3 pi=41 Shape | 136px | HWP3 vpos↔reflow 불일치 (#425) |
| page 8 pi=78 Shape | 124px | HWP3 vpos↔reflow 불일치 (#425) |
| page 10 pi=97 Shape | 16px | HWP3 vpos↔reflow 불일치 (#425) |

## 4. 수정 파일 목록

| 파일 | 변경 |
|------|------|
| `src/parser/hwp3/mod.rs` | caption.width 보정, version.major=3, 하이퍼링크 URL 추출 |
| `src/parser/mod.rs` | is_hwp3 분기, assign_auto_numbers_in_controls |
| `src/renderer/layout/picture_footnote.rs` | base_y 기반 반환 |
| `src/renderer/layout/paragraph_layout.rs` | U+FFFC AutoNumber 처리 |
| `src/renderer/pagination/engine.rs` | 주석 추가 |
| `src/bin/dump_pictures.rs` | 신규 (그림 덤프 유틸) |
| `src/parser/hwp3/` (전체) | 신규 (9개 파일) |
| `src/bin/` | 신규 |

## 5. 파생 이슈

| 이슈 | 내용 |
|------|------|
| #425 | HWP3 혼합 단락 텍스트/그림 겹침 (vpos 미사용) |
