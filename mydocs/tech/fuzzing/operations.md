---
kind: guide
status: active
canonical: mydocs/tech/fuzzing/agent_surface_robustness.md
last_verified: 2026-08-03
---

# 퍼징 운영 — 어떻게 돌리고 무엇을 보나

> 인프라 자체의 1차 출처는 저장소 안의 [`fuzz/README.md`](../../../fuzz/README.md) 다.
> 이 문서는 그것을 복제하지 않고, **운영 판단**을 더한다 — 얼마나 돌릴지, 코퍼스를
> 어떻게 키울지, CI 에 넣을지, 이 환경에서 되는지, 나온 걸 누가 받는지.
> 묶음 지도는 [README.md](README.md), 크래시 처리는 [crash_triage.md](crash_triage.md).

측정 기준선: **2026-08-03**, `rhwp v0.8.2`(`Cargo.toml:3`), HEAD `9095cd52d`.

---

## 0. 먼저 — 퍼징이 이 저장소에서 무엇을 검출하는가

RFC [#3141](https://github.com/edwardkim/rhwp/issues/3141) §1 이 세 가지 결함 클래스를
지목했고, 하네스 6개는 그 세 가지만 본다.

| 클래스 | 어떻게 검출되나 | 저장소의 실제 사례 |
| --- | --- | --- |
| **패닉 / abort** | libFuzzer 가 프로세스 종료를 크래시로 기록 | #3311 (`cfb_reader.rs:407` OOB 슬라이스) |
| **자원 고갈(OOM)** | `-rss_limit_mb` 초과 시 크래시 | #2743 (HML `Id` → 382B 파일이 120MB, 385B 가 240GB 요구 후 abort) |
| **무한루프 / 타임아웃** | `-timeout` 초과 시 크래시 | #3012 (`parse_polygon_shape_data` 부호확장) |

**검출하지 않는 것**(RFC §9):

- 렌더링 정합성·왕복(round-trip) 속성 소실 — #2740 영역이다. 하네스가 `Err` 를 버리므로
  "틀린 결과를 조용히 내는" 결함은 원리상 보이지 않는다.
- **봉투가 잘못된 판정을 내는 것** — 하네스는 파서 함수만 부르고 CLI 봉투를 만들지 않는다.
  이 공백이 [agent_surface_robustness.md](agent_surface_robustness.md) 의 주제다.

하네스 6개는 전부 반환값을 버린다:

```rust
fuzz_target!(|data: &[u8]| {
    let _ = rhwp::parser::parse_hwp(data);
});
```
— `fuzz/fuzz_targets/parse_hwp.rs`

**`Err` 는 성공이다.** 손상 입력이 `Err` 로 끝나는 것이 계약이고, 퍼저가 잡는 건
프로세스가 죽거나 안 끝나는 경우뿐이다.

---

## 1. 준비 — 툴체인

```sh
rustup toolchain install nightly
cargo install cargo-fuzz
```

cargo-fuzz 는 `-Z sanitizer` 계열 플래그를 쓰므로 **nightly 가 필수**다
(`fuzz/README.md` §사전 준비).

이 저장소는 `rust-toolchain.toml` 로 **stable 1.93.1 을 핀**한다.

```toml
[toolchain]
channel = "1.93.1"
components = ["clippy", "rustfmt"]
targets = ["wasm32-unknown-unknown"]
profile = "minimal"
```

따라서 퍼징 명령에는 **반드시 `+nightly` 를 붙인다** — rustup 의 우선순위에서
명령행 `+toolchain` 이 `rust-toolchain.toml` 핀보다 위이기 때문이다.
`+nightly` 를 빼면 1.93.1 로 떨어지고 cargo-fuzz 가 sanitizer 플래그에서 실패한다.
(이 저장소에서 실제로 실행해 확인한 것은 아니다 — §6 참조. **확인되지 않음**.)

---

## 2. 실행

### 2-1. 빌드만 (배선 점검용)

```sh
cargo +nightly fuzz build
```

하네스가 참조하는 `pub` 경로가 실제로 존재하는지, 6개가 전부 링크되는지 확인한다.
`fuzz/Cargo.toml` 의 `[[bin]]` 개수(6)와 `fuzz/fuzz_targets/*.rs` 개수(6)가 어긋나면
여기서 깨진다.

### 2-2. 개별 타깃 실행

```sh
cargo +nightly fuzz run parse_hwp          -- -rss_limit_mb=2048 -timeout=30
cargo +nightly fuzz run parse_hwp3         -- -rss_limit_mb=2048 -timeout=30
cargo +nightly fuzz run parse_hwpx         -- -rss_limit_mb=2048 -timeout=30
cargo +nightly fuzz run parse_hml          -- -rss_limit_mb=2048 -timeout=30
cargo +nightly fuzz run parse_wmf          -- -rss_limit_mb=2048 -timeout=30
cargo +nightly fuzz run parse_ooxml_chart  -- -rss_limit_mb=2048 -timeout=30
```

`--` 뒤는 libFuzzer 인자다. cargo-fuzz 는 `fuzz/corpus/<타깃>/` 을 코퍼스 디렉터리로
자동 사용하고, 크래시 산출물을 `fuzz/artifacts/<타깃>/` 에 쓴다
(둘 다 `fuzz/.gitignore` 에서 `artifacts/` 만 제외 — 코퍼스는 커밋 대상).

### 2-3. 플래그를 그 값으로 쓰는 이유

| 플래그 | 값 | 왜 |
| --- | --- | --- |
| `-rss_limit_mb` | `2048` | #2743 류 **무검증 할당**을 OOM 크래시로 승격시킨다. libFuzzer 기본값도 2048 이지만, 값이 아니라 **의도**를 남기려고 항상 명시한다 |
| `-timeout` | `30` | #3012 류 **부호확장 무한루프**를 잡는다. 기본값 1200초는 이 용도로 너무 길다 — 20분간 한 입력에 묶여 있으면 그 세션은 사실상 죽은 것이다 |
| `-jobs` / `-workers` | 코어 수 | 병렬 실행. 코퍼스 디렉터리를 공유하므로 발견 입력이 서로에게 즉시 전파된다 |
| `-max_len` | (기본) | 포맷 파일은 헤더 제약이 강해 짧은 입력이 얕게 끝난다. 시드가 있으면 굳이 조이지 않는다 |

**`-rss_limit_mb` 를 크게 잡지 마라.** 값을 키우면 "죽지는 않는" 상태가 되어
#2743 류가 검출되지 않는다. #2743 은 `Id="1000000"` 에서 **`Ok` 를 반환하면서**
120MB 를 먹었다 — 상한이 없으면 퍼저에게 보이지 않는 결함이다
(`tests/issue_2743_hml_resource_id_limit.rs` 주석의 "조용한 구간").

### 2-4. 단건 재현 (크래시 입력 하나만)

```sh
cargo +nightly fuzz run parse_hwp fuzz/artifacts/parse_hwp/crash-<해시>
```

파일 인자를 주면 퍼징하지 않고 그 입력 하나만 실행한다. 자세한 절차는
[crash_triage.md §2](crash_triage.md).

---

## 3. 코퍼스 — 어디서 오고 어떻게 키우나

### 3-1. 지금 있는 시드 (실측 `ls fuzz/corpus/*/`)

| 코퍼스 | 파일 | 크기 | 출처 |
| --- | --- | ---: | --- |
| `parse_hwp` | `english.hwp` · `Textmail.hwp` · `shortcut.hwp` | 29K·34K·42K | `samples/basic/` |
| `parse_hwp3` | `hwp3-pagedef-1915.hwp` · `hwp3-sample.hwp` | 2.4K·87K | `samples/` |
| `parse_hwpx` | `neartop_reset_sb2500.hwpx` · `saved_single_line_spacing_after.hwpx` · `tac-host-spacing.hwpx` | 3.7K·3.7K·4.1K | `samples/task2136` · `samples/task2093` · `samples/` |
| `parse_hml` | `exambank_math_equations_min.hml` · `formatting_table.hml` | 4.0K·29K | `tests/fixtures/hml/` · `samples/hml/` |
| `parse_wmf` | `minimal_placeable.wmf` | 46B | **합성** — META_PLACEABLE(0x9AC6CDD7) + 최소 헤더 + META_EOF |
| `parse_ooxml_chart` | `bar_chart.xml` | 782B | **합성** — 최소 `c:chartSpace` 막대 차트 |

합계 12개 / 269KB (`du -sh fuzz/corpus`).

### 3-2. `samples/` 를 시드로 쓸 수 있나 — 쓸 수 있고, 그게 정답이다

`samples/` 에는 **`.hwp`/`.hwpx` 353개, 총 448MB** 가 이미 커밋돼 있다
(`ls samples/*.hwp samples/*.hwpx | wc -l` = 353, `du -sh samples` = 448M).
시드 코퍼스 12개는 그중 **작은 것만 고른 부분집합**이다.

CFB/ZIP 컨테이너 포맷은 구조 제약이 강해서 **시드 없이는 변이가 헤더를 못 넘는다**
(RFC #3141 §9). 즉 시드가 커버리지를 사실상 결정한다. 그런데 코퍼스를 무작정 키우면
libFuzzer 는 매 사이클마다 전체 코퍼스를 실행하므로 **초당 실행 횟수(exec/s)가 떨어진다.**
둘 사이의 운영 규칙:

1. **코퍼스에 넣기 전에 `cmin` 을 돌린다.** 커버리지가 같은 파일은 하나만 남긴다.
   ```sh
   cargo +nightly fuzz cmin parse_hwp
   ```
2. **큰 파일을 그냥 넣지 않는다.** 448MB 를 통째로 넣으면 exec/s 가 무너진다.
   먼저 작은 것부터(수 KB~수십 KB), 커버리지가 정체되면 다음 크기대를 추가한다.
3. **로컬 실험용 코퍼스는 별도 디렉터리를 쓴다.** `cargo fuzz run <타깃> <디렉터리>` 로
   임시 코퍼스를 지정할 수 있고, 커밋 대상인 `fuzz/corpus/` 를 오염시키지 않는다.
4. **커밋하는 것은 "커버리지를 늘린 최소화 입력"만.** `fuzz/README.md` §시드 코퍼스가
   정한 규약이다.

시드 후보 우선순위 — 파서 분기를 많이 여는 순:

| 우선 | 무엇 | 왜 |
| --- | --- | --- |
| 1 | 임베드 객체가 든 문서(WMF·차트·OLE) | `parse_wmf`/`parse_ooxml_chart` 경로가 컨테이너 밖에서만 열림 |
| 2 | 표·중첩표 문서 | `parse_control` 아래 `control/shape.rs` — #3012 결함 위치 |
| 3 | HWP 3.x 실물 | 스케일 곱셈 오버플로가 몰린 곳(§5 참조) |
| 4 | 암호·DRM 문서 | 별도 경로. 다만 복호화 실패가 대부분이라 얕게 끝남 |

### 3-3. 회귀 코퍼스는 아직 없다

`fuzz/README.md` §트리아지는 최소화한 크래시 입력을 `fuzz/corpus/` 가 **아니라**
`fuzz/regressions/<타깃>/` 에 커밋하라고 정하는데, 그 디렉터리는 **존재하지 않는다**.

```
$ ls fuzz/regressions
ls: cannot access 'fuzz/regressions': No such file or directory
```

즉 규약만 있고 실물이 없다. 첫 크래시가 나오는 순간 [crash_triage.md §8](crash_triage.md)
절차로 생성한다. #3608 M21 체크리스트의 "크래시 코퍼스 회귀 스위트 편입"이 이 항목이다.

---

## 4. 얼마나 돌려야 의미가 있나

libFuzzer 는 종료 조건이 없다 — 사람이 끊어야 한다. 판단 근거를 세 가지로 나눈다.

### 4-1. 저장소가 가진 유일한 실측 기준점

이슈 #3311 의 제보자가 남긴 값이 지금으로선 유일한 실측치다.

> "발견: cargo-fuzz(libFuzzer) **4분 런**, 정부 공개 HWP/HWPX 시드에서 mutate"
> — #3311 §환경

**4분에 패닉 하나.** 이건 "퍼징이 금방 성과를 낸다"는 뜻이 아니라, **당시 코드에
얕은 결함이 있었다**는 뜻이다. 같은 4분이 지금도 뭔가를 낸다는 보장은 없다.

### 4-2. 세션 종료 판단 — 시계가 아니라 지표로

libFuzzer 가 매 줄에 찍는 지표를 본다.

```
#65536  pulse  cov: 4312 ft: 8901 corp: 143/1247Kb exec/s: 812 rss: 210Mb
```

| 지표 | 의미 | 어떻게 쓰나 |
| --- | --- | --- |
| `cov` | 도달한 커버리지 포인트 | **이게 안 늘면 그 세션은 끝난 것** |
| `ft` | feature 수(경로 조합) | `cov` 가 멈춰도 `ft` 가 늘면 아직 진행 중 |
| `corp` | 코퍼스 항목 수 / 크기 | 새 항목이 안 생기면 정체 |
| `exec/s` | 초당 실행 | 급락하면 코퍼스가 비대하거나 느린 입력에 물린 것 |
| `rss` | 상주 메모리 | `-rss_limit_mb` 에 근접하면 OOM 결함 후보 |

**운영 규칙: `cov` 와 `corp` 가 30분 동안 변하지 않으면 그 타깃은 그 코퍼스로
포화한 것이다.** 더 돌리는 대신 (ㄱ) 시드를 추가하거나 (ㄴ) 더 깊은 하네스를
만드는 편이 낫다.

### 4-3. 권장 예산

| 상황 | 타깃당 시간 | 목적 |
| --- | --- | --- |
| PR 스모크 | 60초 | **회귀 검출만.** 새 결함 발견은 기대하지 않는다 |
| 파서 변경 후 로컬 | 10~30분 | 방금 만진 경로의 얕은 결함 |
| 주간 스윕 | 타깃당 2~4시간 | `cov` 정체까지 |
| 상시 | 무기한 | OSS-Fuzz 등재 시 Google 인프라가 담당(RFC §7) |

RFC §6-5 가 제안한 CI 스모크가 "PR당 60초 × 하네스"인 것도 같은 논리다 —
CI 퍼징의 목적은 **발견이 아니라 회귀 차단**이다.

---

## 5. 퍼징이 실제로 무엇을 잡았나 / 무엇을 잡을 수 있었나

### 5-1. 잡았다 — #3311 (유일한 확인 사례)

`tests/issue_3311_malformed_cfb_no_panic.rs` 상단 주석이 출처를 명시한다.

> "외부 리포터(**cargo-fuzz**, 격리 CDR 파이프라인 하드닝 중)가 `LenientCfbReader::open`
> 의 OOB 슬라이스 패닉을 보고했다(`cfb_reader.rs:407`, "range end index 8020 out of
> range for slice of length 3072")."

경위:
- 제보(#3311, 2026-07-17 커밋 `8d3bfa4b` 기준) — 외부 사용자가 **자기 파이프라인에서**
  cargo-fuzz 를 돌리다 발견. 이 저장소의 `fuzz/` 는 그 시점에 아직 없었다(`e78883617` = 07-24).
- 결함 자체는 `6a761a793`(#3220, 07-24)에서 **다른 작업으로 이미 해소**돼 있었다.
- `635d620cc`(#3311, 08-01)가 **계약으로 못박았다** — 177 케이스 무패닉 가드.

교훈이 두 개다. ① 퍼징은 실제로 이 코드베이스에서 결함을 냈다. ② 그러나 그때
퍼저를 돌린 건 우리가 아니라 **밖의 누군가**였다.

### 5-2. 잡을 수 있었다 — 손으로 찾은 결함 13건

`git log --oneline` 에서 "무검증 할당·부호확장·오버플로·무한루프" 계열 수정만 추린 것이다.
전부 **사람이 코드를 읽어서** 찾았다(RFC §1 의 핵심 관찰과 동일).

| 커밋 | 날짜 | 결함 | 클래스 | 퍼저가 잡았을까 |
| --- | --- | --- | --- | --- |
| `905b3261e` | 07-19 | lenient CFB 파서가 손상 헤더 값에 패닉 | 패닉 | **예** — `parse_hwp` |
| `12ecfece6` | 07-20 | TAB_DEF·연결선 제어점 무제한 할당·무한루프 | OOM/루프 | **예** — `parse_hwp` |
| `6b29fa1da` | 07-20 | 확장 레코드 `pos+size` 오버플로가 경계 검사 무력화(wasm32) | 오버플로 | 부분 — wasm32 전용 경로 |
| `821d84465` | 07-22 | 표 그리드 68GB · HML 리소스 Id 선형 비례 (#2722·#2743) | OOM | **예** — `-rss_limit_mb` 로 |
| `55e5ba086` | 07-22 | Region `scan_count`(i16) 음수 부호확장 → capacity overflow (#3004) | 부호확장 | **예** — `parse_wmf` |
| `3202e746b` | 07-22 | DIB `colors_used`(u32) 무상한 `with_capacity` (#3000) | 무상한 | **예** — `parse_wmf` |
| `6a761a793` | 07-24 | 악성 입력 무한루프·오버플로 방어 6건 (#3220) | 혼합 | **예** — 6건 중 CFB·WMF 다수 |
| `1b02247ff` | 07-25 | WMF/CFB 메모리 안전 3건 (#3301) | 혼합 | **예** — `parse_wmf` |
| `cdd55c838` | 08-02 | HWP3 셀 패딩 `read_i16 as u32 * 4` 부호확장 | 부호확장 | **예** — `parse_hwp3` |
| `e288b0a7f` | 08-02 | HWP3 표/그림 여백 `i16 * 4` 오버플로 | 오버플로 | **예**(debug 빌드) |
| `6bcbadcd1` | 08-02 | HWP3 drawing margin/size/offset 스케일 곱셈 오버플로 | 오버플로 | **예**(debug 빌드) |
| `77627e953` | 08-02 | HWP3 셀 안여백 `u32` 곱셈 오버플로 | 오버플로 | **예**(debug 빌드) |
| `ed339e78f` | 08-02 | HWP5 `row_span`/`col_span` 0 → `u16` 언더플로 패닉 | 언더플로 | **예** — `parse_hwp` |

**"예"가 12건이다.** 퍼징 인프라는 07-24 에 들어왔는데, 그 뒤로도(08-02) 같은 클래스가
5건 손으로 잡혔다. 인프라가 있는 것과 **돌아가는 것**은 다르다는 증거다.

> ⚠️ 오버플로 3건(`e288b0a7f`·`6bcbadcd1`·`77627e953`)은 **debug 빌드에서만 패닉**한다
> (release 는 wrap). cargo-fuzz 는 기본적으로 디버그 어서션을 켠 채 빌드하므로 검출
> 대상이지만, 릴리스 프로파일로 빌드하면 조용히 틀린 값이 된다 — 그건 퍼징이 아니라
> 왕복 정합성의 영역이다.

### 5-3. 아직 남아 있다 — 지금 코드에 있는 후보 2곳 (HEAD `9095cd52d` 실측)

`fuzz/regressions` 도 없고 퍼저도 안 돌았으니, **RFC 가 지목한 클래스가 아직 코드에
남아 있는지** 직접 확인했다. 남아 있다.

**후보 ①** — `src/wmf/parser/records/drawing/poly_line.rs:47`

```rust
let (number_of_points, number_of_points_bytes) =
    crate::wmf::parser::read_i16_from_le_bytes(buf)?;   // i16 — 음수 가능
record_size.consume(number_of_points_bytes);

let mut a_points = Vec::with_capacity(number_of_points as usize);   // ← 부호확장
```

`number_of_points` 는 MS-WMF 정의상 **signed 16-bit** 이고(같은 파일 15~17행 주석),
`read_i16_from_le_bytes` 는 `impl_from_le_bytes!` 매크로가 생성한 `(i16, usize)` 반환
함수다(`src/wmf/parser/mod.rs:73-89`). 음수면 `as usize` 가 부호확장돼 `usize::MAX` 근처가
되고 `Vec::with_capacity` 가 capacity overflow 로 패닉한다.

**같은 결함이 `src/wmf/parser/records/drawing/polygon.rs:50` 에도 있다** (동일 형태).

이게 오탐이 아닌 근거: **동일 패턴이 이미 고쳐진 자리**가 바로 옆에 있다.
`src/wmf/parser/objects/graphics/region.rs:96` 은

```rust
if scan_count < 0 {
    ... cause: format!("The scan_count field `{scan_count}` must not be negative"),
```

로 막고, 같은 파일 7~28행의 인라인 테스트가 기전을 그대로 적어 놓았다 —
"`scan_count`에 -1(0xFFFF)을 넣으면 `as usize` 부호확장으로 `Vec::with_capacity`가
usize::MAX 근처 값을 요청해 capacity overflow". 그 수정이 `55e5ba086`(#3004)이고,
`poly_line`/`polygon` 두 곳은 그때 함께 손보지 않았다.

**후보 ②** — `src/wmf/parser/objects/structure/bitmap16.rs:126`

```rust
pub fn calc_length(&self) -> usize {
    ((((self.width * self.bits_pixel as i16 + 15) >> 4) << 1) * self.height) as usize
}
```

`width`·`height` 는 `i16`(같은 파일 13·16행). i16 끼리 곱하므로 debug 빌드에서
곱셈 오버플로 패닉, 음수면 `as usize` 부호확장. 반환값은
`Bitmap16::parse` 가 `read_variable(buf, bitmap.calc_length())` 로 **읽기 길이**에
그대로 쓴다(56~60행).

**두 후보 모두 `parse_wmf` 하네스의 사정거리 안이다.** `META_POLYLINE::parse` 는
`src/wmf/converter/mod.rs:224` 에서, `META_POLYGON::parse` 는 같은 파일 230행에서
`WMFConverter::run()` 의 레코드 디스패치가 직접 부른다 — 하네스 진입점이 바로 그
`run()` 이다(`fuzz/fuzz_targets/parse_wmf.rs`).

이 후보들은 #3273 처리 문서(`mydocs/report/task_m100_3273_report.md` §후속)가 이미
"퍼징 실행 시 자동 재현될 것으로 기대"라고 적어 둔 항목이다. **1년 가까이 기대만 남아
있다** — 퍼저를 한 번도 안 돌렸기 때문이다.

> 이 문서는 조사 문서이므로 여기서 수정하지 않는다. 확인된 후보는
> [crash_triage.md §7](crash_triage.md) 의 이슈→수정 절차를 따른다.

---

## 6. 이 PC 에서 퍼징이 되나 — **안 된다** (실측)

Windows 11 / `<저장소>` 워크트리 기준. 세 단계에서 막힌다.

### 6-1. nightly 툴체인이 없다

```
$ rustup toolchain list
stable-x86_64-pc-windows-gnu
stable-x86_64-pc-windows-msvc (default)
1.93.1-x86_64-pc-windows-msvc (active)
```

nightly 없음. `cargo +nightly fuzz build` 는 첫 줄에서 멈춘다.

### 6-2. cargo-fuzz 가 설치돼 있지 않다

```
$ cargo fuzz --version
error: no such command: `fuzz`
help: a command with a similar name exists: `fix`
```

`~/.cargo/bin/` 실측에도 `cargo-fuzz.exe` 없음(있는 것: cargo-clippy, cargo-fmt,
cargo-miri, rustfmt 등 14개).

### 6-3. 근본 원인 — MSVC 링커가 깨져 있다

이건 설치로 해결되지 않는다. 최소 크레이트 하나를 핀된 툴체인으로 링크해 봤다.

```
$ rustup run 1.93.1 cargo build     # 빈 hello-world 크레이트
error: linking with `link.exe` failed: exit code: 1123
  = note: CVTRES : fatal error CVT1107:
      'C:\Program Files (x86)\Windows Kits\10\lib\10.0.26100.0\um\x64\dbghelp.lib'
      이(가) 손상되었습니다.
    LINK : fatal error LNK1123: COFF로 변환하는 동안 오류가 발생했습니다.
```

**Windows SDK 의 `dbghelp.lib` 자체가 손상**됐다. rustc 는 이 라이브러리를 모든
MSVC 타깃 바이너리에 무조건 링크하므로, 이 PC 에서 **MSVC 타깃 실행 파일은
어떤 것도 만들 수 없다.** cargo-fuzz 는 실행 파일을 만드는 도구다.

### 6-4. GNU 툴체인은 링크된다 — 그러나 우회가 되지 않는다

```
$ rustup run stable-x86_64-pc-windows-gnu cargo build --target x86_64-pc-windows-gnu
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.66s
```

GNU 타깃은 정상 링크된다. 다만 libFuzzer 는 sanitizer 런타임에 의존하고,
`x86_64-pc-windows-gnu` 에 대한 rustc sanitizer 지원 여부는 **이 환경에서 확인하지 못했다**
(nightly 가 없어 `-Z sanitizer` 를 시험할 수 없다). **확인되지 않음.**

`fuzz/README.md` 가 적어 둔 Windows 우회

```sh
RUSTFLAGS="-C linker=rust-lld" cargo +nightly fuzz build
```

역시 **이 PC 에서 검증하지 못했다** — 그 앞 단계(6-1)에서 막히기 때문이다. 다만 이
우회가 문서에 적혀 있다는 사실 자체는, 과거 누군가 Windows 에서 같은 벽에 부딪혔다는
방증이다.

### 6-5. 결론 — 이 환경에서의 운영 방침

| 하고 싶은 일 | 이 PC 에서 | 대안 |
| --- | --- | --- |
| 하네스 배선 점검 | ✗ | 코드 리뷰 + `[[bin]]`↔파일 개수 대조(§2-1) |
| 실제 퍼징 | ✗ | Linux/macOS, 또는 CI(§7), 또는 OSS-Fuzz |
| 크래시 재현 | ✗ | 최소화 입력을 **회귀 테스트로 승격**하면 `cargo test` 로 확인 가능 — 그 테스트는 MSVC 링크가 필요하므로 이 PC 에선 여전히 불가 |
| 코퍼스 정리(`cmin`) | ✗ | 퍼징 가능 환경에서 |
| 코드 근거 조사 | ✓ | §5-3 이 그 방식으로 후보 2곳을 찾았다 |

**이 환경의 유일한 검증 수단은 CI 다.** 이는 퍼징만의 문제가 아니라 이 워크트리
전반의 조건이다.

---

## 7. CI 에서 도는가 — **돌지 않는다** (수동 전용)

### 7-1. 실측

```
$ grep -ril "fuzz" .github/
(출력 없음)
```

`.github/workflows/` 의 워크플로 12개 — `ci.yml` · `codeql.yml` · `cache-generation-sweep.yml` ·
`cancel-stale-pr-runs.yml` · `close-issues-on-devel-push.yml` · `deploy-pages.yml` ·
`full-renderer-sweep.yml` · `node-binding.yml` · `npm-publish.yml` · `python-binding.yml` ·
`release-binary.yml` · `render-diff.yml` — **어디에도 `fuzz` 문자열이 없다.**

`ci.yml` 의 잡 8종도 마찬가지다: `preflight` · `build-test-archive` · `test-shard`(8샤드) ·
`lint`(fmt/clippy/WASM check) · `native-skia-tests` · `frontend-package-gates` ·
`build-and-test` · `wasm-build`.

즉 **퍼징은 100% 수동**이고, 지금까지 이 저장소에서 정기적으로 돌았다는 기록은 없다.
`fuzz/` 디렉터리를 건드린 커밋은 3개뿐이다(`git log -- fuzz/`): `489319d2c` ·
`e78883617` · `2b87440ea` — 전부 인프라 도입 커밋이고, **크래시 유입 커밋은 0건**이다.

### 7-2. 인접한 것 — CodeQL 은 돈다

`.github/workflows/codeql.yml` 이 정적 분석을 돌린다. 다만 CodeQL 의 Rust 지원 범위와
이 저장소 설정에서 무엇을 보는지는 **확인하지 않았다**. 퍼징과 겹치지 않는 축으로 두고,
필요하면 별도로 조사한다.

### 7-3. 넣는다면 — 두 단계로 나눈다

RFC §6-5 는 "PR당 60초 × 하네스" 또는 "회귀 코퍼스 재생만이라도"라고 두 선택지를 준다.
비용·효용을 실제 CI 구조에 맞춰 갈라 보면 순서가 분명하다.

**단계 A — 회귀 코퍼스 재생 (먼저, 싸다)**

- 하는 일: `fuzz/regressions/<타깃>/` 의 입력을 **각각 1회씩** 실행. 변이 없음.
- 비용: 입력 수 × 수 ms. `test-shard` 에 얹으면 사실상 공짜다.
- 얻는 것: **고친 결함의 재유입 차단.** 지금 `test-shard` 가 하는 일과 성격이 같다.
- 전제: `fuzz/regressions/` 가 생겨야 한다(§3-3). 그리고 이 경로는 **cargo-fuzz 없이도**
  구현 가능하다 — 회귀 입력을 읽어 `parse_*` 를 부르는 평범한 `#[test]` 면 된다.
  `tests/issue_3311_malformed_cfb_no_panic.rs` 가 정확히 그 모양이다.
- **권장: 이 형태로 먼저 넣는다.** nightly·cargo-fuzz 를 CI 에 들이지 않고
  회귀 차단의 90%를 얻는다.

**단계 B — 스모크 퍼징 (나중에, 비싸다)**

- 하는 일: 타깃 6개 × 60초 변이 실행.
- 비용: 러너 시간 6분 + nightly 설치 + cargo-fuzz 빌드(캐시 필요).
- 얻는 것: 새 결함의 조기 발견 — 다만 60초는 §4-1 기준으로도 얕다.
- 리스크: **비결정적 실패.** 퍼징은 매 실행이 다르므로 "관계없는 PR 에서 빨간불"이
  생긴다. 그 순간 팀은 게이트를 무시하기 시작한다. 넣는다면
  `workflow_dispatch` + 야간 스케줄로 두고 **PR 게이트로 삼지 않는 것**이 안전하다.
  (`full-renderer-sweep.yml` 이 이미 그런 무거운 스윕 잡의 선례다.)

**단계 C — OSS-Fuzz (RFC §7)**

상시 퍼징은 근본적으로 우리 CI 가 할 일이 아니다. RFC 가 지적한 대로 등재 요건
(MIT 라이선스 — 충족)은 이미 갖췄고, 남는 건 트리아지 부담을 감당할지에 대한
**메인테이너 판단**이다. 등재 상태는 **확인되지 않음**([README.md §5](README.md)).

---

## 8. 커버리지 측정

```sh
cargo +nightly fuzz coverage parse_hwp
```

`fuzz/.gitignore` 가 `coverage/` 를 제외 대상으로 이미 잡아 두었다 — 즉 이 명령이
쓰는 산출 경로를 상정한 설계다. 실제로 실행된 적이 있는지는 **확인되지 않음**
(`fuzz/corpus` 외 산출물이 저장소에 없다).

커버리지를 볼 때의 질문은 하나다: **컨테이너를 뚫었나?**
`parse_hwp` 코퍼스가 `cfb_reader.rs` 만 훑고 `body_text.rs`·`control.rs` 에 못 닿았다면,
RFC §4 의 2순위 하네스(`parse_body_text_section`·`parse_doc_info`·`parse_control`)를
만드는 편이 시드를 더 넣는 것보다 낫다.

---

## 9. 무엇이 나오면 어디로 가나

```
크래시 산출물 (fuzz/artifacts/<타깃>/)
        │
        ├─▶ [crash_triage.md] 최소화 → 재현 → 층 판별
        │
        ├─▶ 이슈 등록 (기여 절차: 이슈 → 분석 → 수정 → 처리결과 문서 → PR)
        │
        ├─▶ 수정 PR + 회귀 테스트 (tests/issue_####_*.rs)
        │
        └─▶ 최소화 입력을 fuzz/regressions/<타깃>/ 에 커밋
```

보안 성격 판단이 필요하면 [`SECURITY.md`](../../../SECURITY.md) 와
[agent_security/disclosure.md](../agent_security/disclosure.md) 를 따른다 —
**공개 이슈로 먼저 올리지 않는다.** 단, 파서 크래시(DoS)는 #3311 처럼 공개 이슈로
접수된 선례가 있으므로, 원격 코드 실행급이 아니면 공개 이슈가 관행이다.

---

## 10. 운영 체크리스트

퍼징 세션 하나를 돌리기 전/후에 확인할 것.

**전**
- [ ] `cargo +nightly fuzz build` 통과 — 6개 타깃 전부
- [ ] `fuzz/corpus/<타깃>/` 에 시드가 있는가 (빈 코퍼스는 헤더도 못 넘는다)
- [ ] `-rss_limit_mb=2048 -timeout=30` 을 붙였는가
- [ ] 디스크 여유 — `fuzz/target/` 과 코퍼스 증식이 GB 단위로 늘 수 있다
      (`e288b0a7f` 커밋 메시지가 "C: 드라이브 상시 포화"로 테스트를 완주 못 한 기록을 남겼다)

**후**
- [ ] `cov`/`corp` 가 늘었는가 — 안 늘었으면 시드나 하네스를 바꿀 때다
- [ ] 새로 늘어난 코퍼스 항목을 `cmin` 으로 줄였는가
- [ ] 크래시가 있으면 [crash_triage.md](crash_triage.md) 로
- [ ] 실행 조건(커밋 해시·타깃·시간·exec/s·최종 cov)을 어딘가에 남겼는가 —
      **지금 이 저장소에는 그런 기록이 하나도 없다**(§7-1)

---

## 11. 확인되지 않음

| 항목 | 이유 |
| --- | --- |
| `cargo +nightly fuzz build` 의 실제 통과 여부 | 이 PC 실행 불가(§6). #3158·#3273 처리 문서도 같은 이유로 미검증이라 기록 |
| `+nightly` 가 `rust-toolchain.toml` 핀을 실제로 이기는가 | rustup 문서상 그렇지만 이 저장소에서 실행 확인 안 함 |
| windows-gnu 타깃의 sanitizer/libFuzzer 지원 | nightly 부재로 시험 불가 |
| 지금까지의 누적 퍼징 시간·발견 건수 | 저장소에 기록 없음. `fuzz/artifacts/` 는 gitignore |
| OSS-Fuzz 등재 신청 상태 | 저장소 밖 정보 |
| CodeQL 이 이 저장소에서 무엇을 보는가 | `codeql.yml` 내용 미조사 |
| §5-3 후보 2곳이 **실제로** 퍼저에서 재현되는가 | 코드 근거만 확인. 실행 재현은 못 함 |

## 관련

- [README.md](README.md) — 이 묶음 지도 · [crash_triage.md](crash_triage.md) · [agent_surface_robustness.md](agent_surface_robustness.md)
- [`fuzz/README.md`](../../../fuzz/README.md) — 실행 명령의 1차 출처
- [agent_security/threat_model.md](../agent_security/threat_model.md) §1.1 — 신뢰 경계 ①/②
- 이슈: [#3608](https://github.com/edwardkim/rhwp/issues/3608) M21 · [#3141](https://github.com/edwardkim/rhwp/issues/3141) RFC · [#3311](https://github.com/edwardkim/rhwp/issues/3311) 실제 발견 사례
