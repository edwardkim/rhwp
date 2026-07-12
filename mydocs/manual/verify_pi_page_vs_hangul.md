# 페이지·PI 매칭 오라클 가이드 (`verify_pi_page_vs_hangul.py`)

rhwp 렌더링 레이아웃(페이지네이션)의 문단(PI)별 배치 쪽을 한글(OLE 자동화) 캐럿 쪽과
1:1 대조한다. `hangul_page_oracle.md`(총 페이지수 대조, #1560)보다 한 단계 정밀한
로컬 오라클(#1643 도입).

## 사용
```bash
python tools/verify_pi_page_vs_hangul.py --batch <원본폴더> [--sample N] [--seed S] -o out.tsv
python tools/verify_pi_page_vs_hangul.py --files a.hwpx b.hwp -o out.tsv
```
- rhwp: `dump-pages` 의 pi 첫 등장 쪽(1-기반). 한글: `SetPos(0, para, 0)` 후 `current_page`.
- 판정: MATCH / PI_MISMATCH / PAGE_DELTA / PARA_COUNT / ERR. mismatch 1건↑ 종료코드 1.
- 요구: Windows + 한컴오피스 + pyhwpx + rhwp release 바이너리.

## 알려진 한계 — 캐럿-개체 분리 오탐 (#1757)
rhwp 는 표 **몸체**가 놓인 쪽, 한글은 **캐럿**(문단 시작) 쪽을 보고하므로, 아래 두 유형은
시각 정합인데도 PI_MISMATCH 로 나온다 (hwpdocs 조사에서 한글 PDF 시각 판정으로 확정):

| 유형 | 증상 | 확정 사례 |
|------|------|----------|
| 자리차지 다쪽 표 anchor | 한글 캐럿(anchor 줄)이 표 끝 쪽 → rhwp(표 시작 쪽)와 큰 차이 | 17991519 공항시설법 별표3 (pi1 rhwp 1쪽 ↔ 한글 4쪽) |
| 쪽 경계 TAC 표 문단 | 표는 양쪽 모두 다음 쪽 통째 렌더, 한글 캐럿은 이전 쪽 | 2789777 군수품 별표3 pi4, 36389863 물품검사 조서 pi9 |

3. **[#1920] 쪽 하단 빈 문단** — rhwp 는 저장 lineseg 대로 쪽 하단에 배치하는데 한글
   캐럿은 다음 쪽 상단으로 보고한다 (예: 36398160 pi3, rhwp 1쪽 vs 한글 2쪽 — 해당 PI 는
   controls=0 빈 문단, stored vpos=66269 = 쪽 하단, rhwp 배치가 저장 lineseg 와 일치).
   도구가 자동 분리한다: 불일치 PI 전부가 빈 문단(text_len=0, controls=0)이고 rhwp 쪽 =
   한글 쪽 - 1 이면 verdict PI_MISMATCH_CARET + detail [empty-caret?] 태그 (종료코드
   실패 미계상 — 시각 확정 전 후보 등급).

**오탐 판별 절차**: mismatch pi 문단을 `rhwp dump`(컨트롤/wrap 확인) → 다쪽 자리차지 또는
쪽 경계 TAC 표이면 한글 PDF 생성(pyhwpx `save_as(..., format="PDF")`) 후 해당 쪽을 시각
대조 — 표 몸체 배치가 같으면 오탐(도구 한계)으로 기록하고 결함 조사 대상에서 제외한다.

## 관련
- 총 페이지수 오라클: `mydocs/manual/hangul_page_oracle.md` (#1560)
- 개체 단위 오라클: `mydocs/manual/object_visual_regression.md` (#1720)
- 배경 조사: hwpdocs 페이지·PI 대조 (#1745/#1749/#1750/#1753/#1755 수정 체인)
