---
kind: investigation
status: active
canonical: mydocs/working/gym_studio_rendertree.md
last_verified: 2026-08-18
---

# studio-e2e · render-tree pack 확장 작업 노트

이 문서는 이슈 #5262 (`feat/gym-studio-rendertree-expand`) 를 키운 작업
기록이다. 규범 문서는 각 pack README 다.

- [gym/packs/studio-e2e/README.md](../../../gym/packs/studio-e2e/README.md)
- [gym/packs/render-tree/README.md](../../../gym/packs/render-tree/README.md)

## 무엇을 했는가

devel 의 두 pack 은 과제 한 줄이었다. studio-e2e 는 ST01(차트 첫 칸
91.7), render-tree 는 RT01(2010-01-06 첫 쪽)뿐이라 에이전트가 힌트 한
줄을 외워 축 전체를 통과할 수 있었다. 같은 계약으로 표본·칸·쪽·플래그를
갈라 ST02–ST40 · RT02–RT40 을 더하고, 한국어 README 두 장과 계약 테스트
`scripts/tests/test_gym_studio_rendertree_packs.py` 를 붙였다.

건드리지 않은 것:

- 새 CLI, 새 pack, 새 연산자, 새 샘플
- `automation` · `core-cli` · `casual-rides` · `expert-challenges` 및
  다른 열린 PR 의 pack·도구
- `gym/core/checks.py` · `profiles/` · `gym/README.md` · `gym/PARK.md`
- `cargo fmt --all` (JSON·문서·테스트만 바꿨다)
- `pack.json` 의 `runner` 신원 (`rhwpVersion` / `rhwpCommit` /
  `capabilitiesSha256`). 요구 명령 목록도 기존 값을 유지했다.
- ST01 · RT01 본문. 기존 계약을 덮어쓰지 않았다.

## 왜 이 두께인가

두 pack 은 명령 수가 적다. 얇게 두면 한 표본 × 한 칸이 축 전부가 된다.
같은 `csv-to-chart` 라도 계열 0 과 계열 1, HWP 와 HWPX, `out.hwp` 와
`out.hwpx` 가 다른 계약이다. 같은 `export-render-tree` 라도 `-p 0` 의
001 과 `-p 3` 의 004, `--show-para-marks` 와 기본 추출이 갈라진다.

과제를 합치면 에이전트가 "첫 칸을 91.7 로, 첫 쪽 001 만 내면 된다"고
학습한다. 갈라 두면 자리를 다시 지목해야 한다.

## 과제 계보

### studio-e2e

#### 기존 (devel)

| ID | 명령 | 요지 |
|---|---|---|
| ST01 | `csv-to-chart` | 묶은세로막대 계열0값0 4.3→91.7 |

#### 이번 확장 (ST02–ST40)

| 묶음 | ID | 요지 |
|---|---|---|
| 편집 | ST02–ST05 | 같은 HWP, 다른 칸 (계열 1/2, 둘째·마지막 행) |
| 편집 | ST06–ST07 | 같은 칸의 HWPX 쌍, 산출 `out.hwpx` |
| 조사 | ST08–ST12 | 묶은세로 HWP/HWPX 의 chartCount·rowCount·colCount |
| 조사 | ST13–ST16 | 가로·꺾은선·원형·분산형 차트 수 |
| 조사 | ST17–ST19 | 실사용 보고서 전부/첫째 행/둘째 열 |
| 조사 | ST20–ST21 | 누적·3D 세로 차트 수 |
| 추출 | ST22–ST26 | 머리·라벨·HWPX·분산형 X·BOM |
| 추출 | ST27–ST30 | 꺾은선·원형·가로·보고서+행 수 |
| 조사 | ST31–ST36 | 차트 번호, 전부 추출, 누적가로, 표식, 쪼갠 원형, 3D 원형 |
| 추출 | ST37–ST38 | 백분율 누적·주식형 시트 |
| 하한 | ST39–ST40 | `value_ge chartCount` · `len_ge charts` |

편집 센티넬: `91.7` · `88.1` · `77.3` · `66.2` · `55.9`. 채점은 항상
`changedCount == 0` (같은 CSV 를 `--dry-run` 으로 다시 적용). 전역
훑기 `deep_contains` 는 쓰지 않았다.

### render-tree

#### 기존 (devel)

| ID | 명령 | 요지 |
|---|---|---|
| RT01 | `export-render-tree -p 0` | `2010-01-06.hwp` → `001`, minBytes 10000 |

#### 이번 확장 (RT02–RT40)

| 묶음 | ID | 요지 |
|---|---|---|
| 첫 쪽 | RT02–RT17 | 표·문단·시험지·가로·영문·서식·필드·각주·미주·그림·수식·다중표·실문서·KTX·차트 |
| HWPX | RT18–RT22 | pic2 · sample2 · form-01 · basic-table · 차트 HWPX |
| 뒤 쪽 | RT23–RT29, RT38–RT39 | 2p/3p/4p 의 001–004 |
| 플래그 | RT30–RT34, RT40 | para-marks · control-codes · vpos-reset |
| 루트만 | RT35–RT37 | Page 루트, 쪽수 답 없음 |

확장 과제의 `minBytes` 는 200 이다. 빈 껍데기만 거르고, 쪽 수·bbox 를
박제하지 않는다. 구조 표지는 `json_value_eq type == Page` 다.

## 제약과 지킨 것

1. **기존 표본만.** 모든 `input` 은 `samples/` 아래 실재 파일이다.
2. **기존 연산자만.** `file_exists` · `differs_from_input` · `value_eq` ·
   `answer_eq` · `csv_cell_eq` · `json_value_eq` · `utf8_bom` ·
   `value_ge` · `len_ge`. 스키마에 없는 연산자는 없다.
3. **편집에 전역 훑기 없음.** 편집 과제는 `changedCount` 를 `value_eq`
   로 지목한다. `deep_contains` 없음.
4. **라이브 오라클.** `chartCount` · `rowCount` · `colCount` ·
   `pageCount` 를 과제 JSON 에 숫자로 박제하지 않았다. 예외는 편집
   센티넬 재적용 `changedCount == 0`, 분산형 머리 `X`, 카테고리 머리
   `계열 1`/`항목 1`, 렌더 트리 루트 `Page` 뿐이다.
5. **차트 번호는 1 기준.** `--chart` 인자는 1 이상이다.
6. **렌더 쪽은 0 기준.** `-p 0` 이 첫 쪽, 파일은 `쪽+1`.
7. **고유 ID.** ST·RT 접두사는 다른 pack 과 충돌하지 않는다.

## 위험

- ST18·ST30 은 실사용 보고서의 행 수에 묶여 있다. 문서가 바뀌면 라이브
  오라클 정답이 바뀐다. 그게 조사 과제의 계약이다.
- 분산형 ST25 의 `X` 머리는 포맷 표지다. 카테고리형 시트를 그 자리에
  내면 실패하는 것이 맞다.
- 다쪽 시험지(RT23–RT28)는 이름에 적힌 쪽 수를 전제로 `-p` 를 올린다.
  재조판으로 쪽 수가 줄면 없는 쪽 요청이 된다. 입력은 layout-rendering
  이 이미 쓰는 `exam-kor-Np.hwp` 계열이다.
- 편집 CSV 는 묶은세로막대 데모 시트(3계열×4값)에만 묶여 있다. 그
  샘플의 값이 바뀌면 ST02–ST07 자산도 같이 고쳐야 한다.

## 검증

저장소 루트에서:

```
python gym/tools/audit.py
python -m unittest scripts.tests.test_gym_packs
python -m unittest scripts.tests.test_gym_studio_rendertree_packs
```

`cargo fmt --all` 는 돌리지 않는다. JSON·Markdown·Python 만 바꿨다.

## 의도적으로 하지 않은 것

- `pack.json` runner 신원 갱신. 점수 신원은 기존 커밋에 묶여 있다.
- `requires.commands` 확장. studio 는 `chart-to-csv`/`csv-to-chart`,
  render-tree 는 `export-render-tree`/`info` 만 쓴다.
- 새 골든 바이트, 새 샘플, 새 연산자, 새 하위명령.
- gym 프로필·PARK 문서 수정. 과제 수 표기가 남아 있으면 후속 문서
  PR 에서 맞춘다.
- `from_e2e.mjs` 변경. 열린 PR `#5241` 과 겹친다. ST01 어댑터 경로는
  README 에만 남겼다.
