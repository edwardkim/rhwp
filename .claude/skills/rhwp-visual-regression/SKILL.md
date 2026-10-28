---
name: rhwp-visual-regression
description: 편집/변환 전후의 HWP/HWPX 레이아웃 회귀를 숫자로 판정합니다. render-diff(자기 라운드트립·두 파일 비교·폴더 배치, px 변위+구조 불일치) → ir-diff(구조 차이) → thumbnail/export-png(눈 검증) 판단 트리를 태우고, STRUCT_MISMATCH 를 반사적으로 실패 처리하지 않고 노드 경로로 판독합니다. 트리거 — 사용자가 "편집 전후 화면 비교", "레이아웃 회귀/깨졌는지 확인", "라운드트립 시각 검증", "render-diff 돌려줘", "바뀐 게 의도한 것뿐인지" 등을 요청할 때.
---

# rhwp-visual-regression — 전후 시각 회귀 판정 Skill

`edit`/`convert`/`export-hwpx` 를 돌린 뒤 "내용이 바뀌었다"가 아니라
"**의도한 것만** 바뀌고 나머지 레이아웃은 그대로다"를 px 단위 수치로 판정한다.
이 스킬은 **실 에이전트 경로**다. gym 이 아니고, 새 CLI 명령을 발명하지 않는다.
코어는 이미 있는 `render-diff` / `ir-diff` / `thumbnail` / `export-png` 를
그대로 부른다.

IR 비교(`--verify`)로는 안 잡히지만 화면에서는 티가 나는 차이(표 병합·폰트
치환·페이지 넘김)를 잡는다. 판정은 예외가 아니라 **데이터**다.

상세는 `references/` 를 단계별로 연다. SKILL.md 는 인덱스와 정지 규칙만 담는다.

## 바이너리

```bash
cargo build --release
./target/release/rhwp <명령> [옵션]
```

`export-png` 은 `native-skia` feature 빌드 필요(release 바이너리에 포함됨).
공통 규약은 [rhwp-cli skill](../rhwp-cli/SKILL.md) 참조.
산출물은 `output/` 아래 분리 권장. 원본은 어떤 실패에서도 불변이다.

## 사다리 (강제 순회 아님)

`render-diff(자기) → render-diff(전후) → ir-diff --json → thumbnail/export-png`

질문이 이미 답이면 다음 단으로 내려가지 않는다. 각 단의 정지 조건은
[14_failure_signals.md](references/14_failure_signals.md) 와 아래 정지 표.

```
render-diff <파일> [--via hwpx|hwp]          자기 라운드트립
render-diff <A> <B> [--max-disp PX] [-p N]   두 파일 직접 비교
render-diff --batch <폴더> [-o 출력]         geom_inventory.tsv
     │
     ├─ PASS ──▶ 끝 (F01). A==A 는 항상 PASS 여야 한다 (F02)
     │
     ├─ STRUCT_MISMATCH ──▶ 변위 노드 경로를 읽는다 (반사적 실패 금지, F03)
     │     예: Page/Body2/Column0/TextLine10/TextRun0 가 편집 필드면 정상
     │     경로가 편집한 위치와 일치 → 정상 (값이 바뀌면 그 자리 구조도 바뀐다)
     │     편집과 무관한 페이지/단     → 진짜 회귀 (F04)
     │
     ├─ PAGE_MISMATCH ──▶ dump-pages 로 갈라지는 쪽 좁힘 (F05)
     ├─ OVER ──▶ 임계만 넘음, 구조는 동일. worst_page 로 좁힘 (F06)
     └─ LOAD_FAIL ──▶ info 로 그 파일만 따로 연다 (F07)
2. ir-diff <A> <B> --json                    구조(IR) 차이. 차이 = exit 3 (F08)
3. thumbnail / export-png                    눈 검증. thumbnail 은 저장 미리보기 (F09)
4. export-render-tree <파일> -p N            정밀 bbox 좌표 (선택)
```

## 요청 → 명령

| 사용자 요청 | 명령 | 레퍼런스 |
| --- | --- | --- |
| 이 파일 포맷 왕복이 안전한지 | `render-diff <파일> --via hwpx` (HWP 어댑터는 `--via hwp`) | 01_render_diff_self.md |
| 편집 전후 비교해줘 | `render-diff <전> <후> [-p N] [--max-disp PX]` | 02_render_diff_two_file.md |
| 폴더 전체 회귀 게이트 | `render-diff --batch <폴더> [-o 출력]` → `geom_inventory.tsv` | 03_render_diff_batch.md |
| STRUCT 가 떴다 | 노드 경로를 읽는다. 반사적 실패 금지 | 04_struct_mismatch.md |
| 어느 구조가 달라졌는지 | `ir-diff <a> <b> --json` (차이 = exit 3) | 06_ir_diff.md |
| 빨리 눈으로 확인 | `export-png <파일> [-p N]` (재렌더). `thumbnail` 은 저장 미리보기 | 07_thumbnail_vs_png.md |
| 자기 비교가 흔들린다 | `render-diff <A> <A>` 가 PASS 가 아니면 도구 비결정성 | 08_determinism.md |
| 임계를 조여라 | `--max-disp` 기본 1.0px. 구조 불일치는 임계와 무관 | 09_max_disp.md |

## 정지 규칙

| ID | 언제 | 행동 |
| --- | --- | --- |
| F01 | `status: PASS` | 끝. 다음 단으로 내려가지 않는다 |
| F02 | `render-diff A A` 가 PASS 가 아님 | 도구/렌더 비결정성. 문서 회귀보다 심각. 중단 |
| F03 | `STRUCT_MISMATCH` + 경로가 편집 위치 | 정상. 값이 바뀌면 그 자리 구조도 바뀐다. 실패로 읽지 않는다 |
| F04 | `STRUCT_MISMATCH` + 경로가 편집과 무관 | 진짜 회귀. export-png 로 그 쪽을 본다 |
| F05 | `PAGE_MISMATCH` | 쪽 수 자체가 다름. 의도면 정상, 아니면 `dump-pages --json` |
| F06 | `OVER` (구조 동일, 변위 > 임계) | `worst_page` 로 좁힘. 임계가 빡센지 실제 여백 회귀인지 |
| F07 | `LOAD_FAIL` 또는 파일 읽기 실패 | 단건 exit 1 / 배치 폴더 오류 exit 2. `info` 로 그 파일만 |
| F08 | `ir-diff --json` 이 exit 3 | 차이 검출은 데이터. `identical`/`diffCount`/`categories` 로 판정 |
| F09 | 눈으로 확인이 필요 | `export-png`/`export-svg` 를 본다. `thumbnail` 은 저장 시점 PrvImage |
| F10 | `--batch` TSV 에 혼합 상태 | 요약 줄만 보지 않는다. 행별 status 로 격리 |
| F11 | 질문이 이미 답 | 다음 단 금지 |
| F12 | `WARN_TEXTRUN` | TextRun ±1 조성 노이즈(#1773). 하드 실패가 아니다 |

**금지 기본값**

- 새 비교 CLI 발명 (기존 네 명령만)
- gym pack / gym 과제 작성
- `STRUCT_MISMATCH` 를 경로도 안 읽고 롤백
- `thumbnail` 을 편집 후 재렌더로 착각
- `A==A` 실패를 문서 회귀로 오진
- `--max-disp` 를 키워 STRUCT 를 숨기려 함 (구조 불일치는 임계와 무관)
- 원본을 비교 과정에서 덮어쓰기
- 이 스킬 안에서 onboarding / mcp-session / safe-edit / provenance / doc-triage / form-fill 를 재작성

## 인계

- 누름틀 채움 자체 → `rhwp-form-fill` (채운 뒤 여기로 돌아와 비교)
- 표 CSV 왕복 → `rhwp-table-exchange`
- 원본을 계획서로 여러 번 고침 → `rhwp-safe-edit`
- 배포 전 점검 → `rhwp-security-sweep`
- 미지 문서 파악만 → `rhwp-doc-triage` (읽기, 비교하지 않음)

상세: [13_handoff.md](references/13_handoff.md)

## 봉투·종료 코드 — 판정은 데이터다

사람 모드(`render-diff` 텍스트): `PASS`/`WARN_TEXTRUN` 만 0,
`OVER`/`STRUCT_MISMATCH`/`PAGE_MISMATCH`/`LOAD_FAIL` 은 1.
이미 1 을 실패로 읽는 CI 가 있다.

`--json` 모드: 단건은 JSON 한 줄, 배치는 NDJSON.
하드 실패는 **exit 3** (회귀 검출). 파일을 못 읽으면 1, 사용법 2.
`ir-diff --json` 과 같은 "판정은 데이터" 축이다.

```
{"schemaVersion":"1.0","mode":"pair"|"roundtrip","status","maxDisp","regression",
 "pageCountA","pageCountB","threshold","pages":[{path,disp,...}]}
```

`ir-diff --json`:

```
{"schemaVersion":"1.0","a","b","identical","diffCount","categories":{…}}
```

- 0 = 동일 / **3 = 차이 발견** / 1 = 읽기·파싱 실패(stdout 0바이트) / 2 = 사용법
- 기본(텍스트) 모드의 정상 비교는 차이가 있어도 0 (기존 소비자 무변경)

`--max-disp` 기본 **1.0px**. 구조 불일치는 임계값과 **무관하게** 항상 플래그된다.

필드 표: [10_envelopes.md](references/10_envelopes.md) · [20_exit_codes.md](references/20_exit_codes.md)

## 통과 게이트 (기계)

```bash
# 결정성 기준선 — 같은 파일을 두 번 비교하면 반드시 PASS
rhwp render-diff 산출.hwp 산출.hwp; test $? -eq 0

# 배치 CI — TSV 의 status 열을 읽는다 (요약 줄만 보지 말 것)
rhwp render-diff --batch samples -o output/rd
awk -F'\t' 'NR>1 && $2!="PASS" && $2!="WARN_TEXTRUN" {print; bad=1} END{exit bad}' \
  output/rd/geom_inventory.tsv

# ir-diff 변환 게이트 — 차이 = exit 3 = 데이터
rhwp ir-diff A.hwpx B.hwp --json || 격리처리
```

`STRUCT_MISMATCH` 행은 게이트에서 바로 실패로 접지 말고, `struct_delta` 와
노드 경로가 편집 위치와 맞는지를 한 번 더 본다.

## 레퍼런스 목차

1. [00_tree.md](references/00_tree.md) — 판단 트리
2. [01_render_diff_self.md](references/01_render_diff_self.md) — 자기 라운드트립
3. [02_render_diff_two_file.md](references/02_render_diff_two_file.md) — 두 파일
4. [03_render_diff_batch.md](references/03_render_diff_batch.md) — 배치·TSV
5. [04_struct_mismatch.md](references/04_struct_mismatch.md) — 경로로 읽기
6. [05_status_codes.md](references/05_status_codes.md) — PASS/OVER/STRUCT/PAGE/LOAD
7. [06_ir_diff.md](references/06_ir_diff.md) — ir-diff --json
8. [07_thumbnail_vs_png.md](references/07_thumbnail_vs_png.md) — 미리보기 vs 재렌더
9. [08_determinism.md](references/08_determinism.md) — A==A
10. [09_max_disp.md](references/09_max_disp.md) — 임계 1.0px
11. [10_envelopes.md](references/10_envelopes.md) — JSON 봉투
12. [11_pitfalls.md](references/11_pitfalls.md) — 함정
13. [12_journeys.md](references/12_journeys.md) — 실사용 여정
14. [13_handoff.md](references/13_handoff.md) — 다른 스킬로
15. [14_failure_signals.md](references/14_failure_signals.md) — 신호 → 처방
16. [15_node_paths.md](references/15_node_paths.md) — 노드 경로 읽는 법
17. [16_worked_traces.md](references/16_worked_traces.md) — 재현 트레이스
18. [17_intent_matrix.md](references/17_intent_matrix.md) — 발화 → 명령
19. [18_tsv_schema.md](references/18_tsv_schema.md) — geom_inventory.tsv
20. [19_gate_recipes.md](references/19_gate_recipes.md) — jq/awk 게이트
21. [20_exit_codes.md](references/20_exit_codes.md) — 종료 코드
22. [21_page_mismatch.md](references/21_page_mismatch.md) — 쪽 수 불일치
23. [22_load_fail.md](references/22_load_fail.md) — 로드 실패
24. [23_over_status.md](references/23_over_status.md) — OVER
25. [24_export_render_tree.md](references/24_export_render_tree.md) — 정밀 bbox

예제: [examples/](examples/). 기계 가독 픽스처: [fixtures/](fixtures/).

## 권위

- [`mydocs/manual/cli_commands.md`](../../../mydocs/manual/cli_commands.md)
  (`render-diff` · `ir-diff` · `thumbnail` · `export-png` · 종료 코드)
- [`recipes/06_visual_regression_before_after.md`](../../../mydocs/manual/recipes/06_visual_regression_before_after.md)
- [`mydocs/manual/ir_diff_command.md`](../../../mydocs/manual/ir_diff_command.md)
- [`mydocs/manual/export_png_command.md`](../../../mydocs/manual/export_png_command.md)
- 처리 결과: [`mydocs/working/agent_visual_regression.md`](../../../mydocs/working/archives/agent_visual_regression.md)
