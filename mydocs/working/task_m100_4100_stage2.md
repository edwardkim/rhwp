---
kind: working
status: done
canonical: mydocs/working/task_m100_4100_stage2.md
last_verified: 2026-08-11
---

# #4100 Stage 2 — 중첩 CFB 스트림 교체 (프로브 승격 + 신설)

- **계획서**: [`mydocs/plans/task_m100_4100.md`](../plans/task_m100_4100.md)
- **기준 커밋**: `devel = dd9ecdc4b`
- **산출**: `src/parser/ole_container.rs`(`all_ole_streams` 승격) ·
  `src/serializer/ole_container.rs`(신규) · `src/serializer/mod.rs`(선언) ·
  `tests/issue_4100_chart_data_edit.rs`(Stage 2 테스트 4건)

## 1. 잔량은 셋이 아니라 둘이었다

계획서 §3 의 실측대로다.

| 이슈가 부른 이름 | 착수 시점 | 이번에 한 일 |
|---|---|---|
| `ole_root_clsid` | **이미 프로덕션** (`parser/ole_container.rs:184`, #4097 이 승격) | 없음 — 그대로 쓴다 |
| `all_ole_streams` | 테스트 전용 `all_streams` | **승격** |
| `replace_ole_stream` | 이름 자체가 없음 | **신설** (`serializer/ole_container.rs`) |

프로브의 `root_clsid`(`cfb` 크레이트 오라클)는 계획대로 **승격하지 않았다.** 프로덕션으로
바꾸면 `assert_eq!(read(write(x)), read(x))` 가 공허해진다 — Stage 2 판정에서 그 오라클을
채점자로 쓴다.

## 2. 결정과 근거

### 2-1. 전수 열거 위에 선다

`parse_ole_container` 는 아는 이름 4종만 뽑는다. 그걸로 재포장하면 **나머지가 소실된다.**
차트 편집은 `OOXMLChartContents` 하나만 갈고 레거시 `Contents`(③)와 `\x02OlePres000`
EMF(④)는 원본 바이트 그대로 되실어야 하므로 전수 열거가 필요하다.

#4055 가 코퍼스 28종에서 "아는 4종 밖 스트림 0건"을 실측했지만 이름을 고정하지 않았다 —
그 관찰은 **코퍼스의 성질이지 포맷의 보장이 아니다.**

경로는 `\` → `/` 로 정규화한다. Windows 의 `cfb` 는 `/BinData\BIN0001.OLE` 처럼 구분자를
섞어 돌려주는데 반환값을 이름으로 비교하는 소비자가 있다.

### 2-2. 프로덕션이므로 panic 하지 않는다

프로브는 `expect("CFB 열기")` 였다. 승격본은 `Option`/`Result` 다 — 문서가 신뢰할 수 없는
입력이고 WASM 에서 돌기 때문이다. `cfb::CompoundFile::create()` 를 쓰지 않는 이유도 같다
(`SystemTime::now()` 가 wasm32 에서 panic — `hwp3/ole.rs` 선례).

### 2-3. 없는 스트림은 만들지 않는다

`replace_ole_stream` 은 지목한 이름이 없으면 `StreamNotFound` 로 거부한다. 새로 만들면
이름 오타가 **한컴이 못 읽는 파일**로 조용히 흘러간다.

### 2-4. 바뀐 게 없으면 재포장하지 않는다 — 이번 단계의 핵심 발견

**재포장은 바이트 동일을 보장하지 않는다.** 섹터 배치가 원본 작성기(한컴)와 다르다.
그래서 새 바이트가 기존과 같으면 **원본을 그대로 돌려주는 짧은 회로**를 넣었다.

이게 없으면 수용 기준 2("무편집 CSV 왕복이 바이트 동일")가 **값을 하나도 안 바꿔도**
깨진다 — CSV 를 그대로 먹였을 뿐인데 `.ole` 이 통째로 달라진다. 스캐너·패처를 아무리
정확히 만들어도 이 층에서 무너지는 실패라, 판정을 코퍼스 56건으로 고정했다
(`unchanged_stream_content_skips_the_repack_entirely`).

## 3. 판정

### 3-1. 코퍼스 56건 — 전건 green

| 테스트 | 무엇을 고정하나 |
|---|---|
| `repack_preserves_every_stream_and_leaves_the_others_byte_identical` | 스트림 **집합**이 보존되고, `OOXMLChartContents` 밖 스트림은 바이트 동일. 이름을 고정하지 않고 집합으로 판정한다 |
| `repack_preserves_the_root_class_id` | 루트 CLSID 보존. **판정은 `cfb` 크레이트 오라클** — rhwp 의 `ole_root_clsid` 로만 재면 읽기·쓰기가 같은 오프셋 오해를 공유해도 통과한다. 원본 CLSID 가 0 이 아님을 먼저 단언해 판정이 공허해지지 않게 했다 |
| `unchanged_stream_content_skips_the_repack_entirely` | 무편집이면 중첩 CFB 바이트가 그대로 (§2-4) |
| `repack_refuses_to_invent_a_missing_stream` | 없는 이름은 거부 |

단위 8건(`serializer::ole_container`)도 green — 스트림 교체·CLSID 보존·짧은 회로·
슬래시 무관 이름·비 CFB 입력 거부·CLSID 0 유지.

### 3-2. 게이트

```text
cargo fmt --check                            Diff in 0건
cargo clippy --all-targets -- -D warnings    exit 0
```

## 4. 다음 (Stage 3)

주소 `(section, para, control)` → ①② 슬롯 해석 + `get_chart_data_native`.
위험 R2(`bin_data_id` 가 보통 1-based 인덱스)와 R5(`--chart N` 번호 체계)가 거기서 걸린다.
