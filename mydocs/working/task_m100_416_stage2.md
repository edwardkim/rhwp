# Stage 2 완료 보고서 — Task M100 #416

## 작업 내용

차트 회귀 방지 단위 테스트 2 개 추가 + 실제 `hwpspec.hwp` SVG 시각 검증 + 다른 샘플 회귀 점검.

### 차트 회귀 우려 분석

작업지시자 우려: "차트쪽 이미지가 회귀되지 않는지 조심해야 합니다"

**점검 결과 — 회귀 위험 0**:

| 케이스 | Task #195 (가드 있음) | 본 task (가드 제거) | 동작 |
|--------|--------------------|------------------|------|
| HWP 일반 그림, storage_id ≠ 인덱스 (hwpspec) | 가드 실패 → fallback storage_id 검색 → **잘못된 매칭** | **인덱스 매칭** (정확) | 정정 |
| HWP 일반 그림, storage_id == 인덱스 | 인덱스 매칭 (가드 통과) | 인덱스 매칭 | 동일 |
| HWPX 일반 그림 (id = 인덱스+1, 항상 일치) | 인덱스 매칭 (가드 통과) | 인덱스 매칭 | 동일 |
| **HWPX 차트** (id=60001+, 인덱스 범위 밖) | 인덱스 None → fallback id 검색 → 60001 매칭 | 인덱스 None → fallback id 검색 → **60001 매칭** | 동일 |

**근거**:
- HWPX 파서 (`parser/hwpx/mod.rs:99-131`): 일반 BinData 가 먼저 1~N push, 그 뒤 차트가 60001+ id 로 push
- 차트 sparse id 60001+ 는 `bin_data_content.len()` 보다 항상 큼 → 인덱스 범위 밖 → fallback 작동
- 차트의 ID 인 60001+ 가 인덱스 범위 안일 가능성 = **일반 BinData 가 60000+ 개** → 현실적으로 발생 불가
- 차트는 HWPX 전용 (HWP 에는 OOXML 차트 없음, OLE 만)

### 변경 1 — 단위 테스트 2 개 추가 (총 5 → 7)

`src/renderer/layout/utils.rs::tests` 에 추가:

#### `find_bin_data_hwpx_realistic_layout_with_chart`

HWPX 의 실제 push 패턴 (일반 BinData 1~N + 차트 60001+) 모사:

```rust
let v = vec![
    mk(1, "png"),               // index 0 — bin_data_id=1
    mk(2, "png"),               // index 1 — bin_data_id=2
    mk(3, "jpg"),               // index 2 — bin_data_id=3
    mk(60001, "ooxml_chart"),   // index 3 — chart 1
    mk(60002, "ooxml_chart"),   // index 4 — chart 2
];

// 일반 그림: 인덱스 매칭
// 차트: 60000 범위 밖 → fallback id 검색 → 60001/60002 정확 매칭
```

#### `find_bin_data_hwp_hwpspec_page_bg_pattern`

`hwpspec.hwp` 의 실제 BinData 14 개 모사:

```rust
let v = vec![
    mk(0x000C, "png"),  // index 0 — bin_data_id=1 → 페이지 표지 (BIN000C.png)
    mk(0x0001, "bmp"),  // index 1 — bin_data_id=2
    ...
];

// 페이지 표지 — bin_data_id=1 → 인덱스 0 → storage_id=12 (PNG)
let bg = find_bin_data(&v, 1).expect("페이지 표지");
assert_eq!(bg.id, 0x000C, "회귀: bin_data_id=1 이 storage_id=12 가 아님");
```

### 변경 2 — `hwpspec.hwp` SVG 재생성 + 시각 비교

```bash
cargo run --release --bin rhwp -- export-svg samples/hwpspec.hwp -p 0 --debug-overlay -o output/svg/issue-416/
```

| 항목 | 정정 전 (`output/svg/issue-image-regression/`) | 정정 후 (`output/svg/issue-416/`) |
|------|--------------------------------------------|------------------------------|
| 파일 크기 | 42 KB | **260 KB** |
| `<image>` width × height | **16 × 13** (작은 PNG 강제 늘림) | **793.72 × 1121.56** (페이지 전체 자연스럽게) |
| viewBox trick | `viewBox="0 0 16 24.92" preserveAspectRatio="none"` 로 강제 stretch | 없음 — 직접 페이지 크기로 그림 |
| base64 시작 | `iVBOR...uklEQVR4Ae3AA6Ak...` (작은 BMP 의 PNG 변환) | `iVBOR...AlkAAAOoCAIAAABtI3YY` (1137 bytes 정상 PNG) |

→ **페이지 표지 이미지가 의도된 PNG (BIN000C) 로 정상 매칭**.

## 검증

| 항목 | 결과 |
|------|------|
| 단위 테스트 (`cargo test --lib renderer::layout::utils::`) | ✅ **7/7 passed** |
| 전체 lib test (`cargo test --lib`) | ✅ **1023 passed**, 0 failed (1016 + 7 신규) |
| svg_snapshot (`cargo test --test svg_snapshot`) | ✅ 6/6 passed (다른 샘플 무회귀) |
| `samples/hwpspec.hwp` 1 페이지 SVG | ✅ 페이지 표지 정상 (16×13 → 793×1121) |

## 시각 검증 (작업지시자)

산출물 — `output/svg/issue-416/hwpspec_001.svg` 작업지시자 환경에서 시각 확인 부탁드립니다:

| 시나리오 | 기대 |
|----------|------|
| 페이지 배경 | 페이지 표지 PNG 정상 (이전 16×13 잘못된 작은 이미지가 늘려진 형태 아님) |
| 디버그 오버레이 | 문단 / 표 경계 + 인덱스 라벨 정상 표시 |

## 차트 회귀 추가 점검 — 작업지시자 검증 권장

본 저장소 샘플에 HWPX 차트 포함 파일 없음 → 단위 테스트로만 검증 가능.

작업지시자가 **차트 포함 HWPX 파일** 보유 시 다음 점검 권장:

```bash
cargo run --release --bin rhwp -- export-svg <chart_hwpx 파일> -p <차트있는 페이지> -o output/svg/chart-check/
```

차트 위치에 **OOXML 차트 SVG 출력** 이 정상적으로 보이면 회귀 없음.

## 다음 단계

Stage 3 — 자동 검증 종합 + 트러블슈팅 문서 갱신.

## 산출물

- 변경 파일: `src/renderer/layout/utils.rs` (단위 테스트 2 개 추가)
- 시각 산출물: `output/svg/issue-416/hwpspec_001.svg`
- 본 보고서: `mydocs/working/task_m100_416_stage2.md`
