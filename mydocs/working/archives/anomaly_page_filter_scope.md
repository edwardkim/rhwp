# layout-anomaly `-p` 의 적용 범위 (#6348)

## 무엇이 문제였나

`rhwp layout-anomaly <파일> -p N` 은 **`pages` 배열만** 걸러냈다. 카운트와
`hasSignal`, 그리고 `--strict` 종료코드는 필터 이전의 **문서 전체** 값이 그대로
나갔다. 그래서 한 봉투 안에서 서로 모순되는 값이 나온다.

`samples/2025 행정업무운영 편람(최종).hwp` (384 쪽) 실측 — 수정 전:

```
-p 0   pages=0개  overflow=163 offCanvas=11 overlap=1 textOverlap=24 emptyPage=18
       hasSignal=true   --strict 종료코드=3
```

0 쪽에는 신호가 하나도 없는데 `hasSignal: true` 가 나오고 `--strict` 는 3 을 냈다.
`pages: []` 와 `overflowCount: 163` 이 같은 JSON 안에 함께 있으니, 이 출력으로는
**"이 쪽이 깨끗한가"를 판정할 수 없다.** 쪽 단위로 좁혀 보는 것이 `-p` 의 유일한
용도이므로 실질적으로 `-p` 는 사람이 눈으로 배열을 세는 용도 외에는 쓸 수 없었다.

`--batch` 도 같았다. `fill_ok_row` 가 문서 전체 카운트로 `row.status` 를 정해,
`-p 0` 을 줘도 그 쪽과 무관하게 `ANOMALY` 로 찍혔다.

## 어떻게 고쳤나

`src/diagnostics/layout_anomaly.rs` 에 `filtered_for_page()` 를 두고, `-p` 를
**한 번만** 적용한 뒤 그 결과에서 카운트·`hasSignal`·종료코드를 모두 끌어낸다.

- 단일 파일 경로(`run`): 범위 검사 직후 한 번 걸러 JSON·사람용 출력·종료코드가
  같은 집합을 본다.
- 배치 경로(`fill_ok_row`): 행 카운트와 `status`, 봉투가 같은 집합을 본다.
- `envelope()` 안에서도 한 번 더 적용한다. 이미 걸러진 값을 받아도 무해하며
  (idempotent), 호출자가 빠뜨렸을 때 봉투 안에서 배열과 카운트가 어긋나는 것을 막는다.
- `page` 가 `None` 이면 `Cow::Borrowed` 라 종전 경로에 복사 비용이 없다.

`pageCount` 는 그대로 문서 전체 쪽수다. 필터와 무관한 메타데이터이고,
`pageFilter` 와 같이 읽으면 "전체 N 쪽 중 M 쪽" 이 된다.

`hasSignal` 의 정의(=`empty_page` 는 확정 신호가 아니다)는 건드리지 않았다.
좁힌 집합에 같은 정의를 적용할 뿐이다.

## 검증 실측

`samples/2025 행정업무운영 편람(최종).hwp` — 수정 후:

| 실행 | pages | overflow | offCanvas | overlap | textOverlap | emptyPage | hasSignal | `--strict` |
|------|------:|---------:|----------:|--------:|------------:|----------:|-----------|-----------:|
| `-p 68` | 1 | 4 | 0 | 0 | 0 | 0 | true | 3 |
| `-p 0` | 0 | 0 | 0 | 0 | 0 | 0 | false | 0 |
| `-p 2` | 1 | 0 | 0 | 0 | 0 | 1 | false | 0 |
| 필터 없음 | 149 | 163 | 11 | 1 | 24 | 18 | true | 3 |

`-p 2` 는 `empty_page` 만 있는 쪽이다. 확정 신호가 아니므로 `hasSignal=false`,
종료코드 0 — 문서 전체 실행에서의 정의와 같다. 필터 없는 실행 값은 수정 전과
동일하다(회귀 없음).

배치 — `table_giant_cell_overfill.hwpx`(신호는 18·39~44 쪽) 와
`hwp3-sample.hwp` 한 폴더:

```
-p 0     [CLEAN]   overflow=0  table_giant_cell_overfill.hwpx
필터 없음 [ANOMALY] overflow=3  table_giant_cell_overfill.hwpx
```

### 게이트

```
cargo fmt --all -- --check                                   통과
cargo clippy --profile release-test --all-targets -- -D warnings   통과
node scripts/rust-test-suite-manifest.mjs --check --base-ref upstream/devel   통과
node scripts/rust-unit-test-tiers.mjs --check --base-ref upstream/devel       통과
cargo test --test regression_suite_031   131 passed
cargo test --test regression_suite_024   134 passed
cargo test --test regression_suite_023   126 passed
cargo test --test regression_suite_032   138 passed
```

## 회귀 테스트

`tests/cases/layout_anomaly_contract.rs` 의
`layout_anomaly_page_filter_scopes_counts_and_strict_exit`.

쪽 번호를 상수로 박지 않는다. 필터 없는 실행의 `pages` 에서 overflow 가 있는 쪽과
어느 목록에도 없는 쪽을 뽑아 쓴다. 조판이 바뀌어 신호가 다른 쪽으로 옮겨가도
계약 자체는 계속 검사된다.

수정 전 코드에서 이 테스트는 실패한다(실측):

```
left: Number(3)   right: 1
```

`-p <신호 있는 쪽>` 인데 `overflowCount` 가 문서 전체 값 3 으로 나오는 것을 잡는다.

## 남는 것

`--batch -p N` 에서 그 쪽 번호가 없는 문서는 종전처럼 `error` 행이 되고 종료코드
1(측정 실패)이 된다. 쪽 수가 제각각인 폴더에 `-p` 를 주면 대부분이 error 가 되는데,
이는 "전건을 재봤다고 말할 수 없으면 실패" 라는 기존 계약이라 이 변경에서 건드리지
않았다.
