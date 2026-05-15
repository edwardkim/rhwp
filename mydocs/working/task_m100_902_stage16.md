# Task #902 Stage 16 보고서 — 광범위 회귀 + 시각 검증

**Stage**: 16 / 17 (v2 확장)
**상태**: 완료

## 1. 회귀 검증

### 1.1 cargo test --release --all-targets

```
Total passed: 1412 / failed: 0
```

기존 baseline 유지. Stage 12-15 의 raster module 추가 후에도 SVG 경로 회귀 없음.

### 1.2 Golden SVG snapshot

```
cargo test --release --test svg_snapshot
8 passed / 0 failed
```

### 1.3 다중 sample SVG export

| Sample | 페이지 | 결과 |
|--------|--------|------|
| sample14 | 11 | ✓ (Task #860 fixture) |
| sample16 | 64 | ✓ (#902 본 대상) |
| sample17 | 12 | ✓ |
| sample18 | 69 | ✓ |

## 2. RasterPlayer 시각 검증

### 2.1 sample16 (4.7 MB WMF, 20869 records)

| 출력 | 크기 | dimensions |
|------|------|-----------|
| `/tmp/comparison_libreoffice_s16.png` (LO 변환) | 140 KB | 794×1123 |
| `/tmp/comparison_rhwp_s16.png` (resvg SVG 경로) | 292 KB | 1217×824 |
| `/tmp/task902_raster_s16.png` (RasterPlayer 신규) | 92 KB | 2434×1648 |

### 2.2 sample18 (211 KB WMF, 14896 records)

```
/tmp/task902_raster_s18.png: 518788 bytes, 1600×1200
```

### 2.3 시각 quality 한계

본 stage 의 RasterPlayer 구현은 다음 한계:
- **Bitmap records 미구현** (DIBSTRETCHBLT 등 — sample16 의 임베디드 비트맵 영역 누락)
- **font weight/italic 미적용** (synthetic bold/slant 향후 구현)
- **arc/pie/chord 미구현**
- 정확한 굴림체 metric 부재 (시스템 폰트 의존)

## 3. dispatcher 정책

| 경로 | 상태 | 사용처 |
|------|------|--------|
| WMF → SVG → resvg PNG | **default** | 안정, 모든 records 지원 |
| WMF → RasterPlayer → PNG | **opt-in** | `rasterize_wmf_direct_pub()` API |

기존 SVG 경로는 그대로 유지하여 회귀 위험 0. RasterPlayer 의 default 전환은 bitmap 미구현 해결 후 follow-up.

## 4. 산출물

- `/tmp/comparison_libreoffice_s16.png` — LibreOffice 참조 (외부 변환)
- `/tmp/comparison_rhwp_s16.png` — 현재 default 경로 (resvg)
- `/tmp/task902_raster_s16.png` — 신규 RasterPlayer 경로
- `/tmp/task902_raster_s18.png` — sample18 RasterPlayer
- 본 보고서: `mydocs/working/task_m100_902_stage16.md`

## 5. 다음 단계

Stage 17: 최종 보고서 + PR (작업지시자 명시 승인 후)
