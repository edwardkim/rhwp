# PR #2309 검토 — 거대 rowspan 표 정밀도 3건 (#2291/#2237)

- PR: https://github.com/edwardkim/rhwp/pull/2309 (planet6897, 누적 merge 50+)
- 이슈: #2291 (거대 셀 컷 정합), #2237 (선언>콘텐츠 슬랙), 모 이슈 #2287
- base=devel, 6커밋, MERGEABLE/BEHIND → merged tree 로컬 선검증
- 스택 관계: planet6897 열린 6건 중 첫 건 (독립 — #2315→#2321→#2323 스택과 별개)

## 변경 본질 (4 + 동봉 2)

1. **셀 저장-ls-1 폭 초과 재래핑** (`recompose_stored_single_line_if_overflowing`):
   기계생성 문서의 "다줄 문단에 저장 lineseg 1개" 관례에서 저장 신뢰 가드가
   텍스트를 절단 → 저장 ls==1 + 실폭 > 내폭×1.05 시 fresh 재분할.
   가로쓰기 셀 한정(세로쓰기 회귀 실측 배제), 호출부 2곳(table_layout/partial).
2. **rowspan 병합 셀 선언-잔여의 마지막 걸침 행 가산** (resolve + mt 동일 규칙):
   한글 행 괘선 실측(r183: c3 rs=4 선언 217.8 vs 행합 201.3, 한글 = 39.8+16.5
   =56.3 정확 일치)으로 확정한 관례. 종전 rhwp 는 이 잔여가 소실되어
   rowspan 중첩 문서가 쪽당 +15% 조밀.
3. **rowspan 블록 오프셋 walk 에 relaxed hard-break 이식** (거대 셀 한정):
   plain 블록 walk 와 동일 의미론 — RowBreak + (col≤2 or row>5) + 비-TAC
   흐름 + 유닛≥24 셀 한정, 소형 셀 보호 가드(21217935 8→9 회귀 실측 배제).
4. **행 앵커 대조 도구** (`tools/task2287/row_anchor_map.py`) — 조사 도구.
- 동봉: 한글 2022 정답지 PDF (Hancom PDF producer, **415쪽 확인**,
  `pdf/task2287/` — 명명·배치 규약 적합, 2.6MB) + 회귀 oracle 테스트 2건.

## 구조 검토

- 두 정정 모두 케이스 구조 가드(저장 필드·문서 속성 기반)로 한정 — 샘플명/
  페이지 분기 없음. 3번의 임계(col≤2∥row>5, 유닛≥24)는 휴리스틱이지만 문서
  속성 기반 + 반례 실측(21217935) 가드 동반 — 하드코딩 금지 원칙과 양립.
  후속 문서 유형에서 임계 재조정 가능성은 관찰로 기록.
- 선언-잔여 규칙이 resolve(table_layout)와 mt(height_measurer) **두 경로에
  동일 적용** — 두-경로 정정 패턴 준수.

## 로컬 재실증 (merged tree = devel a3b63932 + head)

| 게이트 | 결과 |
|--------|------|
| PR oracle (issue_2291_rowspan_declared_residual) | **2/2** |
| `cargo test --tests` release-test | **실패 0** (knife-edge 핀·task81 세로쓰기·issue_1891 포함) |
| fmt --all / clippy | 통과 / 0 |
| 연결맵 페이지 수 | devel **380 → 385** (주장 일치, 한글 415 방향 +5) |
| CI (head) | 전 항목 pass |

## 판단

**merge 권고.** 실측(행 괘선 PDF 선분 파싱)으로 한글 관례를 확정하고 두 경로
동시 정정 + 반례 가드 + 정답지·oracle 동봉까지 자기완결적. 잔여 −30쪽(s5
rowspan 걸침 행 통째 배치)은 #2291 다음 관문으로 이슈 추적 명시.
