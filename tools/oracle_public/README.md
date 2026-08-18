# oracle_public — 페이지 수 스모크 (M01-4)

렌더 없는 1차 게이트. `rhwp dump-pages` 쪽수와 커밋된 한컴 PDF 쪽수를 비교한다.

- `scripts/visual_sweep.py` 를 수정하지 않는다.
- M01-1 `oracle_resolver` 병합을 기다리지 않는다. 로컬 글롭 또는 자체 매니페스트.
- 판정은 데이터다. 기본 종료 코드는 불일치가 있어도 0. `--strict` 만 실패한다.

## 1커맨드 스모크

저장소 루트에서:

```text
python tools/oracle_public/page_smoke.py
```

짝 규칙 (각 PDF 루트, 재귀):

- `pdf/{stem}.pdf`
- `pdf/{stem}-*.pdf`

기본 PDF 루트: `pdf/` · `pdf-2020/` · `pdf-large/`. 문서 루트: `samples/`.

## CI (tiny fixture, 269 PDF 불필요)

```text
python -m unittest tools.oracle_public.test_page_smoke
python -m unittest tools/oracle_public/test_page_smoke.py
```

시험은 임시 디렉터리에 최소 PDF·가짜 dump-pages 만 쓴다. 커밋된 한컴 PDF 전수는 돌리지 않는다.

## Full sweep (로컬, 무거움)

sparse checkout 에 PDF 트리가 있어야 한다.

```text
git sparse-checkout add pdf pdf-2020 pdf-large
cargo build --release --bin rhwp
python tools/oracle_public/page_smoke.py
python tools/oracle_public/page_smoke.py --json > page-smoke.json
python tools/oracle_public/page_smoke.py --strict
```

`--limit N` 으로 앞 N 짝만. 짝 목록을 고정하려면:

```text
python tools/oracle_public/page_smoke.py --write-manifest tools/oracle_public/fixtures/pairs.json
python tools/oracle_public/page_smoke.py --manifest tools/oracle_public/fixtures/pairs.json
```

매니페스트는 `{"pairs":[{"doc":"...","pdf":"..."}]}` 이다. `sample`/`pdf` 키도 받는다 (M01-1 가 같은 모양이면 그대로 읽는다. 임포트하지 않는다).

## 불일치 재현

리포트의 `repro` 열을 그대로 실행한다.

```text
python tools/oracle_public/page_smoke.py --pair samples/foo.hwp pdf/foo-2022.pdf
rhwp dump-pages samples/foo.hwp --json
python tools/oracle_public/page_smoke.py --pdf-count pdf/foo-2022.pdf
```

## 종료 코드

| 코드 | 조건 |
| --- | --- |
| 0 | 기본. 불일치·ERROR 도 데이터로 출력 |
| 1 | `--strict` 이고 MISMATCH 또는 ERROR ≥ 1 |
| 2 | 인자/매니페스트/PDF 시그니처 사용법 오류 (`--pdf-count` 실패 포함) |
## 한컴 기준 PDF 오라클 쌍 매니페스트 (M01-1)

`samples/` 의 `.hwp`/`.hwpx` 와 `pdf/{stem}-{한컴버전}.pdf` 를 짝 지어
**시각 판정 기준(오라클 쌍)** JSON 을 만든다. 렌더·편집 로직은 건드리지 않는다.
`scripts/visual_sweep.py` 는 수정하지 않는다.

한컴 버전은 2018 / 2020 / 2022 / 2024 이다. `-hwp-2020`, `-hwpx-kopub-2020` 같은
변형과, stem 에 연도가 이미 들어 있는 `pdf/{stem}.pdf` 도 인정한다.
`.hwp` 는 `-hwpx-{year}` PDF 와, `.hwpx` 는 `-hwp-{year}` PDF 와 짝짓지 않는다.

목표 약 269쌍은 이슈의 참고값이다. 실제 개수는 매니페스트의 `pairCount` 를 따른다.
devel(sparse `pdf`/`pdf-2020`/`pdf-large`) 실측은 링크 409쌍, 매칭 샘플 389, 짝 없음 305.

## 한 줄로 매니페스트 만들기

저장소 루트에서:

```bash
python tools/oracle_public/oracle_resolver.py --pretty --validate -o tools/oracle_public/oracle_pairs.json
```

오라클 PDF 가 비어 있으면 먼저 `git sparse-checkout add pdf pdf-2020 pdf-large crates` 를 한다.

## 요구사항

- Python 3.10+ (표준 라이브러리만 사용)
- `samples/` 와 하나 이상의 `pdf/` · `pdf-2020/` · `pdf-large/`

## 매칭 규칙

1. `samples/` 를 재귀 순회해 `.hwp`/`.hwpx` 만 수집한다.
2. 같은 상대 하위 경로를 `pdf/`, `pdf-2020/`, `pdf-large/` 에서 찾는다.
3. 파일명이 `{stem}-{year}.pdf` 이거나 `{stem}-hwp-{year}.pdf` /
   `{stem}-hwpx-{year}.pdf` / `{stem}-hwp-kopub-{year}.pdf` 등 허용 변형이다.
4. `{stem}.pdf` 이면서 stem 안에 2018·2020·2022·2024 토큰이 있으면 그 연도로 짝짓는다.
5. 한 샘플이 여러 버전 PDF 를 가지면 링크를 모두 남긴다.
6. 짝이 없는 샘플은 `unmatched` 에 이유와 함께 남긴다.

## 시험

```bash
python tools/oracle_public/tests/test_oracle_resolver.py
```

픽스처는 `tools/oracle_public/fixtures/mini_repo/` 이다.

## 산출

| 필드 | 의미 |
|---|---|
| `pairCount` | `pairs` 길이 (샘플×PDF 링크) |
| `matchedSampleCount` | 오라클이 하나라도 있는 샘플 수 |
| `unmatchedCount` | 짝 없는 샘플 수 |
| `targetPairCount` | 참고 목표 269 |
| `byHancomVersion` | 연도별 링크 수 |
| `pairs[]` | `sample`, `pdf`, `hancomVersion`, `variant`, `oracleRoot` |
| `unmatched[]` | 짝 없는 샘플과 `reason` |
