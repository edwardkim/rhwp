---
kind: investigation
status: historical
canonical: mydocs/tech/investigations/issue-1772/README.md
last_verified: 2026-07-16
---

# Issue #1772 잔여 OVER 28건 조사 보고서 — 유형 분류 + 본문 줄 군집 근본 원인

## 요약

#1772(outMargin 동기화, PR #1784) + #1785(셀 안 여백 규칙, PR #1788) 적용 후 hwpdocs
코퍼스 2,500건에 남은 OVER 28건을 전수 분류하고, 최대 변위 군집(본문 줄 이동, 최대
345px)의 근본 원인을 코드 수준으로 확정했다.

## 유형 분류 (28건, render-diff 최대 변위 노드 서명 기준)

| 유형 | 건수 | 변위 범위 | 대표 | 상태 |
|------|------|----------|------|------|
| A. 본문 줄 세로 이동 (exclusion) | 3+α | 104~345px | seoul_0377 | **근본 원인 확정** |
| B. 셀 내 TextRun 가로 이동/분할 | 13 | 2~150px | seoul_0978 | 1차 조사 (아래) |
| C. 본문 TextRun 이동 | 7 | 2~42px | seoul_0606 | 미조사 (B 인접 추정) |
| D. 셀/표 기하 | 4 | 1.9~95px | seoul_0765 | 미조사 |
| E. TextBox 내부 | 1 | 101px | seoul_0043 | 미조사 |

전체 목록: 조사 시 산출 `/tmp/over28_sigs.txt` → 본 문서 부록 참조 필요 시 재생성
(`rd_big_hwpx_1785b/geom_inventory.tsv` 의 OVER 행).

## A. 본문 줄 세로 이동 — 근본 원인 (확정)

재현: 동작소방서 36385142 (seoul_0377), 345px.

1. 문단 기준(vert=Para) 자리차지 표(603×309px)가 exclusion zone 을 만든다.
2. 후속 문단 0.8 의 줄: 저장 lineseg lh=1200, th=1200(잉크 16px), ls=720(9.6px).
   잉크 하단(545.9px)은 zone top(552.7px) **위** — 한컴 저장 vpos(34925→529.9px)도
   이 줄을 표 위에 유지 (HWP5 재파스 렌더와 일치).
3. `src/renderer/layout.rs` exclusion consult 의 `overlaps_zone` 프로브(≈L4180-4224,
   **is_hwpx_source 게이트**)가 `line_height + line_spacing`(25.6px)으로 겹침 판정 →
   line_spacing 포함분(~2.8px)만 겹치는데도 문단을 zone.bottom(875px)으로 밀어냄.
4. HWP5 경로는 이 프로브 자체가 없어(게이트) 한컴 위치 유지 → 파스 경로 비대칭.

**검증 실험** (RHWP_1772_PROBE_NO_LS, 원복 완료): 프로브에서 line_spacing 제외 →
- seoul_0377 345→0.00 / seoul_0030 111→0.00 / seoul_0973 104→0.00 (완전 해소)
- 코퍼스 300건 변화 없음 / 2,500건 개선 3·악화 0

**수정 방향 제안**: 프로브를 `line_height`(또는 잉크 `text_height`) 기준으로 축소.
단, 이 프로브가 도입된 원 과제의 한컴 정합 케이스(A-side vs 한컴, 라운드트립으로는
안 보임)가 line_spacing 포함에 의존할 수 있으므로 **전체 cargo test(golden 포함)로
도입 목적 회귀를 반드시 확인** 후 적용.

## B. 셀 내 TextRun 가로 이동 — 1차 조사

재현: seoul_0978, 150px (Cell/TextLine0/TextRun2-3).
- 원본(A)에는 금액 한글표기 run("(구백칠십육백사십사만…")이 있고 재파스(B)는 해당
  위치 run 이 빈 문자열 + x 가 13px 이동 — **run 분할/내용 자체가 다름**.
- 셀 필드(누름틀) 텍스트의 HWP5 직렬화 소실 계열로 추정 (#1772 조사에서 확인한
  cell field_name 소실, 문단 0.5 cc 58→50 제어문자 소실과 동족).
- 후속: 직렬화 시 필드 컨트롤/누름틀 텍스트 보존 여부 조사 → 별도 이슈 분리 권장.

## 후속 조사 결과 (2차 — 순서대로 자동진행)

- **A 유형**: #1789 수정 완료 (PR #1791). 재현 3건 0.00px, 코퍼스 2,500건 개선 3·악화 0.
- **B/C 유형 (20건) 심층**: 시각 동등성 판정(줄 단위 연결 텍스트+좌표) 결과 —
  - 시각 완전 동일 5건 (seoul_0978/0367/1069/0606/0624): 직렬화 필드·제어문자 소실로
    run 경계만 변한 것을 render-diff LCS 가 변위로 오측정한 **게이트 오탐**.
  - 특수문자 렌더 손상 다수: '-'(U+002D)→NUL, 반대로 NBSP(U+00A0)→'-' (seoul_0505).
    IR 텍스트(dump)·ir-diff 는 무결, export-text/render 에서만 손상 → 컴포즈 단계의
    HWP5 특수문자 코드(하이픈 24/묶음빈칸 30 등) ↔ 유니코드 변환 비대칭 추정.
    → **Issue #1793** 등록.
  - 기타: seoul_1118 (22.7px 세로 이동), seoul_1225 (공백 폭 4.5px) — 별개 소결함.
  - 참고: 초기 bold 단서(dump [CS] attr 파생)는 허위 — 렌더러는 cs.bold 필드를 쓰므로
    양 경로 동일. dump 의 bold 표기는 attr 비트 파생이라 HWPX 파스에서 항상 false
    (파서가 bold 필드만 채우고 attr 비트 미기록 — 표기 개선 여지).
- **D 유형**: seoul_0765 — 중첩 1×1 쪽 기준 표 전체가 95.16px 이동 (63셀 동반).
  → **Issue #1794** 등록. 소수 px 3건은 동족 가능성.
- **E 유형**: seoul_0043 — 글상자 내부 줄바꿈 지점 차이 (텍스트 폭 계측 비대칭).
  → **Issue #1795** 등록.

## 진행 이력

- 조사 브랜치: local/task1785 (PR #1788 스택 위, 코드 무수정 — 실험 원복 완료)
- 산출물: output/poc/hwpdocs_renderdiff/rd_big_hwpx_probe/ (실험 inventory),
  rt377a/rt377b (render tree 대조)
