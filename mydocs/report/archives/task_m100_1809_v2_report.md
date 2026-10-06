# Task #1809 v2 최종 보고 — 잔여 admrul_0556/0072 한글 판정·게이트 소스 무관화

## 판정 (조사 9차)

Windows + 한글 2022(pyhwpx)로 원본 HWPX 의 편집기 PDF(권위)를 생성, rhwp A(HWPX 직파스)/B(HWP5 재파스) PDF 와 내용 앵커 기준 기하 3자 대조 (PyMuPDF 텍스트 baseline + 표 경계선 y):

| 케이스 | 측정 앵커 | 한글 | A (현행) | B (재파스) | 정답 |
|--------|----------|------|----|----|------|
| admrul_0556 p1 | RowBreak 조각 하단 컷 y | 808.8pt | **808.7** (pad ON) | 810.1 (pad 없음) | **pad 적용** |
| admrul_0072 p4 | 서명 셀 '성명' 줄→하단 경계 | 25.5pt | 13.9 (extra 없음) | **25.9** (extra ON) | **extra 적용** |

두 게이트(c7dbe8a2 도입)의 정답 방향이 **서로 반대** — 공통 결론은 두 보정 모두 소스 무관 기하라는 것. 각각 반대편 파스 경로에서 한글과 어긋나 있었다.

## 수정 (`src/renderer/layout/table_layout.rs` 2곳)

1. `hwpx_rowbreak_top_pad`: `is_hwpx_source` 조건 제거 → `is_block_rowbreak && !has_internal_line_reset` 만으로 적용
   - 합성 seg 태그(#1811 TAG_IMPLEMENTATION_PROPERTY) 기반 증거 판별도 시도 — B 의 seg 는 HWPX linesegarray 유래 원본 태그라 판별 불가, 소스 무관화가 정공 (시도·기각 기록)
2. `mixed_nested_flow_extra_from_cut`: `is_hwpx_source` 조기 0 반환 제거

## 검증

- **대상 케이스**: admrul_0556 OVER 1.88 → **PASS 0.00** / admrul_0072 OVER 16.00 → **PASS 0.00**
- **한글 절대 기하 재확인**: 0556 B 컷 위치 808.7 = 한글 808.8 / 0072 A 서명 셀 gap 25.9pt = 한글 25.5pt
- **cargo test --release 전수 (통합 스택 + 본 수정)**: 80개 타깃 전부 통과 (svg_snapshot 골든 포함 — 네이티브 시각 회귀 검출 축)
- **big_hwpx 2,500 배치**: PASS 2470 / OVER 6 / STRUCT 20 / PAGE 4 (직전 기록 2455/8 대비 순개선). 파일 단위 rd_big_hwpx 기준선 대비 개선 103 / 회귀 2
- **회귀 2건(seoul_0776/1006) 귀속 이등분 완료**: 게이트 수정 **미포함** 빌드들에서 동일 재현 — **본 수정 무관**.
  - seoul_1006: devel PASS / devel+PR#1826(#1811) 단독 재현 → **PR #1826 기인** (PR 코멘트로 보고)
  - seoul_0776: **devel 자체에서 재현** (기병합 커밋 기인, 과거 devel 스냅샷 TSV 는 PASS) → 신규 이슈 **#1836** 등록
- **big_hwp 2,500 네이티브 배치**: PASS 2494 / OVER 4 / STRUCT 2, rd_big_hwp 기준선 대비 **상태 회귀 0**. OVER 내 변위 변화 2건(0593 개선 9→6.7 / 0646 증가 152→247)은 게이트 수정 미포함 빌드에서도 동일 — 본 수정 무관
- **devel + 본 수정 단독 (PR 브랜치)**: admrul_0556/0072 모두 PASS 0.00, cargo test --release --lib 2067 통과

## 잔여

- row_span=1 측정의 aim=true && pad 0 정합 (#493 시멘틱 충돌) — 별도 규칙 설계 (v1 보고서부터 이월)
- 회귀 2건은 본 태스크 밖(스택 기인) — 귀속 확정 후 해당 이슈/PR 에 보고
