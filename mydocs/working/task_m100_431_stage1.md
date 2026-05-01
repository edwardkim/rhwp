# Task #431 Stage 1 — 베이스라인 측정 + dump-pages 보강

## dump-pages 보강

`src/document_core/queries/rendering.rs::dump_page_items` 의 `PartialTable` 출력에 split 정보 추가:

```rust
let split_info = if *split_start_content_offset > 0.0 || *split_end_content_limit > 0.0 {
    format!("  split_start={:.1} split_end={:.1}", split_start_content_offset, split_end_content_limit)
} else {
    String::new()
};
```

## 베이스라인 측정

`samples/synam-001.hwp` 페이지 13~17 의 PartialTable + SVG 정합:

| 페이지 | rows | split_start | split_end | SVG bytes | text 갯수 | 정상/결함 |
|--------|------|-------------|-----------|-----------|----------|----------|
| 13 | 2..7 | 510.7 | 315.1 | (정상) | (정상) | 정상 |
| 14 | 6..7 | 315.1 | 965.4 | 237,476 | **859** | 정상 |
| **15** | **6..7** | **1280.6** | **965.4** | **3,230** | **6** | **결함 — 빈 페이지** |
| 16 | 6..7 | 2246.0 | 0.0 | 335,623 | 1,210 | 정상 |
| 17 | 7..8 | (없음) | | (정상) | (정상) | 정상 |

dump-pages 결과는 페이지 14, 15, 16 모두 `rows=6..7 cont=true` 동일 — split_start/end 만 다름. 페이지 15 만 SVG 빈 페이지.

## 결함 본질 확정

### `compute_cell_line_ranges` 의 라인 2160-2171:

```rust
let line_end_pos = cum + line_h;  // 누적 위치 (절대 좌표)

if has_offset && line_end_pos <= content_offset {
    // 이전 페이지에서 완전히 렌더링됨 → 스킵
    cum = line_end_pos;
    continue;
}

if has_limit && line_end_pos > content_limit {  // ← **단위 mismatch!**
    // limit 초과 → 이 줄과 이후 모든 콘텐츠 차단
    break;
}
```

### 단위 mismatch

- **`line_end_pos = cum + line_h`** = **절대 좌표** (셀 시작부터의 누적 px)
- **`content_offset`** = **절대 좌표** (의도 정합 — 이전 페이지까지의 누적)
- **`content_limit` = `split_end_limit` = `avail_content`** = **상대 길이** (현재 페이지에서 표시할 영역의 px)

### 케이스별 분석 (`is_split_start_row=true && is_split_end_row=true` — 한 행 안 split_start + split_end 둘 다 적용)

| 페이지 | content_offset | content_limit | cum 시작 | cum 끝 (=offset) | line_end_pos > limit | 결과 |
|--------|---------------|---------------|---------|------------------|---------------------|------|
| 14 | 315.1 | 965.4 | 0 | ~315.1 (스킵) | False (315.1 < 965.4) | 정상 ~ 965.4 까지 표시 |
| **15** | **1280.6** | **965.4** | 0 | **~1280.6 (스킵)** | **True (1280.6 > 965.4)** | **즉시 break — 빈 페이지** |
| 16 | 2246.0 | 0.0 | 0 | ~2246.0 (스킵) | (has_limit=false, limit 미적용) | 정상 ~ 끝 까지 표시 |

페이지 14 는 우연히 정상 — `content_offset (315.1) < content_limit (965.4)` 라 영역 겹침. 페이지 15 는 결함 — `content_offset (1280.6) > content_limit (965.4)` 라 영역 안 겹침 + 즉시 break.

## 정정 방향 (Stage 2 에서 결정)

### 정정안 A — content_limit 을 절대 좌표로 변환

```rust
let abs_limit = if has_limit { content_offset + content_limit } else { 0.0 };

if has_limit && line_end_pos > abs_limit {
    break;
}
```

### 정정안 B — line_end_pos 를 상대 좌표로 비교

```rust
if has_limit && (line_end_pos - content_offset) > content_limit {
    break;
}
```

두 안 모두 같은 결과. **정정안 A 가 의미 명확** (`abs_limit = 페이지에 표시할 절대 끝 좌표`).

## 회귀 origin 다시 점검

bisect 결과 commit `af6753f5` (Task #362) 가 회귀 origin. 그러나 위 결함은 **Task #362 이전부터 존재했을 가능성** — Task #362 가 `bigger_than_page` 가드 도입으로 본 결함 케이스가 발현된 가능성.

Stage 2 에서 정정 적용 후 광범위 회귀 점검에서 — Task #362 의 정정 의도 (kps-ai 88→79) 가 보존되는지 확인 필수.

## 다음 단계

Stage 2: `compute_cell_line_ranges` 의 `content_limit` 비교 정합화 (정정안 A 적용).
