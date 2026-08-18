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
