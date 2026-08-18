---
kind: guide
status: active
canonical: gym/packs/render-tree/README.md
last_verified: 2026-08-18
---

# render-tree — 페이지별 렌더 트리 구조 추출

이 pack 은 **눈이 아니라 파일로 렌더 트리를 뽑는** 축이다. `export-render-tree` 는
stdout JSON 이 아니라 쪽 파일만 만든다(`--json` 없음). 그래서 기준 풀이는 명령을
실제로 실행해 `_rt/render_tree_NNN.json` 을 남기고, 채점은 그 파일의 존재와
루트 `type` 으로 명령을 강제한다. 쪽수는 `info` 라이브 오라클이다.

새 CLI 는 없다. 기존 `export-render-tree` · `info` 와 `samples/` 만 쓴다.
`pack.json` 의 `requires.commands` 와 `runner` 신원은 그대로 둔다.

RT01 한 줄만 있으면 에이전트는 "2010-01-06 의 001 만 내면 축 전체"라고
학습한다. 같은 명령이라도

- 표본이 표인가, 시험지인가, 서식인가, HWPX 인가
- `-p` 가 0 인가 1 인가 2 인가 3 인가
- 파일 번호가 001 인가 002 인가 004 인가
- 조판부호·제어코드·vpos-reset 플래그를 켰는가
- 쪽수 답을 같이 내는가, Page 루트만 보는가

가 다른 계약이다.

## 쪽 번호 계약

| 말하는 것 | 값 | 산출 파일 |
|---|---|---|
| 첫 쪽 | `-p 0` | `_rt/render_tree_001.json` |
| 둘째 쪽 | `-p 1` | `_rt/render_tree_002.json` |
| 셋째 쪽 | `-p 2` | `_rt/render_tree_003.json` |
| 넷째 쪽 | `-p 3` | `_rt/render_tree_004.json` |

`-p` 는 **0 기준**이다. 파일 번호는 **쪽+1** 이다. `info.pageCount` 는
문서 전체 쪽 수이지 파일 번호가 아니다.

루트 JSON 은 `{type, bbox, children}` 이고 `type` 은 `Page` 다. 원본 HWP 를
그 자리에 두거나 `info` 봉투를 저장하면 `json_value_eq` 가 거절한다.

## 여정 지도

### J1. 첫 쪽을 뽑는다 (`-p 0` → `001`)

| ID | 하는 일 | 표본 |
|---|---|---|
| RT01 | 다쪽 실문서 | `2010-01-06.hwp` |
| RT02 | 표 | `table-001.hwp` |
| RT03 | 문단 | `para-001.hwp` |
| RT04 | 1쪽 시험지 | `exam-kor-1p.hwp` |
| RT05 | 가로 | `landscape-001.hwp` |
| RT06 | 영문 | `basic/english.hwp` |
| RT07 | 서식 | `form-01.hwp` |
| RT08 | 누름틀 | `field-01.hwp` |
| RT09 | 각주 | `footnote-01.hwp` |
| RT10 | 미주 | `endnote-01.hwp` |
| RT11 | 그림 | `pic2.hwp` |
| RT12 | 수식 | `math-001.hwp` |
| RT13 | 다중 표 | `multi-table-001.hwp` |
| RT14 | 실문서 | `aift.hwp` |
| RT15 | KTX | `basic/KTX.hwp` |
| RT16 | 차트 | `묶은세로막대형.hwp` |
| RT17 | 실문서 | `143E433F503322BD33.hwp` |

**실패 모드**

- 모든 표본에 RT01 의 001 을 복사한다. 트리는 문서마다 다르다.
- `-p 1` 로 첫 쪽을 요청한다. 첫 쪽은 0 이다.
- `export-svg` 나 `thumbnail` 로 우회한다. 이 pack 은 렌더 트리 파일만 본다.

### J2. HWPX 첫 쪽

같은 그림의 쌍이라도 파일이 다르다.

| ID | 하는 일 | 표본 |
|---|---|---|
| RT18 | 그림 HWPX | `pic2.hwpx` |
| RT19 | 샘플 HWPX | `hwpx_sample2.hwpx` |
| RT20 | 서식 HWPX | `hwpx/form-01.hwpx` |
| RT21 | 표 HWPX | `hwpx/basic-table-01.hwpx` |
| RT22 | 차트 HWPX | `묶은세로막대형.hwpx` |

**실패 모드**

- `pic2.hwp` 트리를 `pic2.hwpx` 과제에 낸다.
- studio-e2e 의 CSV 를 JSON 자리에 둔다.

### J3. 뒤 쪽을 지목한다

다쪽 시험지에서만 `-p` 를 올린다. 없는 쪽을 요청하지 않는다.

| ID | 하는 일 | 표본 | `-p` | 파일 |
|---|---|---|---|---|
| RT29 | 2쪽의 첫 쪽 | `exam-kor-2p.hwp` | 0 | 001 |
| RT23 | 2쪽의 둘째 | 같은 문서 | 1 | 002 |
| RT38 | 3쪽의 첫 쪽 | `exam-kor-3p.hwp` | 0 | 001 |
| RT24 | 3쪽의 둘째 | 같은 문서 | 1 | 002 |
| RT25 | 3쪽의 셋째 | 같은 문서 | 2 | 003 |
| RT39 | 4쪽의 첫 쪽 | `exam-kor-4p.hwp` | 0 | 001 |
| RT26 | 4쪽의 둘째 | 같은 문서 | 1 | 002 |
| RT27 | 4쪽의 셋째 | 같은 문서 | 2 | 003 |
| RT28 | 4쪽의 넷째 | 같은 문서 | 3 | 004 |

**실패 모드**

- `-p 2` 에 `render_tree_002.json` 을 낸다. 파일 번호는 쪽+1 이다.
- 2쪽 문서의 002 를 3쪽·4쪽 문서에 복사한다.
- 이름에 `4p` 가 있다고 `-p 4` 를 준다. 마지막 쪽은 3 이다.

### J4. 플래그를 켠다

같은 쪽이라도 플래그가 있으면 자식이 달라질 수 있다. 기본 추출을
재사용하지 마라.

| ID | 플래그 | 표본 |
|---|---|---|
| RT30 | `--show-para-marks` | `table-001.hwp` |
| RT31 | `--show-control-codes` | `para-001.hwp` |
| RT32 | `--respect-vpos-reset` | `exam-kor-1p.hwp` |
| RT33 | `--show-para-marks` | `landscape-001.hwp` |
| RT34 | `--show-control-codes` | `basic/english.hwp` |
| RT40 | `--show-para-marks` | `2010-01-06.hwp` (RT01 과 같은 문서) |

**실패 모드**

- RT01 산출을 RT40 에 낸다. 플래그가 계약이다.
- 플래그 이름을 과제 JSON 에 적기만 하고 기준 풀이 `run` 에서 뺀다.

### J5. Page 루트만 본다

쪽수 답을 요구하지 않는 입문 과제다. 그래도 트리 파일은 있어야 한다.

| ID | 하는 일 | 표본 |
|---|---|---|
| RT35 | 서식 Page 루트 | `form-01.hwp` |
| RT36 | 누름틀 Page 루트 | `field-01.hwp` |
| RT37 | 각주 Page 루트 | `footnote-01.hwp` |

**실패 모드**

- `answer.json` 만 내고 트리 파일을 빼먹는다.
- 루트가 없는 빈 객체 `{}` 를 낸다. `type` 이 `Page` 여야 한다.

## 라이브 오라클

쪽수(`pageCount`)는 과제 JSON 에 숫자를 박제하지 않는다. `answer_eq` 가
채점 시점에 `info --json` 을 다시 읽는다. 박제하는 값은 구조 표지뿐이다.

- `json_value_eq` 의 `type == Page` — 렌더 트리 루트 이름. 쪽 수·bbox
  숫자가 아니다.
- `file_exists` 의 `minBytes` — 빈 파일 거부. RT01 만 큰 실문서라
  10000 을 쓰고, 확장 과제는 200 으로 빈 껍데기만 거른다.

## 재현 (기준풀이 왕복)

```bash
python gym/tools/build_baseline.py --agent baseline --pack render-tree --bin target/debug/rhwp
python gym/score.py               --agent baseline --pack render-tree --bin target/debug/rhwp
```

`runner` 블록은 기존 왕복을 검증한 바이너리 신원(v0.8.4)이며 이 확장에서
갱신하지 않는다.

## 이 pack 이 아닌 것

- 조판 쪽수·형식 검증은 `layout-rendering` (`info` · `verify`).
- 차트 숫자 시트는 `studio-e2e` (`chart-to-csv`).
- SVG/PNG 눈 비교는 `export-svg` / `render-diff` — 이 pack 의 명령이 아니다.

관련: 이슈 #5262.
