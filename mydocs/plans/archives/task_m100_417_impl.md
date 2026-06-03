---
타스크: #417 HWP 3.0 정식 파서 구현 — parse_hwp3() → Document IR
구현계획서
브랜치: local/task417
작성일: 2026-04-28
---

# 구현계획서

## 1. 사전 조사 결과

### 1.1 파일 구조

```
src/parser/hwp3/
├── mod.rs          # 메인 파서 (1529줄) — parse_hwp3(), convert_char/para_shape()
├── records.rs      # HWP3 구조체 (DocInfo, DocSummary, CharShape, ParaShape, Style 등)
├── paragraph.rs    # ParaInfo, LineInfo
├── encoding.rs     # decode_hwp3_string() — MBCS → UTF-8
├── johab.rs        # decode_johab() — 조합형 → 유니코드
├── johab_map.rs    # JOHAB_SYMBOLS 매핑 테이블
├── special_char.rs # Hwp3SpecialChar 파싱
├── drawing.rs      # 그리기 객체 트리 파싱 (792줄)
└── ole.rs          # OLE 개체 구조체 (147줄)

src/bin/
└── dump_pictures.rs  # 그림/캡션 덤프 유틸 (33줄)
```

### 1.2 debug println! 위치

`src/parser/hwp3/mod.rs`:
- 617번: `println!("TABLE caption_pos byte: {}", caption_pos);`
- 625번: `println!("TABLE mapped caption_direction: {:?}", caption_direction);`
- 768번: `println!("PICTURE caption_pos byte: {}", caption_pos);`
- 776번: `println!("PICTURE mapped caption_direction: {:?}", caption_direction);`
- 1518-1519번: 불필요 주석 2줄

유지 대상: 760번 `eprintln!` (그리기 객체 파싱 실패 로그 — 정당한 사용)

### 1.3 하이퍼링크 미추출 위치

`src/parser/hwp3/mod.rs` 938번:
```rust
controls.push(crate::model::control::Control::Hyperlink(crate::model::control::Hyperlink {
    url: String::new(), // TODO: TagID 3에서 추출
    text: text.trim().to_string(),
}));
```

### 1.4 하이퍼링크 스펙 (`mydocs/tech/한글문서파일구조3.0.md` §8)

추가정보블록 #1:
- TagID 1: 포함 이미지 — 현재 처리 중
- TagID 3: 하이퍼링크 정보

TagID 3 데이터 구조 (스펙 §8 "하이퍼링크" 항목):
```
하이퍼링크 개수: WORD (2B)
반복 {
  hyperlink_id: WORD (2B)   ← special_char의 FieldCode data[0..2]와 매칭
  url_length:   WORD (2B)
  url:          hchar[url_length]
  bookmark_len: WORD (2B)
  bookmark:     hchar[bookmark_len]
  macro_len:    WORD (2B)
  macro:        hchar[macro_len]
}
```

구현 방법:
1. `parse_hwp3()` 내 추가정보블록 순회에서 TagID 3 발견 시 `HashMap<u16, String>` 구성
2. `parse_paragraph_list()` 호출 시 hyperlink 맵 참조 전달
3. 938번: `url: hyperlink_map.get(&hyperlink_id).cloned().unwrap_or_default()`

## 2. 단계 구성

### Stage 1 — debug println! 제거 + untracked 커밋

**작업:**
1. `mod.rs` 617, 625, 768, 776번 줄 `println!` 4건 제거
2. `mod.rs` 1518-1519번 불필요 주석 2줄 제거
3. `cargo test` 통과 확인
4. `cargo clippy -- -D warnings` clean 확인
5. git add + 커밋:
   - `src/parser/hwp3/` (9개 파일)
   - `src/bin/dump_pictures.rs`
   - `src/parser/mod.rs`
   - `mydocs/` 문서 (orders, plans, working)

**완료 기준:**
- `cargo test` 전체 통과
- `cargo clippy` clean
- `rhwp export-svg samples/hwp3-sample.hwp` — SVG 출력 정상
- git 미추적 파일 0건

### Stage 2 — 하이퍼링크 URL 추출 구현

**작업:**
1. `mydocs/tech/한글문서파일구조3.0.md` §8 TagID 3 스펙 재확인
2. `parse_hwp3()` 추가정보블록 파싱 부분에 TagID 3 처리 추가
3. `HashMap<u16, String>` 구성 후 `parse_paragraph_list()` 전달 (파라미터 추가 또는 별도 postprocess)
4. `mod.rs` 938번 `url: String::new()` → 실제 URL 대입
5. `cargo test` 통과 확인

**완료 기준:**
- 하이퍼링크가 있는 HWP 3.0 샘플에서 URL 추출 확인
- `cargo test` 전체 통과
- `cargo clippy` clean

### Stage 3 — 최종 검증 + 보고서

**작업:**
```bash
cargo test
cargo clippy -- -D warnings
rhwp export-svg samples/hwp3-sample.hwp
rhwp dump samples/hwp3-sample.hwp
```

최종 보고서 `mydocs/report/task_m100_417_report.md` 작성.
오늘할일 `mydocs/orders/20260428.md` 상태 갱신.

## 3. 수정 대상 파일

| 파일 | 변경 |
|------|------|
| `src/parser/hwp3/mod.rs` | println! 제거, 하이퍼링크 URL 추출 |
| `src/parser/hwp3/` (전체) | git add (신규 추적) |
| `src/bin/dump_pictures.rs` | git add (신규 추적) |
| `src/parser/mod.rs` | 변경분 커밋 |
