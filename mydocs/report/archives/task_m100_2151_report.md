# 최종 보고서 — Task M100 #2151: HWP3 그림 pgy=0 페이지 시작 후 거짓 쪽 경계 수정

- 이슈: #2151 / 브랜치: `fix/2151-hwp3-pagination` / 작성일: 2026-07-10
- 계획서: `mydocs/plans/task_m100_2151.md`

## 원인 (계측 확정)

HWP3 저장 `line_info.pgy`(한글97 계산 줄 Y, 감소 = 새 페이지 신호) 처리에서:

1. 그림 호스트 문단은 pgy=0으로 저장 — `first_pgy(0) < prev_last_pgy(15441)`로
   쪽 경계 승격 (정당)
2. `prev_last_pgy` 갱신이 `last_pgy > 0` 조건부라 그림 문단(pgy=0)이 **이전
   페이지 기준값을 유지**
3. 다음 문단의 정상 새-페이지 pgy(sample14 pi17=3521, HWP5 변환본 vpos 14084/4와
   정확 일치)가 잔존 기준(15441)보다 작아 **거짓 쪽 경계 재승격** → 그림만 있는
   유령 페이지 (rhwp 12쪽 vs 한글 11쪽)

## 수정 (1줄 + 주석, `src/parser/hwp3/mod.rs`)

`prev_last_pgy` 갱신 조건 `last_pgy > 0` → `last_pgy > 0 || is_page_break`.
새 페이지를 시작한 문단은 pgy=0이어도 기준을 리셋한다. HWP3 전용 로직으로
`src/parser/hwp3/` 밖 무접촉 (CLAUDE.md 규칙 준수).

## 검증

### 한글 2022 COM per-pi 오라클 (HWP3 원본 10종 전수 재검)

| 파일 | 수정 전 (#2154 스윕) | 수정 후 |
|---|---|---|
| hwp3-sample14 | PAGE_DELTA 12vs11, n_mm 239 | **MATCH 11=11** |
| hwp3-sample11 | PAGE_DELTA 152vs151, n_mm 3523 | **MATCH 151=151** |
| hwp3-sample10 | PI_MISMATCH 92 (763=763) | PI_MISMATCH 92 — 잔존 서브축(p39부터 산발), 이슈에 기록 |
| hwp3-sample16 | CARET 1 | CARET 1 (오탐 후보, 불변) |
| sample/4/5/13/19/pagedef-1915 | MATCH | MATCH (무회귀) |

- 산출: `output/poc/task_pipage_sweep/hwp3_retest.tsv`

### 게이트

- `cargo test --release` 전체: 실패 0 (변경 후 전량 재실행)
- `cargo clippy --release`: 0
- 신규 핀 테스트: `tests/issue_2151_hwp3_ghost_page.rs`
  (sample14=11쪽, sample11=151쪽 — repo 추적 실문서 fixture, 한글 오라클 +
  HWP5/HWPX 변환본 3자 정합 권위)

## 잔존 (이슈 유지 사유)

- hwp3-sample10.hwp n_mm 92 (쪽수 동일, p39부터 산발 밀림) — pgy 기준 축이 아닌
  별도 서브시그니처. caret 혼입 다수로 시각 대조 필요.
- hwp3-sample16-hwp5.hwpx −1쪽 / sample5-hwp5 계열 16mm — **HWP5/HWPX 변환본**
  전용 축(HWP3 파서 무관), #2151 본문 표 참조.
