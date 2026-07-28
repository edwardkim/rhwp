# #3507 Stage 2 — 전체 회귀 검증과 IR baseline 갱신

- Issue: #3507
- 브랜치: `codex/issue-3507-sectiondef-ctrl-data`
- 최종 기준: `upstream/devel` `f32d964dfc5bdb1a28f41f9277e4aa2d7e4387ff`
- 착수 기준: `upstream/devel` `ef72fee5138e3c491b6d8f38e459dc94670284f6`
- 수행일: 2026-07-28

## 전체 IR sweep 판정

첫 전체 sweep은 SectionDef 중복 제거에 따른 개선 70개 baseline key와 함께 다음 7개 key의
증가를 보고했다.

- `hwp3-sample10-hwp5.hwp`: `char_offsets[]` 1749 → 1751
- `hwp3-sample16-hwp5*.hwp` 5종: `list_header_width_ref` 각각 +2
- `issue2063_huge_cellbreak_table.hwp`: `list_header_width_ref` 1998 → 2000

baseline을 바로 덮어쓰지 않고 변경 전 `upstream/devel`을 LFS smudge 없이 별도 복제해 같은
전체 sweep을 실행했다. 변경 전 dump는 기존 baseline과 byte-for-byte 같았고 671행,
총 발산 110,345건으로 통과했다.

7개 증가는 신규 필드 손실이 아니라 문서당 진단 상한 `MAX_DIVERGENCES=2000`의 슬롯 이동이다.
변경 전에는 각 샘플의 SectionDef 중복 길이 발산 2건이 먼저 상한을 차지했다. 중복을 제거하자
그 뒤에 이미 존재하던 발산 2건이 집계됐다. 7개 샘플 모두 총계가 정확히 `2000 → 2000`이다.

나머지 비포화 샘플 28개는 모두 정확히 2건씩 감소했다. 따라서 최종 baseline 변화는 다음과 같다.

| 항목 | 변경 전 | 변경 후 | 판정 |
|---|---:|---:|---|
| 전체 dump 행 | 671 | 601 | 개선 |
| 전체 발산 수 | 110,345 | 110,289 | 56건 감소 |
| 제거된 SectionDef 길이 key | 70 | 0 | 35개 샘플 × 2개 경로 |
| 상한 포화 샘플 | 7 | 7 | 모두 총 2,000건 유지 |

최종 dump만 `tests/fixtures/ir_field_sweep_baseline.tsv`에 반영했다. 전체 sweep 재실행 결과는
803개 샘플, 스킵 3개, 601개 발산 경로, 총 110,289건이며 신규 회귀 없이 통과했다.

## control 경계 보강

전체 sweep 원인 분리 과정에서 기존 일반 control 계약도 다시 고정했다. 문단 control 슬롯의
`CTRL_DATA` 검색은 첫 중첩 `CTRL_HEADER` 전까지만 허용한다. SectionDef도 같은 경계 안의 첫
직접 자식만 canonical owner로 가져간다.

중첩 header 뒤의 직접 자식 `CTRL_DATA`는 바탕쪽 등 raw 구조의 일부일 수 있으므로 원래 위치에
보존한다. serializer의 legacy IR 중복 방어도 같은 경계 안에서만 작동한다. 다음 테스트를 추가했다.

- parser: 중첩 header 뒤의 직접 자식은 `ctrl_data_records`로 이동하지 않고 raw 자식에 남는다.
- serializer: 중첩 header 뒤의 동일 payload는 exact duplicate로 오인해 제거하지 않는다.

기존 #3507 실물 경로는 그대로 통과하며 최종 후보 바이트도 Stage 1 후보와 완전히 같다.

## 전체 검증 결과

| 검증 | 결과 |
|---|---|
| 전체 IR field sweep | 2 passed, 803개 샘플, 회귀 0 |
| `cargo test --profile release-test --tests` | 전체 test binary 실패 0; lib 2,988 passed / 7 ignored |
| `cargo fmt --all -- --check` | pass |
| `git diff --check` | pass |
| `cargo clippy --all-targets -- -D warnings` | pass |

renderer·layout·WASM API 변경이 없으므로 별도 WASM build와 신규 시각 sweep은 요구 범위가 아니다.

PR 준비 직전 전진한 `upstream/devel` `9c69bc3d3` 위로 4개 커밋을 재배치했다. 공용
오늘할일 문서의 최신 기록을 보존해 충돌을 해결한 뒤 위 게이트를 모두 다시 실행했다.
IR sweep dump는 갱신된 baseline과 byte-for-byte 동일했다.

Draft PR 생성 직후 `devel`에 #3517의 CI·문서 변경 2커밋이 추가되어 `f32d964df` 위로 다시
재배치했다. 이 기준 이동은 `.github`와 문서에만 한정되어 Rust 소스·테스트·샘플 트리는
`9c69bc3d3` 기준과 동일하다.

## 최종 인수 후보

| 항목 | 값 |
|---|---|
| 파일 | `/private/tmp/rhwp-issue-3507-artifacts.hiqXmd/bokhak-set-cell-final.hwp` |
| 크기 | 110,080 B |
| SHA-256 | `2ced2123f59cf0b7e5aac6167138139b82d471ac4e81b3fe46b5fc881ab3ce1e` |
| Stage 1 후보와 byte 비교 | 동일 |
| 셀 값 | `(table=0,row=1,col=1) = 강남대학교` |
| SectionDef CTRL_DATA | 1개 / 280바이트 / 원본 payload와 동일 |

## 외부 호환성 최종 판정

| 환경 | 결과 |
|---|---|
| macOS 한컴오피스 한글 Viewer | 파일 손상 대화상자 없이 정상 개방, 편집 값 표시 |
| Windows 한글 2024 | 작업지시자 수동 확인으로 정상 개방 |

2026-07-28 작업지시자가 Windows 한글 2024에서도 정상 개방됨을 확인했다. 두 외부 판정과 모든
로컬 검증이 통과했으므로 #3507 구현의 인수 조건을 충족했고 오늘할일 상태를 `완료`로 변경했다.

계획 대비 차이와 전체 인수 판정은
[`task_m100_3507_report.md`](../report/task_m100_3507_report.md)에 종합한다.
