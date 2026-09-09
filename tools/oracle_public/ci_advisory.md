# 오라클 공개 비교 CI advisory 잡

MEGA QUEUE M01-7 (#5357). 한컴 라이선스가 없는 기여자가 커밋된 한컴 PDF 오라클로
269쌍을 1커맨드 비교할 수 있게 된 뒤, 그 비교를 CI 에 **advisory** 잡으로 올리는
설계다. 이 문서는 제안이다. 병합만으로 PR 게이트를 바꾸지 않는다.

`scripts/visual_sweep.py` 는 읽거나 수정하지 않는다. gym 은 쓰지 않는다.

초안 워크플로: [`.github/workflows/oracle-public-advisory.yml`](../../.github/workflows/oracle-public-advisory.yml).
트리거는 `workflow_dispatch` 만이다. `pull_request` 블록은 주석으로만 남겨 두었다.

## 1. 목적과 비목표

### 목적

- 커밋된 단일 `pdf/` 오라클이 **실제로 있는** 러너에서
  공개 비교(`tools/oracle_public/page_smoke.py`)를 돌린다.
- 불일치는 데이터가 된다. 상위 N건을 아티팩트로 올려 사람이 본다.
- M01-6 `issue_draft.py` 가 있으면 초안 markdown 까지 디스크에 남긴다. `gh issue create`
  는 호출하지 않는다.

### 비목표

| 하지 않는 일 | 이유 |
| --- | --- |
| required check / branch protection 등록 | PR 게이트를 막지 않는다 |
| `scripts/visual_sweep.py` 호출·수정 | jangster77 금지, 픽셀 전수는 이 잡 범위 밖 |
| `--strict` | 쪽수 불일치가 있으면 종료 1 이 되어 빨간 엑스처럼 보인다 |
| gym | 이 클레임 범위 밖 |
| `issues: write` 로 이슈 자동 제출 | 제출은 사람 |
| `pdf/` 없는 포크·sparse clone 에서 실패 | 없으면 skip, 종료 0 |

## 2. 왜 advisory 인가

1. 쪽수 불일치는 이미 알려진 회귀일 수 있다. 지금 강제 게이트로 켜면 devel PR 이 한꺼번에 막힌다.
2. `pdf/` 는 메인 `ci.yml` 이 `paths-ignore` 하는 코퍼스다. 없는 checkout 이 흔하다.
3. 269쌍 × `dump-pages` 는 렌더보다 가볍지만, hang 이 나면 게이트를 잡아먹으면 안 된다.
4. 판정은 데이터다. M01-4 `page_smoke.py` 기본 종료 코드는 불일치가 있어도 0 이다.

운영 등급은 [github_operations.md](../../mydocs/manual/github_operations.md) 의 **O2
(라우팅·비용)** 이다. required check·권한 확대는 O3/O4 이고 이 제안 범위가 아니다.

## 3. 잡 계약

| 항목 | 값 |
| --- | --- |
| workflow 파일 | `.github/workflows/oracle-public-advisory.yml` |
| workflow 이름 | `Oracle Public Advisory` |
| job id / 이름 | `advisory` / `oracle-public-compare-advisory` |
| 러너 | `ubuntu-latest` |
| `timeout-minutes` | 30 (성능 목표 아님. [#3284] hang 상한과 같은 철학) |
| 내부 벽시계 | `timeout 22m` — 잡 timeout 전에 부분 리포트를 남긴다 |
| permissions | top-level `contents: read` 만 |
| concurrency | `oracle-public-advisory-${{ github.ref }}`, cancel-in-progress |
| `continue-on-error` | `true` (이중 안전장치) |
| 비교 종료 코드 | 불일치여도 0. `--strict` 금지 |
| required checks | **등록 금지**. 아래 4.3 |

현재 required context (Lint / Build & Test / CodeQL / Render Diff 등) 와
이름을 겹치지 않게 `oracle-public-compare-advisory` 를 쓴다.

## 4. 트리거

### 4.1 1단계 (이 PR 이 넣는 것)

`workflow_dispatch` 만. 입력:

| 입력 | 기본 | 의미 |
| --- | --- | --- |
| `top_n` | `10` | 아티팩트로 남길 상위 실패 건수 |
| `limit` | `0` | 비교할 최대 짝. `0` = 전수 |

PR synchronize 마다 돌지 않는다. 메인테이너가 Actions 탭에서 켠다.

### 4.2 2단계 (제안, 주석으로만)

워크플로 YAML 에 `pull_request` 블록을 주석으로 남겨 두었다. 풀 조건:

- base `devel`
- `types: [opened, reopened, synchronize]`
- `paths` 를 렌더·오라클 도구·이 YAML 로 좁힌다
- **PDF 존재 게이트를 그대로 둔다** (4.4)
- 풀어도 required check 로 등록하지 않는다
- `--strict` 는 여전히 넣지 않는다

`ci.yml` 의 `pdf/**` `paths-ignore` 와 충돌하지 않는다. 이 잡은 별도 workflow 다.

### 4.3 required 로 켜지 않는 방법

1. `on.pull_request` 를 주석 상태로 둔다 (1단계).
2. 나중에 풀더라도 ruleset / branch protection 의 `required_status_checks` 에
   `oracle-public-compare-advisory` 를 넣지 않는다. 이 변경은 O4 이고 별도 승인이다.
3. `ci.yml` `needs:` 에 이 잡을 연결하지 않는다.
4. 비교 커맨드에 `--strict` 를 넣지 않는다.
5. 잡 step 은 마지막에 `exit 0` 이다. `continue-on-error: true` 는 백업이다.

### 4.4 PDF 존재 게이트

sparse checkout 직후, 실제 PDF 시그니처(`%PDF-`)가 하나라도 있어야 비교를 시작한다.
Git LFS 포인터(`version https://git-lfs.github.com/spec/v1`) 만 있으면 **없는 것**으로 본다.

판정:

| 상태 | 동작 | 종료 |
| --- | --- | --- |
| `%PDF-` 파일 ≥ 1 | 비교 진행 | 0 (불일치 포함) |
| 디렉터리만 있거나 LFS 포인터만 | skip, step summary 에 `pdf-absent` | 0 |
| `page_smoke.py` 없음 (M01-4 미병합) | skip, `runner-absent` | 0 |
| `cargo build --bin rhwp` 실패 | skip, `rhwp-build-failed`, 부분 로그 업로드 | 0 |

모든 오라클 PDF는 일반 Git blob으로 단일 `pdf/` 트리에서 관리한다. LFS 선택 입력은 없다.

## 5. Sparse checkout

`actions/checkout` cone-mode. 루트 `Cargo.toml` · `Cargo.lock` · `rust-toolchain.toml` 은
cone 이 자동으로 포함한다.

```text
src
crates
tests
samples
pdf
tools
bindings
scripts
```

근거:

- `pdf/` 가 공개 오라클이다. `samples/` 가 짝 문서다.
- workspace members (`crates/*`, `tools/rhwp-subsecond`, `tools/batch-convert`,
  `bindings/Native`) 가 빠지면 `cargo build --bin rhwp` 가 매니페스트에서 실패한다.
- `tools/oracle_public/` 는 비교 러너·리포트 변환기다.
- 오라클은 `pdf/` 하나만 sparse checkout 하며 LFS 포인터를 허용하지 않는다.

로컬 재현은 `tools/sparse_clone_hint.py` 의 `visual-regression` 프리셋과 같다.

```text
git sparse-checkout add pdf
python tools/sparse_clone_hint.py --task visual-regression --apply
```

## 6. 비교 명령

호출 순서:

1. `tools/oracle_public/page_smoke.py` 가 없으면 skip (`runner-absent`).
2. `cargo build --release --bin rhwp`.
3. 짝 소스:
   - `tools/oracle_public/oracle_pairs.json` 이 있으면 `--manifest` (M01-1).
   - 없으면 글롭 `pdf/{stem}.pdf` · `pdf/{stem}-*.pdf` (M01-4 기본).
4. 비교 루트는 `--pdf-dirs pdf` 하나다.
5. `limit > 0` 이면 `--limit N`.
6. **`--strict` 없음.** `--json` 만.

```text
timeout 22m python tools/oracle_public/page_smoke.py \
  --rhwp target/release/rhwp \
  --json \
  --timeout 30 \
  > oracle-advisory/page-smoke.json
```

`--timeout 30` 은 짝당 `dump-pages` 상한(초)이다. 기본 180 초 × 269쌍은 잡 30분을 넘길 수 있다.

픽셀 비교·`visual_sweep.py` 는 호출하지 않는다. 쪽수 스모크가 1차 공개 비교다.

## 7. 상위 N 실패 아티팩트

작업 디렉터리 `oracle-advisory/`:

```text
oracle-advisory/
  page-smoke.json          # 전체 봉투 (schemaVersion / summary / rows)
  summary.md               # job summary 와 동일
  top-n.json               # 상위 N 실패만
  top/
    01-<stem>.json
    02-<stem>.json
    …
  drafts/                  # issue_draft.py 가 있을 때만
    manifest.json
    *.md
```

선정:

1. `verdict == MISMATCH` 를 `|delta|` 내림차순, 같으면 `stem` 오름차순.
2. 자리가 남으면 `verdict == ERROR` 를 같은 키로 채운다.
3. `N = inputs.top_n` (기본 10, 1..50 으로 clamp).

각 top 파일은 `page_smoke` 행 + `repro` 문자열을 그대로 둔다. 예:

```text
python tools/oracle_public/page_smoke.py --pair samples/foo.hwp pdf/foo-2022.pdf
```

M01-6 `issue_draft.py` 가 있으면 `failure_report/v1` 로 변환해 `--out oracle-advisory/drafts`
만 수행한다. `--submit` / `--gh` 는 넣지 않는다.

업로드:

- `actions/upload-artifact` SHA 핀 (저장소 관례 v7.0.1)
- name: `oracle-public-advisory-${{ github.run_id }}`
- `if-no-files-found: ignore` — skip 경로도 잡을 실패시키지 않는다
- `retention-days: 14`

## 8. 비용·시간

| 구간 | 대략 | 비고 |
| --- | --- | --- |
| sparse checkout + `pdf/` blob | 1–5분 | `pdf/` 는 LFS 가 아님 |
| `cargo build --release --bin rhwp` | 5–15분 | cache hit 시 수 분 |
| 269쌍 `dump-pages` | 5–15분 | 짝당 30초 상한, 벽시계 22분 |
| 잡 상한 | 30분 | hang 을 게이트로 승격하지 않음 |

전수 실행도 단일 `pdf/` 루트를 사용한다.

cache: `Swatinem/rust-cache` 의 `save-if` 는 `devel`/`main` push 가 아니다. 이 잡은
`workflow_dispatch` 이므로 restore-only 가 맞다. 캐시 키를 CI Lint 와 섞지 않는다.

## 9. 활성화·롤백

### 활성화 (메인테이너)

1. 이 설계 PR 병합.
2. 제안 이슈에서 M01-4 `page_smoke.py` 병합 여부를 확인.
3. Actions → **Oracle Public Advisory** → `workflow_dispatch` 1회.
4. 아티팩트·step summary 의 `summary.match/mismatch/error` 를 본다.
5. (선택) YAML 의 `pull_request` 주석을 푸는 후속 PR. **required 등록은 하지 않는다.**

### 롤백

- 워크플로 파일 한 개를 지우거나 `on:` 을 빈 주석만 남긴다.
- required checks 를 건드리지 않았으므로 protection rollback 은 없다.

## 10. 로컬에서 같은 계약

```text
git sparse-checkout add pdf
cargo build --release --bin rhwp
python tools/oracle_public/page_smoke.py --json > page-smoke.json
# --strict 없음
```

CI 는 위 커맨드의 상위 N 아티팩트 래퍼다. 새 비교 엔진을 이 잡에 넣지 않는다.
