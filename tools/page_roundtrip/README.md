# page_roundtrip — 페이지 수 왕복 공통 하네스 (M05-1)

판정 도구. `pages(원본) == pages(export → reimport)` 를 문서마다 기록한다.

- `scripts/visual_sweep.py` · `gym/` 는 건드리지 않는다.
- DocumentCore / serializer 구현을 바꾸지 않는다. 기존 CLI (`export-hwpx` / `convert --verify-pages`) 만 부른다.
- `#3518` `#3521` `#3737` `#4056` 은 카탈로그 expected-fail 로 남긴다.
- `#4882` (정책연구용역 중간진도보고서)는 M05-6, `#5128` (한글문서파일형식 5.0)은 M05-7에서 고쳤다. 둘 다 카탈로그에서 뺀다.
- `#4056` · ole/shape-component · char_shapes 는 이 좌석에서 고치지 않는다.
- 판정은 데이터다. 기본 종료 코드는 불일치가 있어도 0. `--strict` 만 신규 위반·ERROR 에서 1.

## 1커맨드 (CI 부분집합)

저장소 루트에서:

```text
python tools/page_roundtrip/harness.py --ci
python tools/page_roundtrip/harness.py --ci --json
python tools/page_roundtrip/harness.py --ci --strict
```

`--ci` 는 `tools/page_roundtrip/fixtures/ci-subset.json` 을 읽는다. samples/ 전수가 아니다.

단위 시험 (가짜 rhwp, 실문서 전수 불필요):

```text
python -m unittest tools.page_roundtrip.test_harness tools.page_roundtrip.test_note_probe tools.page_roundtrip.test_analyze
python -m unittest tools/page_roundtrip/test_harness.py tools/page_roundtrip/test_note_probe.py tools/page_roundtrip/test_analyze.py
```

#4882 실측 코퍼스 (100MB hwp 를 다시 커밋하지 않는다. 동반 HWPX XML 스니펫 + 리포트):

```text
python tools/page_roundtrip/build_issue_4882_corpus.py
```

산출: `fixtures/issue_4882/` (각주 lineseg 인덱스·NDJSON·측정 리포트), `transcripts/` (수정 전 215→223 / 수정 후 215==215).

## samples/ + --limit

```text
python tools/page_roundtrip/harness.py --docs samples --limit 20
python tools/page_roundtrip/harness.py --file samples/foo.hwp --route hwpx
python tools/page_roundtrip/harness.py --file "samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwp" --route hwpx --transcript-dir tools/page_roundtrip/transcripts/out
```

`--limit N` 은 문서 N개다. 경로는 `--route` 로 펼친다.

## Full sweep (로컬, 무거움)

`samples/` 아래 `.hwp` / `.hwpx` 전수. 카탈로그 항목은 빠지지 않고 리포트 `catalog` 절에 남는다.

```text
cargo build --release --bin rhwp
python tools/page_roundtrip/harness.py --docs samples
python tools/page_roundtrip/harness.py --docs samples --json > page-roundtrip.json
python tools/page_roundtrip/harness.py --docs samples --route both
python tools/page_roundtrip/harness.py --docs samples --strict
```

매니페스트로 목록을 고정하려면:

```text
python tools/page_roundtrip/harness.py --docs samples --write-manifest tools/page_roundtrip/fixtures/full.json
python tools/page_roundtrip/harness.py --manifest tools/page_roundtrip/fixtures/full.json
```

## 경로 (route)

| route | 명령 | 기본 |
| --- | --- | --- |
| `hwpx` | `rhwp export-hwpx <입력> <tmp.hwpx> --verify-pages --json` | 기본. 이슈 가족(#3518 등)과 같다 |
| `hwp` | `rhwp convert <입력> <tmp.hwp> --verify-pages --json` | HWP5 저장 왕복 |
| `both` | 위 둘을 문서마다 한 행씩 | 전수 매트릭스 |

기존 CLI 가 원본 쪽수 → 내보내기 → 재파싱 쪽수를 잰다. 하네스는 그 봉투의 `verifyPages.before/after/identical` 을 읽어 판정한다.

## expected-fail 카탈로그

`tools/page_roundtrip/catalog.json`. 알려진 위반을 숨기지 않는다.

| 판정 | 뜻 |
| --- | --- |
| `MATCH` | 쪽수 동일, 카탈로그 밖 |
| `MISMATCH` | 쪽수 불일치, 카탈로그 밖 — **신규 위반** |
| `EXPECTED_FAIL` | 쪽수 불일치, 카탈로그에 있음 |
| `UNEXPECTED_PASS` | 쪽수 동일, 카탈로그에 있음 — 카탈로그가 낡음 |
| `ERROR` | 로드/내보내기/재파싱 실패 |
| `CATALOG_MISSING` | 카탈로그 경로의 파일이 디스크에 없음 |

`--limit` / `--manifest` / `--file` 때문에 이번 실행에 안 들어간 카탈로그 항목은 `catalog[].state=held` 로 남는다. 건너뛴 것처럼 지우지 않는다.

## 불일치 재현

리포트 `repro` 열을 그대로 실행한다.

```text
python tools/page_roundtrip/harness.py --file samples/hwp3-sample16.hwp --route hwpx
rhwp export-hwpx samples/hwp3-sample16.hwp /tmp/rt.hwpx --verify-pages --json
```

## 종료 코드

| 코드 | 조건 |
| --- | --- |
| 0 | 기본. MATCH/MISMATCH/EXPECTED_FAIL 도 데이터 |
| 1 | `--strict` 이고 MISMATCH · ERROR · UNEXPECTED_PASS · CATALOG_MISSING ≥ 1 |
| 2 | 인자/매니페스트/카탈로그 사용법 오류 |

`EXPECTED_FAIL` 은 `--strict` 에서도 통과다 (이미 이슈로 추적 중).
