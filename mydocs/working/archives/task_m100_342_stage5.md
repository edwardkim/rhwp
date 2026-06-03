# Task #342 단계 5 — 메인테이너 검토 응답 + 광범위 시각 검증

> 메인테이너 PR #343 리뷰 4 개 질문 (Q1~Q4) 응답 및 통합 검증 결과.

## Q1 — KTX 목차 페이지번호 +17.01px 회귀 (해소됨)

### 진단

`samples/KTX.hwp` pi=12 (5×3 표) 셀[10] r=4,c=0,cs=3 의 padding=
(141, **1417**, 141, 141) — Right 만 10× 비대칭, aim=false. 작성자가 페이지
번호용 우측 여백 의도적으로 설정.

Task #342 의 `resolve_cell_padding` aim=false → table padding 해석이
cell.padding R=1417 을 무시하고 table.padding R=141 사용 → inner_right
가 17.01px 우측으로 확장 → leader 시멘틱 (\"inner content 우측 끝까지\")
이 페이지번호도 동반 우측 이동.

### 한컴 호환성 결정

Task #279 의 troubleshooting (`toc_leader_right_tab_alignment.md`):
> HWP 스펙은 데이터 포맷 정의일 뿐이고, **한컴 조판 알고리즘은 비공개**.
> 한컴은 의도를 재해석. KTX 목차 시각 검증 (작업지시자) ✅ 한컴과 동등.

→ Task #279 시점 한컴 PDF 와 동등 검증된 결과 = 페이지번호 right edge ≈ 700.
→ **한컴은 aim 무관하게 cell.padding 명시값 적용** (스펙과 다른 동작).

### 수정 (커밋 `782a5a7`)

- `table_layout.rs::resolve_cell_padding`: aim=false 분기 제거. cell.padding
  명시값(0 아님) 우선 + table.padding fallback 단일 흐름.
- `height_measurer.rs` (2 곳): 동일 정책으로 통일하여 layout 과 일관성 확보.

### 검증

- `tests/golden_svg/issue-267/ktx-toc-page.svg` 갱신 — 페이지번호 x=707.77
  → **690.76 복원**.
- 갱신된 골든의 md5 가 Task #279 시점 (`7322447`) 의 골든 md5 와 **정확히
  일치** → Task #279 결과 그대로 회복.

```
$ md5sum tests/golden_svg/issue-267/ktx-toc-page.svg
d6860727ad76dbf3bc7d924306c2eee1   (현재 커밋)
d6860727ad76dbf3bc7d924306c2eee1   (Task #279 시점)
```

## Q2 — issue-157 page-1 −9.6px shift

### 진단

stage3a/3b revert 의 trailing line_spacing 720 HU = 9.6px 효과. cell-clip
y 좌표 모두 −9.6px shift.

### PDF 비교 한계

`samples/hwpx/issue_157.hwpx` 에 대응하는 한컴 PDF 가 저장소에 없어
직접 비교 불가.

### 대안 검증 — HWP 스펙 일치성

Task #342 의 stage3b revert 커밋 (`4d74550`) 명시:
> stage3a: vpos_end 의 trailing line_spacing 제외 → 다음 문단 시작이 trail_ls
> (716 HU=9.55px) 만큼 앞당겨져 줄간격 압축. **HWP 가 명시한 다음 문단
> vpos = prev.vpos + prev.lh + prev.ls 와 불일치**.

→ 현재 (revert 후) 동작이 HWP 의 line_seg vpos 자료 + 다음 문단 vpos 공식
양쪽과 일치. stage3a (이전) 는 trailing ls 를 임의로 trim 하여 spec 위반.

### 결정

**현재 −9.6px shift 가 HWP 데이터에 충실한 방향**. 한컴 PDF 직접 비교가
필요하면 사용자가 기준 PDF 를 제공해야 최종 확정 가능. 현 시점에서는
HWP 자체 line_seg vpos 일관성을 근거로 \"correct\" 로 판정.

## Q3 — 7 핵심 샘플 + form-002 시각 점검

### 점검 결과

| 샘플 | PDF | 시각 점검 결과 |
|------|-----|---------------|
| `21_언어_기출_편집가능본.hwp` | ✅ | 1·2 페이지: PDF 와 컬럼·헤더·문단 구조 일치. Task #342 의 3 영역 유지 |
| `exam_math.hwp` | ✅ | p1 우측 컬럼: 문항 4 의 \"가 실수 전체...\" 텍스트 위치가 PDF 와 차이. 좌측 문항 2 위치도 다름 — **stage3a/3b revert 와 무관, 이전 PR 이 누적시킨 vpos 회귀**. 별도 이슈 가능성 |
| `exam_kor.hwp` | ❌ | 2 컬럼 + 시험지 헤더 정상. 문항 번호·박스 배치 자연 |
| `exam_eng.hwp` | ❌ | 2 컬럼 + 표·이미지 배치 자연. listening 문항 헤더 정상 |
| `KTX.hwp` p2 (TOC) | ⚠ Q1 한정 | 페이지번호 right edge ≈ 700 복원 — Task #279 결과 동일 |
| `aift.hwp` | ❌ | p1 경고 박스: 박스 내용 + 박스 외 주석 위치 정상. issue-147 영역 회귀 없음 |
| `biz_plan.hwp` | ❌ | p1 표지: 제목·날짜·회사명 중앙 정렬 정상 |
| `form-002.hwpx` | ✅ | p1 폼 표: PDF 와 셀 구조·체크박스·텍스트 위치 일치. **Task #324 영역 회귀 없음** |

### 발견된 잔존 의심 사항

**`exam_math.hwp` p1**: 좌측 단의 문항 2 (\"함수 f(x)=x³-8x+7\") 가 PDF 에서는
중·상단에 있는데 우리 SVG 는 하단으로 밀림. 우측 단도 문항 4 답지가 누락된
듯한 시각 차이. 이는 본 PR 의 stage3a/3b revert 효과가 아니라 **Task #297
(VertRelTo::Page/Paper 분리) 또는 더 이전 회귀** 가능성. 작업지시자가
보고한 \"p2 우측 pi=66 베이스라인\" 영역은 동일 페이지 다른 좌표.

→ **본 PR 범위 밖 별도 이슈로 분리 권고**.

## Q4 — 통합 PR 검증 범위 요약

| 시각 검증 | 결과 |
|-----------|------|
| `21_언어 p1 좌측` (Task #342 트리거 1) | ✅ 줄 간격 균일 회복 |
| `21_언어 p2 좌측` (Task #342 트리거 2) | ✅ 셀 마지막 줄 클립 안 잘림 |
| `21_언어 p2 우측` (border wrap) | ✅ inner edge 안 그어짐 |
| `exam_math p2 pi=66` (Task #342 트리거 3) | ✅ 베이스라인 회복 |
| `KTX 목차 페이지번호` (Q1 회귀) | ✅ Task #279 결과 복원 |
| `issue-157 page-1 cell-clip` (Q2) | ⚠ HWP 스펙 일치, PDF 비교 미실시 |
| `form-002 p1` (Task #324 영향 점검) | ✅ 회귀 없음 |
| `aift / biz_plan / exam_kor / exam_eng` p1 | ✅ 명백한 이상 없음 |
| `exam_math p1` | ⚠ 별도 이슈 (Task #342 와 무관 가능성) |

### 잔존 작업

1. issue-157 PDF 가 확보되면 Q2 의 −9.6px 가 한컴과 정확히 일치하는지 재
   검증.
2. exam_math p1 의 layout drift 원인 분석은 별도 이슈로 분리.

## 다음 단계

본 보고서로 stage 5 (메인테이너 검토 응답 + 통합 검증) 마무리.
PR #343 머지 가부는 작업지시자 + 메인테이너 결정.
