---
kind: working
status: active
issue: 6315
---

# 글자 겹침(text-overlap) 래칫 게이트 (#6315)

## 무엇을

`layout-anomaly` 의 text-overlap 판정을 samples 전수 래칫으로 묶어, PR 이 새 글자 겹침을
만들면 `Build & Test` 에서 실패하게 한다.

- `tests/cases/text_overlap_baseline.rs` — samples 전수 스캔, 문서별 건수 래칫
- `tests/fixtures/text_overlap_baseline.tsv` — 현재값 고정(152 문서 / 4,371 건)

판정 로직(`src/diagnostics/layout_anomaly.rs`)은 한 줄도 바꾸지 않는다.

## 왜

판정기는 이미 있고 정확한데, 그 판정을 PR 에서 돌리는 경로가 없다.
`.github/workflows/layout-anomaly-advisory.yml` 는 nightly 전용이고
(`pull_request` 트리거 주석 처리, `continue-on-error: true`), 그래서 글자 겹침 회귀가
required check 를 전부 초록으로 통과한다.

실사례가 PR #6083 이다. 같은 문서·같은 쪽에서:

| 대상 | 편람 69쪽 text-overlap |
| --- | --- |
| `devel` b1485e0a14 | 0 건 |
| PR #6083 head cfb2646e | 7 건 |

7 건은 모두 `Table0/Cell4/TextLine16/TextRun0`(유의사항 상자 마지막 줄) 대
`Body/Column0/TextLine2/TextRun1..7`(상자 다음 본문 줄) 짝이고 겹침은 102.6 x 13.3 px 다.
검토자가 PNG 를 눈으로 비교해 발견했고, 검토 코멘트에 "CI가 통과한 이유는 이 겹침을
검사하는 시험이 없기 때문입니다" 로 남아 있다.

## 어떻게 — 왜 래칫인가

`tools/layout_anomaly/ci_advisory.md` §2 가 게이트 승격을 미룬 이유는 "소표본에도 이미
알려진 겹침이 있어 지금 강제 게이트로 켜면 devel PR 이 한꺼번에 막힌다" 였다. 실측이
그 우려를 확인한다 — samples 524 건 중 **152 문서에서 4,371 건**이 이미 있다.

이 저장소는 같은 상황을 다른 축에서 이미 래칫으로 풀었다.

| 축 | 게이트 | 픽스처 |
| --- | --- | --- |
| `LAYOUT_OVERFLOW_CELL` (#3668) | `tests/overflow_cell_baseline.rs` | `overflow_cell_baseline.tsv` |
| IR field sweep | `ir_field_sweep_baseline` | 동명 TSV |
| **글자 겹침 (이 작업)** | `tests/cases/text_overlap_baseline.rs` | `text_overlap_baseline.tsv` |

기존 발생은 baseline 에 싣고 **신규 발생·증가만** 실패시킨다. 감소는 통과다(dump 로
확인한 뒤 래칫을 조인다). `local_validation.md` §4.3.0.1 의 "가능하면 최소 공개 fixture 와
자동 래칫을 마련한다" 를 이 축에 적용한 것이다.

### required check 를 건드리지 않는 이유

required check / branch protection 변경은 운영 등급 O4 이고 #5400 이 명시적으로 범위 밖으로
남긴 결정이다. 이 작업은 일반 integration test 로만 들어가므로 기존 `Build & Test` 가
그대로 실행한다 — 워크플로 변경 없이 게이트가 서고, 메인테이너 결정 항목은 열어 둔다.

### 판정 대상을 text-overlap 하나로 좁힌 이유

overflow / off-canvas / overlap 은 컨테이너 기하라 정상 조판의 접합·장식으로도 흔히
잡힌다. 보이는 글자끼리 겹치는 것은 두 글자 모두 읽을 수 없게 되는 확정 결함이다.
모듈 머리말이 `--strict` 확정 신호에 text-overlap 을 포함한 근거와 같다.

## baseline 의 성격 — 숨기지 않는 사실

4,371 건은 "허용해도 되는 겹침"이 아니라 **아직 안 고친 겹침**이다. 이 게이트는 그 수를
줄이지 않는다. 늘지 않게만 한다.

최다 문서(`hwpx/2024년 연간 해외직접투자 보도자료 _ ff.hwpx`, 256 건) 표본 분류:

| 분류 | 건수 | 비율 |
| --- | --- | --- |
| 줄 높이 전체가 겹침 | 190 | 74% |
| 세로 부분 겹침(줄 높이의 50% 미만) | 66 | 26% |

겹침 폭 중앙값 9.7px, 최대 38.5px. 본문 한글 글자 폭이 11~16px 이므로 다수는 글자
하나 이상이 실제로 포개진다. 실제 형상도 인접 칸 침범이다.

```
Cell16/TextLine1/TextRun0  x=81.2..163.2
Cell17/TextLine1/TextRun0  x=159.3..199.3   -> 4.0px 침범
```

세로 부분 겹침 26% 는 렌더 트리에 글자 단위 글리프 bbox 가 없어 `TextRun` 상자를
글리프 묶음으로 쓰는 데서 오는 상자 여백 근접이 섞일 수 있다(모듈 머리말이 밝힌 한계).
판정 자체를 조이는 것은 이 작업의 범위가 아니다 — 조이면 baseline 이 함께 내려가므로
별도 축으로 다룬다.

## 검증

| 명령 | 결과 |
| --- | --- |
| `cargo fmt --all -- --check` | 통과 |
| `cargo clippy --profile release-test --all-targets -- -D warnings` | 통과 (9분 04초, 경고 0) |
| 래칫 테스트 on `devel` | 통과 (299.46s, 945 건 스캔 / 스킵 3) |
| 래칫 테스트 + #6083 composer 변경 | **실패** — 게이트가 실제로 잡는다 |

현재값 재생성:

```bash
RHWP_TEXT_OVERLAP_DUMP=<path> cargo test --profile release-test text_overlaps_do_not_grow
```

쪽 단위 위치 확인:

```bash
rhwp layout-anomaly "samples/2025 행정업무운영 편람(최종).hwp" -p 68 --json
```

## 실측

### 1. `devel` 기준 — 통과

```
text-overlap 스윕: 샘플 945건(스킵 3) / 0 아닌 문서 152종 / 총 4371건
test text_overlap_baseline::text_overlaps_do_not_grow ... ok
finished in 299.46s
```

수집은 `overflow_cell_baseline.rs` 와 같은 규칙(하위 폴더 재귀, `.hwp`/`.hwpx`)이라
`samples/` 최상위 524 건이 아니라 945 건이 대상이다. 스킵 3 건은 로드·렌더 실패로,
이 게이트의 관심사가 아니다(크래시·파싱 회귀는 기존 스위트가 잡는다).

### 2. #6083 의 `composer.rs` 변경을 얹으면 — 실패

같은 워크트리에 PR #6083 head 의 `src/renderer/composer.rs` diff 만 적용하고
같은 명령을 다시 돌렸다.

```
증가: 2025 행정업무운영 편람(최종).hwp — 15 → 20건
증가: 2025 행정업무운영 편람(최종).hwpx — 15 → 20건
증가: task2430/1382000_domestic_violence_survey.hwp — 20 → 21건
test result: FAILED. finished in 208.77s
```

세 가지를 확인한다.

1. 게이트가 그 회귀를 **잡는다**. 편람 HWP·HWPX 가 각각 +5 건이다. 그 PR 은 당시
   required check 를 전부 초록으로 통과했다.
2. 검토자가 눈으로 본 것보다 **넓게** 잡는다. 검토는 편람 한 문서였는데, 래칫은
   `task2430/1382000_domestic_violence_survey.hwp` 의 +1 건도 함께 보고한다 —
   사람이 렌더해 보지 않은 문서다.
3. 판정은 문서 단위 수치라 **재현 가능**하다. 어느 쪽에서 늘었는지는
   `rhwp layout-anomaly <문서> -p <쪽> --json` 으로 좁힌다.

검증 뒤 `composer.rs` 는 되돌렸다. 이 PR 에는 `src/**` 변경이 없다.

### 3. 커밋 대상

```
tests/cases/text_overlap_baseline.rs
tests/fixtures/text_overlap_baseline.tsv
mydocs/working/text_overlap_gate.md
```

`tests/generated/`, `tests/suites/manifest.json` 은 `.gitignore` 대상이라 커밋에 들어가지
않는다(PR 템플릿 2번 체크박스 규약).

## 남은 것 — 이 게이트의 사각지대

`scan_page` 는 페이지 트리에서 `Body` 하나만 순회한다. `MasterPage`·`Header`·`Footer`·
`FootnoteArea` 는 스캔 대상이 아니라, **본문 글자가 바탕쪽 사이드바를 덮어도 0 건**이다.
편람 69쪽 `devel` 실측에서 본문↔바탕쪽 글자 겹침이 판정기와 같은 허용치로 4 건 있는데
현재 명령은 0 을 낸다(#5952 의 "사이드바와 겹친다" 가 이 형상이다).

이 게이트를 먼저 세운 뒤 별도 이슈로 다룬다 — 순서가 바뀌면 baseline 이 두 번 흔들린다.
