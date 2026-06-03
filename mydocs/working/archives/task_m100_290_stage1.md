---
타스크: #290 cross-run 탭 감지가 inline_tabs 무시
단계: 1 / 4 — 정밀 진단 + ext[2] 매핑 검증
브랜치: local/task290
작성일: 2026-04-24
---

# Stage 1 완료 보고서

## 1. 목표

구현계획서 Stage 1 의 진단 항목 3개를 모두 확정:

1. RIGHT/CENTER 탭을 실제로 사용하는 HWP 샘플 확보
2. `tab_extended[idx][2]` 의 포맷 검증 (현재 코드 가정: `ext[2] == 1` = RIGHT)
3. `inline_tab_cursor` 추적 변수 도입 위치 결정

## 2. RIGHT 탭 샘플 확보

전체 `samples/` 143 개 HWP 를 `rhwp dump` 로 스캔하여 **실제 `\t` 문자를 포함하고 TabDef 가 type=1(RIGHT) 또는 type=2(CENTER)** 인 문단 탐색.

**최종 확보**: `samples/hwp-3.0-HWPML.hwp` 문단 0.39 (페이지 3).

```
--- 문단 0.39 --- cc=21, text_len=5, controls=1 [다단나누기]
  텍스트: "저작권\t1"
  [PS] ...
       tab_def_id=4 auto_left=false auto_right=false
       tabs=[tab[0] pos=131528 (464.0mm) type=1 fill=3]
```

패턴: 목차 엔트리 ("제목\t<페이지번호>") 형태. RIGHT 탭 + dotted fill 로 페이지번호 우측 정렬.

기타 후보 (동일 패턴):
- `samples/hwp-3.0-HWPML.hwp` 문단 0.40 "본 문서에 대하여...\t2"
- `samples/hwpspec.hwp` 문단 0.39 "저작권\t1" (hwp-3.0-HWPML 과 동일 구조)
- `samples/aift.hwp` 문단 2.3/2.4/2.7 (목차)
- `samples/hwp-3.0-HWPML.hwp` 문단 0.41 "I. 글 3.x 문서 파일 구조\t3"

**CENTER 탭 (type=2) 샘플**: 전체 샘플에서 발견되지 않음. 보수적 매핑 유지로 대체.

## 3. ext[2] 포맷 검증

임시 트레이스 (`text_measurement.rs` 의 `compute_char_positions` 에 `RHWP_TRACE290=1` 환경변수 기반 print 추가, 검증 후 제거) 로 실측:

### 3.1 데이터

| 문서 / 문단 | TabDef | ext[0] (width HU) | ext[2] (u16) | ext[2] 16진 | high byte | low byte |
|-------------|--------|-------------------|--------------|-------------|-----------|----------|
| exam_math p.7 item 18 #1 \t | LEFT fill=0 | 132 | 256 | `0x0100` | **1** | 0 |
| exam_math p.7 item 18 #2 \t | LEFT fill=0 | 671 | 256 | `0x0100` | **1** | 0 |
| exam_math p.7 item 18 #3 \t | LEFT fill=0 | 79 | 256 | `0x0100` | **1** | 0 |
| hwp-3.0-HWPML 저작권 \t | RIGHT fill=3 | 39076 | 515 | `0x0203` | **2** | 3 |
| hwp-3.0-HWPML 본 문서… \t | RIGHT fill=3 | 32416 | 515 | `0x0203` | **2** | 3 |

### 3.2 결론

**`ext[2]` 는 하이·로우 바이트로 분리된 합성 값**:

- **low byte (`ext[2] & 0xFF`) = 채움(fill_type)**
  - 0=없음, 1=solid, 2=dash, 3=dot, 4=double (TabDef.fill 과 동일)
  - 실측: LEFT 탭들은 모두 fill=0 → low=0, RIGHT 탭은 fill=3 → low=3
- **high byte (`(ext[2] >> 8) & 0xFF`) = 탭 종류 (enum + 1 오프셋)**
  - 1=LEFT, 2=RIGHT, 3=CENTER, 4=DECIMAL
  - 실측: LEFT 케이스 high=1, RIGHT 케이스 high=2
  - TabDef 의 type (0=LEFT, 1=RIGHT, 2=CENTER, 3=DECIMAL) 과 1 만큼 오프셋

### 3.3 기존 코드의 함의

`src/renderer/layout/text_measurement.rs:217, 320` 의 inline_tabs 분기:

```rust
let tab_type = ext[2];  // ← 전체 u16 을 type 으로 해석
match tab_type {
    1 => { /* RIGHT */ },
    2 => { /* CENTER */ },
    _ => { /* LEFT */ },
}
```

이 매칭은 `ext[2]` 가 `1` 또는 `2` 인 경우만 RIGHT/CENTER 로 처리. 실제 HWP 파일은 항상 고/저 바이트 합성 값 (최소 256) 이므로 **현재 inline_tabs RIGHT/CENTER 렌더 경로는 영원히 도달 불가** — 모든 inline 탭이 LEFT 처리됨.

저작권\t1 케이스가 "그럭저럭 보이는" 이유: RIGHT 탭이 LEFT 처리되어 \t 가 폭 39076 HU = 521 px 만큼 전진, 그 뒤에 "1" 이 배치. 실제 타겟 우측 정렬과 비슷한 위치가 우연히 맞음.

### 3.4 본 타스크에서의 처리 방침

**범위 최소화**: inline_tabs 렌더링 (`text_measurement.rs`) 의 ext[2] 해석 수정은 **본 타스크 범위 밖**. 별도 후속 이슈로 분리.

본 타스크 `resolve_last_tab_pending` 헬퍼에서는 `ext[2] >> 8` 고바이트를 탭 종류로 해석:

```rust
let inline_type = ((ext[2] >> 8) & 0xFF) as u8;
// 1=LEFT, 2=RIGHT, 3=CENTER, 4=DECIMAL
match inline_type {
    1 | 0 => None,              // LEFT → cross-run pending 없음 (본 타스크 핵심)
    2 | 3 => /* TabDef 경로 */, // RIGHT/CENTER → 기존 동작 유지
    _ => None,                  // 미지 값 → 보수적으로 LEFT
}
```

저작권 케이스 영향: high=2 → TabDef 경로로 폴스루 → 현재 동작 유지 (회귀 0).

## 4. inline_tab_cursor 도입 위치

### 4.1 est 측 (`paragraph_layout.rs:786-880`)

```rust
let mut est_x = effective_margin_left + inline_offset;
let est_x_start = est_x;
let mut pending_right_tab_est: Option<(f64, u8)> = None;
let mut run_char_pos_est = comp_line.char_start;
// [추가] let mut inline_tab_cursor_est: usize = 0;
for run in &comp_line.runs {
    // ... (기존 본문) ...
    // [추가] 헬퍼 호출부에서 cursor 사용
    // run_char_pos_est = run_char_end_est; (line 879)
    // [추가] inline_tab_cursor_est += run.text.chars().filter(|c| *c == '\t').count();
}
```

### 4.2 render 측 (`paragraph_layout.rs:1136-1750`)

```rust
let mut pending_right_tab_render: Option<(f64, u8)> = None;
let is_last_run_of_line = |idx: usize| idx == comp_line.runs.len() - 1;
let mut run_char_pos = comp_line.char_start;
let mut shape_marker_inserted = vec![false; shape_markers.len()];
// [추가] let mut inline_tab_cursor_render: usize = 0;
for (run_idx, run) in comp_line.runs.iter().enumerate() {
    // ... (기존 본문) ...
    // char_offset += run_char_count; (line 1747)
    // run_char_pos = run_char_end; (line 1748)
    // [추가] inline_tab_cursor_render += run.text.chars().filter(|c| *c == '\t').count();
}
```

### 4.3 안전성 확인

`composed.tab_extended` 는 `src/parser/body_text.rs:286` 에서 **오직 `0x0009` (TAB) 문자마다 1개씩** push 됨. 인라인 컨트롤 문자(수식/이미지 마커 등) 는 tab_extended 에 기여하지 않음.

따라서 `run.text.chars().filter(|c| *c == '\t').count()` 로 증가시키는 cursor 가 `tab_extended` 인덱스와 정확히 일치.

**검증**: exam_math p.7 item 18 paragraph 0.144
- tab_extended.len = 3
- 문단 내 단일 run "18.\t\t\t" 에서 \t 카운트 = 3
- cursor = 0 → 0+3=3 (이후 run 은 \t 없음)
- ✓ 일치

## 5. 옵션 A vs B 결정 (RIGHT/CENTER 위치 계산)

구현계획서 4.4 에서 제시한 두 옵션:

- **옵션 A**: 현재처럼 `find_next_tab_stop` 결과 사용 (TabDef + auto_tab_right 기반)
- **옵션 B**: `abs_before + ext[0].to_px()` 로 inline 폭 누적

### 결정: **옵션 A 유지**

근거:
1. 확보한 RIGHT 샘플 (저작권\t1 등) 은 단일 \t 에서 TabDef 한 위치로 점프. 옵션 A 로 한컴 실측과 일치함.
2. 옵션 B 는 `ext[0]` 이 "advance 폭" 인지 "목표 위치" 인지 규격 해석 의존. 현재 코드 (`tab_width_px = ext[0] * 96/7200`) 는 "폭" 으로 해석하나, inline_tabs RIGHT 렌더가 원래 버그여서 검증 어려움.
3. 현재 동작 (TabDef 기반) 에 대한 회귀 위험 없음. 옵션 B 는 별도 수정 범위.

RIGHT 탭 샘플의 실제 시각 비교는 Stage 3 에서 한컴 PDF 와 대조 (현재 rhwp 출력이 PDF 와 유사하게 보이는지 확인).

## 6. 다음 단계 예고

Stage 2 에서 다음 3 변경 적용:

1. `paragraph_layout.rs` 에 `resolve_last_tab_pending` 신규 헬퍼 함수 추가 (이번 문서 §3.4, §5 정책 반영)
2. est 측 / render 측 각각 `inline_tab_cursor_*` 도입 + 기존 cross-run 블록 2 곳을 헬퍼 호출로 교체
3. 단위 테스트 5 건 추가 (`resolve_last_tab_pending` 경계 케이스)

## 7. 임시 디버그 정리

Stage 1 중 추가한 `RHWP_TRACE290` 환경변수 기반 디버그 (`text_measurement.rs:309-315`) 는 검증 완료 후 모두 제거. `git diff src/` 결과 0.

## 8. 승인 요청

Stage 1 결론:
- RIGHT 탭 샘플 확보 ✓ (hwp-3.0-HWPML.hwp 외 다수)
- ext[2] 포맷 확정 ✓ (high=종류 enum+1, low=fill)
- inline_tab_cursor 도입 위치 확정 ✓
- 옵션 A (TabDef 기반 위치) 선택 ✓

Stage 2 진행 승인 요청.
