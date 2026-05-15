# Task #902 최종 결과 보고서 — WMF renderer 근본 개선

**이슈**: [edwardkim/rhwp#902](https://github.com/edwardkim/rhwp/issues/902)
**브랜치**: `local/task902` (base: `upstream/devel @ 72a6bbc3`)
**v1 → v2 전환 + raster Player 확장**: 단일 dx fix → WMF renderer 근본 개선 → LibreOffice emfio 포팅 baseline
**Timebox**: open-ended (작업지시자 결정)

## 1. 처리 개요

### 1.1 본 task 의 진화

| 단계 | scope |
|------|-------|
| v1 (Stage 1~2) | WMF unit scale 정합 — 단일 dx fix |
| v2 (Stage 3~9) | WMF renderer 근본 개선 — DX byte-aware + viewport + flags + 폰트 metric |
| v2 확장 (Stage 10~11) | B2 raster (SVG→PNG via resvg) + POLYPOLYGON fill-rule fix |
| v2 LO 포팅 (Stage 12~17) | LibreOffice emfio (MPL 2.0) raster Player 포팅 baseline |

### 1.2 결정적 발견 (Stage 11)

sample16 vs sample18 의 WMF 구조 차이 — **그림 내용 외에 WMF 자체 구조가 본질적으로 다름**:

| 항목 | sample16 bin3 (문제) | sample18 bin3 (정상) |
|------|---------------------|---------------------|
| 크기 | 4.7 MB | 211 KB |
| **POLYPOLYGON** | **10,476** | **0** |
| **DIBSTRETCHBLT (임베디드 비트맵)** | 177 | 0 |
| EXTFLOODFILL | 168 | 0 |
| SETSTRETCHBLTMODE | 282 | 0 |
| SAVEDC / RESTOREDC | 640 / 640 | 0 / 0 |
| Total records | 20,869 | 14,896 |

sample18 의 정상 = 우리 renderer 의 약점 미사용. sample16 = 복잡 합성 (음영 polygon + 임베디드 비트맵 + flood fill + DC stack) → 우리 renderer 의 갭 다수 노출.

## 2. Stage 별 처리 요약

| Stage | 내용 | 결과 |
|-------|------|------|
| 1 | 다중 sample WMF binary + PDF 추출 | commit |
| 2 | WMF binary 정밀 디코드 + Task #896 분석 오류 정정 | commit |
| **3** | **DX byte-aware indexing + absolute X** | **본 issue ROOT CAUSE fix** |
| 4 | META_SETVIEWPORTEXT/ORG 구현 + MM_ANISOTROPIC ratio | commit |
| 5 | EXTTEXTOUT options flags (ETO_OPAQUE/PDY) | commit |
| 6 | OFFSET WINDOW/VIEWPORT, SCALEVIEWPORTEXT | commit |
| 7 | 폰트 metric (font-family 체인) | commit |
| 8 | 광범위 회귀 검증 | 1412 passed |
| 9 | 최종 보고서 (v2 1차) | commit |
| 10 | B2 raster (WMF SVG → PNG via resvg) | commit |
| **11** | **POLYPOLYGON fill-rule fix + sample16/18 구조 차이 분석** | **결정적 진단** |
| 12 | LibreOffice emfio 포팅 baseline (state context) | commit |
| 13 | raster Player drawing records (polygon/poly_polygon/rectangle) | commit |
| 14 | text rendering (fontdue + LO DrawText 알고리즘) | commit |
| 15 | ellipse / round_rect + dispatcher 공개 API | commit |
| 16 | 광범위 회귀 + 시각 검증 | 1412 passed |
| 17 | 최종 보고서 + PR | 본 보고서 |

## 3. ROOT CAUSE 분석

### 3.1 Stage 3 의 핵심 bug

WMF EXTTEXTOUT 의 DX 배열은 **MBCS byte index**. Korean wide char (2 byte) 는 DX 2 entry 차지.

**기존 버그**:
```rust
for (i, s) in text_content.graphemes(true).enumerate() {
    let dx = *record.dx.get(i - 1).unwrap_or(&0);  // grapheme index — 오류
}
```

**수정**:
```rust
let width = s.width().max(1);  // 1 (ASCII) or 2 (CJK)
let advance: i32 = (0..width).map(|k| dx[dx_idx + k]).sum();
acc_x += advance;
dx_idx += width;
```

LibreOffice 의 `mtftools.cxx::DrawText` 와 동일한 알고리즘.

### 3.2 Stage 11 의 fill-rule fix

POLYPOLYGON 의 다중 서브폴리곤을 단일 `<path>` 의 M/L commands 로 합성 (이전: 별도 `<polygon>` 으로 분리). LO `DrawPolyPolygon` 의 fill-rule (winding/alternate) hole 처리 정합.

## 4. 변경 파일 요약

### 4.1 SVG converter 수정 (Stage 3-7, 10, 11)

| 파일 | 변경 |
|------|------|
| `src/wmf/converter/svg/mod.rs` | DX byte-aware, ETO flags, viewport/offset/scale, POLYPOLYGON path 합성 |
| `src/wmf/converter/svg/device_context.rs` | Viewport struct, offset/scale mutators, MM_ANISOTROPIC ratio |
| `src/wmf/converter/svg/util.rs` | 폰트 substitution garbled filter, Korean fallback chain |
| `src/renderer/svg.rs` | WMF SVG → PNG raster (rasterize_wmf_svg_to_png), RasterPlayer direct (rasterize_wmf_direct) |

### 4.2 새 raster Player 모듈 (Stage 12-15)

| 파일 | 내용 |
|------|------|
| `src/wmf/converter/raster/mod.rs` | LO MPL 2.0 attribution, module entry |
| `src/wmf/converter/raster/state.rs` | RasterState (device context), RasterObject (pen/brush/font) |
| `src/wmf/converter/raster/player.rs` | RasterPlayer (Player trait 구현, tiny-skia 기반) |
| `src/wmf/converter/raster/text.rs` | 폰트 캐시, draw_text_with_dx (fontdue + LO DrawText 포팅) |

### 4.3 도구 + 분석

- `examples/extract_wmf.rs` — WMF binary 추출
- `examples/wmf_record_summary.rs` — WMF records 통계
- `examples/wmf_raster_test.rs` — RasterPlayer 시각 검증 CLI

## 5. 라이센스 정합

LO emfio 알고리즘 참조:
- 공식: https://github.com/LibreOffice/core/blob/master/emfio/source/reader/wmfreader.cxx
- mtftools: https://github.com/LibreOffice/core/blob/master/emfio/source/reader/mtftools.cxx

**MPL 2.0 file-level reciprocity** — 알고리즘이 derived 된 파일 (raster/mod.rs, player.rs, text.rs) 에 LO MPL 2.0 attribution 헤더 유지. rhwp 의 MIT 라이센스와 호환.

## 6. 검증 결과

| 검증 | 결과 |
|------|------|
| `cargo test --release --all-targets` | 1412 passed / 0 failed |
| `cargo test --release --test svg_snapshot` (golden) | 8 / 8 passed |
| sample14 SVG (Task #860 fixture) | 11 페이지 정상 |
| sample16 SVG (#902 본 대상) | 64 페이지 정상 |
| sample17 SVG | 12 페이지 정상 |
| sample18 SVG | 69 페이지 정상 |
| RasterPlayer sample16 WMF | 92 KB PNG (2434×1648) |
| RasterPlayer sample18 WMF | 518 KB PNG (1600×1200) |

## 7. dispatcher 정책

| 경로 | 상태 |
|------|------|
| WMF → SVG → resvg PNG (기존 + Stage 11 fix) | **default** |
| WMF → RasterPlayer (LO 포팅) → PNG | **opt-in API** (`rasterize_wmf_direct_pub`) |

기존 SVG 경로는 모든 WMF records 지원, 안정. RasterPlayer 는 bitmap 미구현 한계로 default 전환은 follow-up.

## 8. 잔존 한계 (follow-up)

| 영역 | 처리 follow-up |
|------|---------------|
| RasterPlayer 의 bitmap records (DIBSTRETCHBLT 등) | image crate decode + pixmap copy 구현 필요 |
| arc / pie / chord | cubic bezier sweep 변환 |
| font weight / italic | synthetic bold/slant 또는 fontconfig variant 매칭 |
| font escapement / orientation | text rotation |
| 굴림체 정확 metric | 폰트 임베딩 (라이센스 고려) |
| sample19 HWP3 parser 실패 | 별도 issue (parser 영역) |

## 9. 산출물

### 9.1 계획서 (mydocs/plans/)

- `task_m100_902.md` (v1 수행 계획서)
- `task_m100_902_impl.md` (v1 구현 계획서)
- `task_m100_902_v2.md` (v2 수행 계획서)
- `task_m100_902_impl_v2.md` (v2 구현 계획서)

### 9.2 Stage 보고서 (mydocs/working/)

- `task_m100_902_stage{1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16}.md`

### 9.3 최종 보고서

- `mydocs/report/task_m100_902_report.md` (본 파일)

### 9.4 Commit (commit 수)

19 commits — Stage 1~16 + fixture + v1/v2 plan 문서

## 10. PR 정보

- Base: `upstream/devel`
- Head: `local/task902`
- Title: "Task #902: WMF renderer 근본 개선 + LibreOffice emfio 포팅 baseline (v2)"
- closes #902

PR 생성 (`gh pr create`) 은 작업지시자 명시 승인 후에만 실행.
