---
kind: guide
status: active
canonical: mydocs/manual/verification/edit_sweep.md
last_verified: 2026-07-19
---

# 편집-스윕 하니스 (`examples/edit_sweep.rs`)

편집 경로 PR(vpos 재계산·pagination·undo 계열)의 **가짜 페이지 변동**을 정량 검출한다.
PR #2314 검증에서 일회성 하네스로 수행해 "devel 변동 70건 → PR 11건"을 실증했던
편집-스윕(#2355)의 상설판.

## 무엇을 하나

1. **스윕** — 대상 디렉터리 재귀의 `.hwp`/`.hwpx`/`.hml`(<2MB 캡, `--max-kb`)을
   읽기순으로 로드 → 편집 적용(기본 `insert1` = para0 앞 1자 삽입, #2314 와 동일) →
   전후 `page_count()` 를 TSV 로 기록. 파일 단위 panic 은 `catch_unwind` 로 격리해
   스윕 전체를 죽이지 않고 `status` 열에 남긴다(파싱/편집 오류 동일).
2. **대조**(`--compare A.tsv B.tsv`) — 두 스윕 결과를 파일 단위로 짝지어
   **공통 변동(기존 동작) / A 에서만(B 가 해소) / B 에서만(신규) / 상태 전이** 로
   분류한 markdown 리포트 출력. **신규 변동 존재 시 종료코드 1** — 게이트 사용.

## 사용

```bash
# devel 에서 baseline 스윕
git switch devel && cargo run --release --example edit_sweep -- samples -o out/sweep/devel.tsv

# 작업 브랜치에서 스윕
git switch <branch> && cargo run --release --example edit_sweep -- samples -o out/sweep/branch.tsv

# 대조 → 공통/해소/신규 분류 리포트 (PR 본문 첨부용)
cargo run --release --example edit_sweep -- --compare out/sweep/devel.tsv out/sweep/branch.tsv -o out/sweep/report.md
```

실측: 581 샘플 전수 스윕 약 12초 (release, M-계열 macOS). 편집 종류는
`--edit` 로 선택하며 현재 `insert1` 하나 — `EDIT_KINDS`/`apply_edit` 에
추가하는 확장 구조(문단 삽입·붙여넣기 등 후속).

## 판정 규약

- **B 에서만 변동(신규)** 이 회귀 후보다. 단, #2314 treatise 사례처럼 "기존
  동작 유지"가 오히려 버그(간격 소실)일 수 있다 — 신규 변동은 좌표 실측으로
  정당 성장 여부를 가려야 하며, 자동 게이트는 후보 검출까지만 담당한다.
- 상태 전이(ok↔parse/edit/panic)는 커버리지 변화로 별도 분류 — 무변동
  위장을 막는다.
- baseline TSV 는 커밋하지 않는다(출력은 `out/` 아래).

## 한계

- 페이지 수만 대조한다 — 쪽수 불변의 페이지 내 배치 변화는 개체 시각
  회귀([object_visual_regression.md](object_visual_regression.md))가 커버.
- 편집 지점이 para0 고정이라 문서 후반부 전용 경로는 미자극 — 편집 종류
  확장으로 보완 예정.
