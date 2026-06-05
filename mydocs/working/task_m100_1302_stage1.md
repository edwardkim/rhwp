# Stage 1 완료보고서 — Task #1302 근본 원인 확정

- **이슈**: #1302 / 브랜치 `local/task1302` (base `stream/devel` 9d3aa212)
- **목표**: pi=852(다줄 미주)→pi=853(같은 문제 연속) 줄간격 과소의 근본 원인 특정 (코드 무수정)

## 증상 재현 (stream/devel)

`rhwp export-svg samples/3-11월_실전_통합_2022.hwpx -p 17` 좌측 단 baseline(px):

| 줄 | baseline | gap |
|----|---------|-----|
| 는에서극댓값을갖고,에서 (분수, 키 큼) | 1023 | — |
| 극솟값을갖는다 (pi=852 끝줄) | 1050 | +27 |
| **(나)를고려하기위해… (pi=853)** | **1062** | **+12 ★** |

PDF(한글2022, ×1.334 보정): 극솟값→(나)를 기대 ≈18px. 현재 12px → 과소.

## 계측 (RHWP_VPOS_DEBUG / 임시 DRAW 계측, 직후 원복)

```
DRAW pi=852 line1 draw_y=1037.7 seg_vpos=953344 en_base=Some((950822,1004.12))  # 다줄 로컬앵커
DRAW pi=853 line0 draw_y=1051.7 seg_vpos=954846 en_base=None                      # 단일줄(col0 partial)→앵커 없음

VPOS_CORR pi=853 prev_pi=852 path=page
  prev_vpos=953344 prev_lh=1050 prev_ls=452  vpos_end(curr_first)=954846
  y_in=1057.77  end_y=1037.35  result=1051.75  page_tail=true
```

## 근본 원인

1. pi=852 는 다줄(end>start+1) → `endnote_line_vpos_base` 로컬앵커(base_y=incoming) 사용.
   pi=853 의 컬럼0 부분은 **단일 줄(partial, end=1)** → 앵커 비활성, cursor advance 로 배치.
2. `paragraph_layout` 은 #1236 게이트로 pi=852 마지막 줄에 trailing(452HU) 포함 →
   반환 y=1057.77 (정답: stored gap 1502HU = lh+ls 와 정합).
3. 그러나 `height_cursor.rs::vpos_adjust` 의 **`compact_endnote_page_tail_backtrack`**
   (조건 L400-407, 결과 L507-511)이 컬럼 하단(>95%)에서 발동:
   `end_y.max(prev_content_bottom_y).min(y_offset)` = prev_content_bottom_y(1051.74)
   → **trailing 6px 제거**. `end_y`(page_base 절대매핑)가 다줄 로컬앵커 대비 ~20px drift 해
   `end_y < y_offset-8` 이 성립한 탓.
4. **모순**: stored gap = 954846−953344 = **1502HU = prev_lh+prev_ls** = 정상 한 줄 전진.
   stored vpos 가 정상 연속을 인코딩하는데 backtrack 이 ls 를 깎음 → overlap 보정이
   정상 연속에 오발동.
5. 기존 #1246 rescue(L596-603)는 **다음이 "문" 제목일 때만** trailing 복원 → 같은 문제
   **연속(비제목)+컬럼 하단** 미커버. #1236(중간 컬럼)도 미해당. = #1302 신규 케이스.

## 비회귀 확인

- #1300 회귀 아님 (직전 커밋 03b04dd3 재현).
- #1236/#1246 수정 코드는 stream/devel 에 이미 존재(병합 누락 아님) — 본 건은 그 게이트들이
  커버하지 못하는 별개 경계 케이스.

## 결론

수정 지점: `vpos_adjust` 의 `compact_endnote_page_tail_backtrack` 조건에
"curr 첫 줄 stored gap 이 정상 한 줄 전진(lh+ls) 이상이면 비발동" 게이트 추가 (구현계획서 §2).
