---
kind: guide
status: active
canonical: gym/tools/README_robustness.md
last_verified: 2026-08-18
---

# robustness.py — 운영 메모

손상-강건성 감사기의 **한 페이지 운영 메모**다. 카탈로그·분류·예외 경로의 정본은
[`gym/docs/robustness.md`](../docs/robustness.md) 다. 작업 기록은
[`mydocs/working/gym_robustness.md`](../../mydocs/working/gym_robustness.md).

## 30초

```bash
python gym/tools/robustness.py --bin target/debug/rhwp
python gym/tools/robustness.py --bin target/debug/rhwp --limit 40 --json
python -m unittest scripts.tests.test_gym_robustness scripts.tests.test_gym_audit
```

손상 문서를 **결정적으로** 만들어 `rhwp info --json` 으로 두드린다. 패닉·행이
있으면 exit 1. 깨끗한 파싱 실패는 정상이다.

## 언제 돌리나

- rhwp 파서/정보 경로를 고친 뒤 회귀 확인.
- 릴리스 게이트에서 "도구가 적대적 입력에 죽지 않는가"를 인증할 때.
- 새 결정적 변형을 넣은 뒤, 시험만으로 부족한 실측이 필요할 때.

돌리지 않는 때:

- pack 채점·리더보드·트라젝토리. 다른 도구다.
- 전 코퍼스 exhaustive 퍼징. `fuzz_corpus.py` 다.
- 포맷 변경. 이번 도구는 Python 만 고친다.

## 인자

| 인자 | 기본 | 메모 |
|---|---|---|
| `--bin` | 필수 | `runner.find_bin` 이 상대/절대/이름 을 해석한다. |
| `--limit` | 16 | `.hwp` 만. 정렬 후 stride. 음수/변환불능은 0건. |
| `--timeout` | 20 | 초. CLI 는 양수만 받는다. 함수 `probe` 는 0 을 오류머리로 접는다. |
| `--json` | off | 봉투를 stdout. 사람용 한 줄은 stderr 가 아니라 stdout. |

표본 디렉터리는 저장소 `samples/` 고정이다. 다른 코퍼스를 넣으려면
`audit(bin, samples_dir, limit, timeout)` 을 직접 부른다.

## 종료 코드

| 코드 | 언제 |
|---|---|
| 0 | 패닉 0 · 행 0 |
| 1 | 패닉 또는 행 |
| 2 | 바이너리를 찾지 못함 |

읽기실패·프로브오류는 0 을 유지한다. 환경 결함을 도구 결함으로 위장하지 않는다.

## 봉투에서 볼 키

성공 판정: `ok`, `panics`, `hangs`.

진단:

- `mutantsChecked` — 프로브까지 간 변형 수. 0 이면 쓰기/프로브가 전부 접힌 것.
- `gracefullyDegraded` — 우아한 실패. 클수록 파서가 손상에 깨끗이 거절한 것.
- `unreadables` — 샘플을 못 읽었거나 변형 생성이 타입 오류.
- `probeErrors` — 없는 바이너리, 디스크, invalid-timeout.
- `inputShapes` — empty/tiny/normal/huge. 한 형태만 있으면 stride 가 편향된 것.

`schemaVersion` 은 `1.0`. 키를 빼거나 더하면 `validate_report()` 가 위반을 낸다.

## 기본 변형 (기존 게이트)

정상 입력에서 항상:

`truncate@25%` `truncate@50%` `truncate@75%` `truncate@95%`
`flip@10%` `flip@50%` `flip@90%`
`zero-header` `header-smash` `ole-trunc-tail` `ff-run` `utf16-nul-sprinkle`

ZIP 로컬 헤더(`PK\x03\x04`)가 있을 때만 `zip-local-header-flip`.

빈 입력은 `empty-to-nul` 한 건.

## 확대 변형 (이번 가지)

가족별로 한 줄:

| 가족 | 라벨 예 | 한 줄 |
|---|---|---|
| truncate | `truncate@10%`, `chop-last`, `cut-first`, `odd-length-chop`, `shrink-gap` | 더 짧은 꼬리·선두 삭제·홀수 길이 |
| flip | `flip@0%`, `flip@25%`, `flip@75%`, `flip@99%` | 가장자리와 사분면 |
| header | `rotate-header`, `increment-header`, `nibble-swap-head` | 매직 순서·미세 오염 |
| ole | `ole-magic-poison`, `ole-sector-shift-poison`, `ole-mini-fat-poison` | CFB 할당 입구 |
| run | `aa-run`, `nul-mid`, `00-run`, `55-run` | 포화/종료/교차 비트 |
| unicode | `utf16-bom-inject`, `utf8-overlong`, `ascii-ctrl-sprinkle`, `path-sep-sprinkle` | 인코딩·제어·경로 |
| zip | `zip-magic-inject`, `zip-cd-magic-flip`, `zip-eocd-flip` | 세 아카이브 입구 |
| length | `length-bomb@*`, `length-zero@30%`, `length-one@60%`, `i32-min@20%`, `u16-max@12` | 할당 폭주·음수 길이 |
| permute | `reverse-prefix`, `swap-ends`, `slide-window-*`, `repeat-mid-block` | 필드 정렬 |
| stripe | `high/low-bit-stripe`, `xor-stride7`, `invert-tail-64`, … | 드문드문 오염(하위 비트는 XOR) |
| splice | `splice-nul-mid`, `crlf-inject`, `pad-eof`, `widen-gap`, `even-length-pad` | 끼워 넣기. 거대 입력 생략 |
| hwp3 | `hwp3-sig-flip`, `hwp3-sig-inject` | HWP3 입구 |

거대(`>= 1MiB`)에서는 splice 가족을 건너뛴다.

## 예외 경로 — 운영자가 헷갈리는 것

| 보이는 것 | 의미 | ok? |
|---|---|---|
| `unreadables: PermissionError` | 샘플을 못 읽음 | 유지(true) |
| `probeErrors: oserror FileNotFoundError` | `--bin` 이 잘못됐거나 권한 | 유지 |
| `probeErrors: probe-error invalid-timeout` | `audit(..., timeout=0)` 직접 호출 | 유지 |
| `probeErrors: TypeError` (쓰기) | 변형이 바이트가 아님. 버그 | 유지, 도구 버그로 고친다 |
| `panics: … panicked` | rhwp 가 죽음 | **false** |
| `hangs: sample:label` | 프로브가 timeout | **false** |
| exit 2 | `find_bin` 실패 | 보고 없음 |

시그널 종료(`code < 0`)는 패닉이다. 행이 아니다.

## 로컬에서 한 샘플만 보고 싶을 때

모듈로 불러 변형만 본다. 바이너리 불필요.

```python
import importlib.util
from pathlib import Path
p = Path("gym/tools/robustness.py")
spec = importlib.util.spec_from_file_location("r", p)
m = importlib.util.module_from_spec(spec)
spec.loader.exec_module(m)
data = Path("samples/some.hwp").read_bytes()
for label, mut in m.deterministic_mutants(data):
    print(f"{label:24} {len(mut):8} {m.mutant_family(label)}")
```

감사 루프를 목킹하려면 `m.probe = lambda *a, **k: (1, False, False, "오류")` 후
`m.audit("bin", "samples", 4, 5)`.

## 시험

```bash
python -m unittest scripts.tests.test_gym_robustness
python -m unittest scripts.tests.test_gym_audit
```

`RobustnessTests` 는 기존 게이트 호환. `ExpandedMutantContractTests` 는 확대
바이트 계약. `ExceptionPathTests` 는 예외 접기. `ShapeAndSelectEdgeTests` 는
stride 와 형태.

`cargo fmt --all` 은 이 변경에 실행하지 않는다. Python 만 고친다.

## 새 라벨을 넣을 때 최소 체크

1. 라벨이 카탈로그 id 와 같거나 `id@파라미터` 형식인가.
2. `mutant_family()` 가 `other` 로 떨어지지 않는가.
3. 2KiB 픽스처에서 결정적이고 원본과 다른가.
4. ZIP/HWP3 조건부가 반대 입력에서 꺼지는가.
5. 거대 입력에서 크기가 늘지 않는가.
6. `test_gym_robustness` 가 그 바이트를 한 줄이라도 고정하는가.

자세한 표는 규약 문서 8절이다.

## 관련

- 이슈 #4814 (도구 강건성 기둥), #5218 (결정적 변형 확대)
- PR 원본: 같은 가지 `feat/gym-robust-mutants`
- 분업: `gym/tools/fuzz_corpus.py` 모듈 문자열
- gym 개요의 해당 절: `gym/README.md` 「손상-강건성 감사」
