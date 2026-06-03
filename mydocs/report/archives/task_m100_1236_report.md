# 최종 결과 보고서 — Task #1236: 미주(해설) 영역 줄간격 간헐적 좁음

- **이슈**: #1236 (M100 / v1.0.0)
- **브랜치**: `feature/issue-1236-endnote-line-spacing` (base: `stream/devel` f83c43b5)
- **기간**: 2026-06-02
- **성격**: 레이아웃 버그 수정 — 미주 줄간격 (조건 게이트)

## 1. 문제

`3-11월_실전_통합_2022.hwpx` 풀이(미주) 페이지(10~14쪽)에서 줄간격이 **간헐적으로** 좁아짐
(전역이 아니라 특정 줄 전환). 작업지시자가 문8·문11·문15·문19·문22·문24 등 다수 지적.

## 2. 원인 (확정)

`src/renderer/layout/paragraph_layout.rs` 의 **다줄 미주 문단**(`endnote_line_vpos_base`,
L1374 게이트 `end > start_line+1`) 줄 배치에서, **마지막 줄의 trailing 줄간격(line_spacing,
~6px)을 0 으로 떨어뜨림**:

```rust
let trailing = if line_idx + 1 < end { line_spacing_px } else { 0.0 };
```

→ 다줄 미주 문단 다음 문단이 `line_height`(14px)만큼만 내려가 좁아짐(정상 20px = lh+ls).
단일줄 미주 문단·일반 문단은 else 분기에서 줄간격을 항상 더해 정상 → **다줄 미주 문단
경계에서만 간헐적**으로 발생(사용자 "간헐적" 증상과 정확히 일치).

## 3. 수정 — 조건 게이트

무조건 줄간격 추가는 issue_1139(문제-사이 7mm 2배)·issue_1189(페이지네이션 연쇄) 회귀를
유발(미주 문단 높이가 페이지네이션과 결합, 메모리 `tech_lazy_base_trailing_ls_gate` 경고).
따라서 **"다음 문단이 같은 미주(문제) 내 연속 문단"일 때만** 마지막 줄 줄간격을 적용:

| 파일 | 변경 |
|------|------|
| `layout.rs` | `endnote_para_has_same_endnote_successor(para_index)` — `endnote_para_sources` 의 `(section,para,control)_index` 비교로 같은 미주 연속 판별 |
| `paragraph_layout.rs` | 마지막 줄 게이트: `line_idx+1 < end \|\| 같은_미주_연속` |

- 같은 풀이 내 연속(문19 "하면"→"f'(x)") → 줄간격 적용(20px). ✓
- 문제 경계(미주 마지막 문단, between-notes margin) → 0 유지 → 중복 가산·회귀 차단. ✓

## 4. 검증

| 항목 | 결과 |
|------|------|
| 문19 "하면→f'(x)" 간격 | 14.1 → **20.1px** (PDF 정합) |
| 지적 페이지 narrow(13~16px) | 10쪽 1→0, 11쪽 3→0, 12쪽 4→0, 14쪽 3→1 |
| 미주 정밀 테스트(issue_1139/1189) | 전부 통과 |
| 골든 스냅샷 | 8 passed |
| **전체 `cargo test`** | **1933 passed, 0 failed** |
| 페이지 수 | 21 불변 |

시각: `output/poc/task1236/mun19_gate_fix.png` (수정전 narrow → 수정후 균일 = PDF).

## 5. 변경 파일

- `src/renderer/layout.rs` (헬퍼)
- `src/renderer/layout/paragraph_layout.rs` (게이트)

## 6. 범위 밖 / 후속 이슈

본 건(다줄 문단 내부 줄간격)과 별개로, 같은 미주 영역에서 조사 중 발견된 2건은 별도 이슈로 분리:

- **#1238** 미주 문제 제목 앞 between-notes margin 누락(문22 등) — 다줄 문단으로 끝나는 문제
  다음 제목이 이전 줄에 붙음. between-notes 가 vpos_offset/last_seg 두 메커니즘으로 적용되며
  문서별로 상이, 무조건 수정은 issue_1139/1189 회귀. 단일줄 미주 render 경로 추적 선결.
- **#1239** 수식 다행(S=…) 줄 병합(문20, 4줄→3줄) — 수식 레이아웃 별개 서브시스템.
- 각주(footnote) 줄간격: 본 건은 미주 한정. 동일 점검은 필요 시 별도.

## 7. 조사 경위 메모

본 건은 black-box 픽셀 측정(수식 첨자 노이즈)으로 난항했으나, 렌더 SVG 의 정확 좌표 추출 +
런타임 분기 진단으로 "다줄 미주 문단 마지막 줄 trailing 줄간격 드롭"을 특정. 무조건 수정의
회귀를 4개 정밀 테스트로 확인 후 "같은 미주 연속" 게이트로 해소.
