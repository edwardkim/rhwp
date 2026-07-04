# Task M100 #1789 최종 결과보고서 — exclusion 겹침 프로브에서 line_spacing 제외

## 요약

#1772 잔여 A 유형(본문 줄 세로 이동, 최대 345px)의 원인인 exclusion 겹침 프로브의
line_spacing 포함 과대 판정을 수정했다. 프로브 도입 원 과제(task 1510)의 테스트는
전부 보존된다.

## 변경 사항

- `src/renderer/layout.rs` `overlaps_zone` 프로브 2개 사이트: `line_height +
  line_spacing` → `line_height` (text_height 우선 규칙 유지)
- `tests/issue_1789_exclusion_probe_line_spacing.rs` + `samples/task1789/` (36385142)

## 검증 결과 (전체 PR 통합 브랜치 test/all-prs-v3 기준)

| 항목 | 결과 |
|------|------|
| 재현 3건 (seoul_0377/0030/0973: 345/111/104px) | 모두 **0.00px PASS** |
| task 1510 원 과제 테스트 (issue_1510, 4건) | 통과 (프로브 도입 목적 보존) |
| 신규 회귀 테스트 (저장 vpos 529.9px 유지) | 통과 |
| 스택 테스트 (1749/1750/1772/1785/493) | 통과 |
| 코퍼스 300건 | 변화 없음 (해당 유형 없음) |
| 코퍼스 2,500건 | **개선 3 (OVER 28→25), 악화 0** |
| 전체 `cargo test` (149 바이너리) | 통과, 실패 1건은 기존 #1775 (Windows CFB 구분자) |
| clippy (`--release --lib`) | 경고 없음 |

## 근거 (한컴 정합)

36385142 문단 0.8: 저장 lineseg vpos=34925 → 529.9px (표 위 유지) = HWP5 재파스 렌더와
일치. 잉크 하단 545.9px < zone top 552.7px — 겹침은 line_spacing 포함분 2.8px 뿐.

## 잔여

- B 유형(셀 내 TextRun 가로 13건 — 직렬화 필드 텍스트 소실 의심), C/D/E 유형은 후속
  조사 (순서대로 진행 중).
