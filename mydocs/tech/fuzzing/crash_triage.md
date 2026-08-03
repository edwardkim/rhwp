---
kind: guide
status: active
canonical: mydocs/tech/fuzzing/agent_surface_robustness.md
last_verified: 2026-08-03
---

# 크래시 트리아지 — 나온 것을 어떻게 처리하나

> 퍼저가 뭔가를 뱉었다. 여기서부터 무엇을 하나.
> 실행 자체는 [operations.md](operations.md), 묶음 지도는 [README.md](README.md).

이 문서의 목표는 **크래시 입력 하나를 저장소의 영구 자산(회귀 테스트)으로 바꾸는 것**이다.
고치기만 하고 테스트를 남기지 않으면 같은 클래스가 다시 들어온다 — 이 저장소에서
실제로 일어난 일이다(§6-0).

기준선: 2026-08-03, `rhwp v0.8.2`, HEAD `9095cd52d`.

---

## 0. 절차 한 장

```
fuzz/artifacts/<타깃>/<파일>
      │
 ①    ├── 재현       cargo +nightly fuzz run <타깃> <파일>          (§2)
      │
 ②    ├── 최소화     cargo +nightly fuzz tmin <타깃> <파일>         (§3)
      │
 ③    ├── 층 판별    스택 최상단 프레임 → 파서 층 지도              (§4)
      │
 ④    ├── 클래스 분류 6종 중 하나로 — 반복되면 스윕 이슈            (§5)
      │
 ⑤    ├── 회귀 테스트 승격  tests/issue_####_*.rs                  (§6)
      │
 ⑥    ├── 이슈 → 분석 → 수정 → 처리결과 문서 → PR                 (§7)
      │
 ⑦    └── 최소화 입력을 fuzz/regressions/<타깃>/ 에 커밋            (§8)
```

⑤와 ⑥의 순서를 바꾸지 마라. **테스트가 red 인 것을 먼저 보고** 나서 고친다.
red 를 못 본 수정은 무엇을 고쳤는지 증명하지 못한다.

---

## 1. 산출물 읽기

크래시 산출물은 `fuzz/artifacts/<타깃>/` 에 떨어진다. `fuzz/.gitignore` 가
`artifacts/` 를 제외 대상으로 두므로 **자동으로는 커밋되지 않는다** — 의도된 설계다
(코퍼스와 회귀 케이스와 일회성 산출물을 섞지 않는다).

파일 이름의 접두사가 1차 분류다.

| 접두사 | 무엇 | 어느 플래그가 잡았나 |
| --- | --- | --- |
| `crash-<sha1>` | 패닉 / abort / 세그폴트 | 기본 |
| `oom-<sha1>` | 메모리 상한 초과 | `-rss_limit_mb=2048` |
| `timeout-<sha1>` | 한 입력이 제한 시간 초과 | `-timeout=30` |
| `slow-unit-<sha1>` | 느리지만 끝나긴 함 | 기본(경고) |
| `leak-<sha1>` | LeakSanitizer | 기본 |

`oom-`/`timeout-` 은 **패닉이 아니다.** 그래서 "테스트가 실패한다"가 아니라
"프로세스가 죽는다"로 나타난다 — 회귀 테스트를 쓸 때 이 차이가 결정적이다(§6-5).

> `slow-unit-` 은 결함이 아닐 수 있다. 큰 정상 문서를 파싱하면 당연히 느리다.
> 입력 크기 대비 시간이 **비선형**인지 먼저 보라 — 385바이트가 30초를 먹으면
> 그건 #2743 류다.

---

## 2. 재현

```sh
cargo +nightly fuzz run parse_hwp fuzz/artifacts/parse_hwp/crash-e3b0c442...
```

파일 인자를 주면 변이 없이 그 입력 하나만 실행한다. 확인할 것:

1. **결정적인가.** 3회 돌려 같은 지점에서 죽는가. 비결정적이면 스레드·시간·환경
   의존이므로 최소화가 무의미하다.
2. **스택 최상단이 어디인가.** 이게 §3 의 입력이다.
3. **입력이 얼마나 큰가.** 크면 §3 최소화 전에는 사람이 읽을 수 없다.

재현 실패라면 대개 셋 중 하나다.
- 코드가 이미 고쳐졌다 (다른 PR 이 우연히 막았다) → 그래도 **회귀 테스트는 만든다**.
  #3311 이 정확히 이 경우였다(§6-0).
- 빌드 프로파일이 다르다 — 오버플로 패닉은 debug 어서션이 켜져야 난다.
- 산출물이 잘렸다.

---

## 3. 최소화

```sh
cargo +nightly fuzz tmin parse_hwp fuzz/artifacts/parse_hwp/crash-e3b0c442...
```

`tmin` 은 "같은 크래시를 유지하면서" 입력을 줄인다. 결과는
`fuzz/artifacts/<타깃>/minimized-from-<sha1>` 로 나온다.

최소화가 중요한 이유는 크기가 아니라 **설명 가능성**이다. #2743 의 재현 입력은
**382바이트**였고, 그래서 회귀 테스트가 바이트열을 소스에 직접 적을 수 있었다.

```rust
let bytes = hml_with_charshape_id("1000000");
assert_eq!(bytes.len(), 382, "재현 입력 크기 고정");
```
— `tests/issue_2743_hml_resource_id_limit.rs`

**바이너리 blob 을 커밋해야만 재현되는 테스트보다, 소스에서 합성되는 테스트가 낫다.**
합성 가능한 크기까지 줄이는 것이 `tmin` 의 진짜 목표다.

`tmin` 이 충분히 못 줄이면:
- `-runs=100000` 등으로 시도 횟수를 늘린다.
- 텍스트 포맷(HML·OOXML 차트)이면 **손으로 줄인다.** XML 은 사람이 읽을 수 있다.
- 컨테이너 포맷이면 최소화가 어렵다 — CFB/ZIP 은 체크섬·오프셋이 얽혀 있어
  한 바이트만 빼도 다른 실패가 된다. 이 경우 **합성 스윕**을 쓴다(§6-2).

---

## 4. 어느 파서 층인가 — 층 지도

스택 최상단 프레임의 파일 경로를 이 표에 대응시킨다. 층이 정해지면 담당 하네스,
전형적 결함 클래스, 그리고 **비슷한 선례**가 함께 정해진다.

| 층 | 코드 | 하네스 | 전형적 결함 | 선례 |
| --- | --- | --- | --- | --- |
| CFB 컨테이너 | `src/parser/cfb_reader.rs` | `parse_hwp` | 손상 섹터 id → OOB 슬라이스, DIFAT 순환 | #3311 · #3181 · `905b3261e` |
| HWP5 진입 | `src/parser/mod.rs:184` (`parse_hwp`) | `parse_hwp` | 포맷 판별·헤더 | — |
| HWP5 DocInfo | `src/parser/doc_info.rs` | (2순위 미구현) | 무상한 카운트 할당 | `12ecfece6` (TAB_DEF) |
| HWP5 BodyText | `src/parser/body_text.rs` | (2순위 미구현) | 레코드 크기 오버플로 | `6b29fa1da` |
| HWP5 컨트롤 | `src/parser/control.rs`, `control/shape.rs` | (2순위 미구현) | **부호확장 → 무한루프** | #3012 |
| IR 조립(표) | `src/model/table.rs` | 간접 | span 0 → `u16` 언더플로 | `ed339e78f` |
| HWP3 | `src/parser/hwp3/mod.rs` | `parse_hwp3` | **스케일 곱셈 오버플로** | `cdd55c838`·`e288b0a7f`·`6bcbadcd1`·`77627e953` |
| HWPX(ZIP) | `src/parser/hwpx/mod.rs` | `parse_hwpx` | zip 폭탄, XML 폭증 | 확인되지 않음 |
| HML(XML) | `src/parser/hml/mod.rs`, `hml/reader.rs` | `parse_hml` | **상한 기구의 빈틈** | #2743 |
| WMF | `src/wmf/converter/mod.rs` + `src/wmf/parser/**` | `parse_wmf` | 음수 카운트 → `with_capacity` | #3004 · #3000 · #3301 |
| OOXML 차트 | `src/ooxml_chart/parser.rs` | `parse_ooxml_chart` | XML 파서 경계 | 확인되지 않음 |
| EMF | `src/emf/**` | **하네스 없음** | (WMF 와 동형 예상) | RFC #3141 §4 2순위 |

### 4-1. 두 층 이상이 관여할 때

컨테이너를 통과해 내부에서 죽으면 **하네스는 컨테이너 것인데 결함은 내부**다.
이때 하는 일:

1. 결함은 **내부 층**으로 귀속시킨다. 수정도 거기서 한다.
2. 회귀 테스트는 **가능하면 내부 함수를 직접 부른다.** 컨테이너를 거치면 테스트가
   컨테이너 변경에 취약해진다.
3. 그 층에 하네스가 없다면(위 표의 "2순위 미구현") **하네스를 만드는 게 후속 작업이다** —
   RFC #3141 §4 가 이미 목록을 갖고 있다.

### 4-2. HML 층 특유의 주의

`parse_hml` 하네스는 **기본 `HmlLimits` 를 그대로 쓴다**(하네스 파일 주석: "상한 기구의
빈틈(#2743류)을 찾는 것이 목적이므로 상한을 풀지 않는다"). 즉 HML 크래시가 나왔다면
그건 **상한을 뚫었다**는 뜻이고, 고칠 곳은 `src/parser/hml/reader.rs:40` 의
`HmlLimits` 에 항목을 추가하는 쪽일 가능성이 높다.

---

## 5. 결함 클래스 분류

RFC #3141 §1 이 관찰한 대로, 이 저장소의 결함은 소수의 클래스가 반복된다.
분류가 중요한 이유는 **클래스가 반복되면 전수 스윕 이슈를 여는 것**이 관행이기
때문이다(#3004 → #3012 흐름).

| # | 클래스 | 서명 | 대표 |
| --- | --- | --- | --- |
| C1 | **부호 있는 정수 → `usize` 무검증 캐스팅** | `capacity overflow`, 또는 끝나지 않는 루프 | #3004 · #3012 · `cdd55c838` |
| C2 | **무검증 할당** (파일 값이 곧 할당 크기) | `oom-`, 또는 `handle_alloc_error` abort | #2743 · #2722 · #3000 |
| C3 | **산술 오버플로** (곱셈 스케일 변환) | debug 에서 `attempt to multiply with overflow` | `e288b0a7f`·`6bcbadcd1`·`77627e953` |
| C4 | **언더플로** (`a - b` 에서 b>a) | `attempt to subtract with overflow` | `ed339e78f` (span 0) |
| C5 | **경계 없는 인덱싱/슬라이싱** | `range end index N out of range for slice of length M` | #3311 |
| C6 | **순환/무한 반복** (체인 순회) | `timeout-` | #3181 (DIFAT 순환) |

**분류할 때 던지는 질문 하나: "이 클래스를 같은 파일에서 이미 고친 적이 있나?"**
있다면 그건 스윕 대상이다. `1b02247ff`(#3301) 커밋 메시지가 그 사고방식을 그대로 적는다 —
"이미 겪은 클래스(#3008 Region scan_count, #3181 CFB DIFAT 순환)와 같은 패턴이 남은 곳."

---

## 6. 회귀 테스트로 승격 — 이 저장소의 세 가지 패턴

### 6-0. 왜 반드시 하나 — #3311 이 준 교훈

`tests/issue_3311_malformed_cfb_no_panic.rs` 주석이 이유를 그대로 적었다.

> "결함 자체는 `6a761a793`(#3220 …)에서 해소됐다 … 다만 그 수정들은 **개별 방어를
> 추가했을 뿐 "손상 입력은 패닉하지 않는다"는 계약을 고정하지 않았다.**
> 이 테스트가 그 계약을 못박아 같은 클래스의 재유입을 검출한다."

수정만으로는 부족하다. **계약을 못박아야** 같은 클래스가 다시 못 들어온다.

### 6-1. 패턴 A — 재현 입력 고정형 (`tests/issue_2743_hml_resource_id_limit.rs`)

최소화 입력이 **소스에서 합성 가능한 크기**일 때 쓴다. 최선의 형태다.

구성 요소 4개:

```rust
/// ① 재현 입력을 함수로 합성 — 파라미터가 결함의 축이다
fn hml_with_charshape_id(id: &str) -> Vec<u8> { ... }

#[test]
fn hml_resource_id_beyond_limit_is_skipped_with_warning() {
    let bytes = hml_with_charshape_id("1000000");
    assert_eq!(bytes.len(), 382, "재현 입력 크기 고정");   // ② 크기 고정

    let parsed = parse_hml(&bytes).expect("상한 초과 Id 여도 문서는 열려야 함");
    let len = parsed.document.doc_info.char_shapes.len();
    assert!(len < 1_000, "...");                            // ③ 결과 단언
    assert_eq!(invalid_reference_warnings(&parsed), 1, "..."); // ④ 경고 단언
}
```

- **② 크기 고정 단언**은 픽스처가 바뀌면 알려 준다.
- **③ 결과 단언이 "죽지 않음"보다 강하다.** #2743 의 "조용한 구간"은 `Ok` 를
  반환하면서 120MB 를 먹었다 — `parse_hml(...).is_ok()` 만 단언했으면 **수정 전에도
  통과**해서 red 가 안 됐다. 테스트 주석이 이걸 명시한다: "따라서 아래 가드는
  '죽지 않음'이 아니라 **결과 테이블 길이와 경고**를 단언한다(그렇게 해야 수정 전에
  red 가 된다)."
- 같은 파일이 **경계값 테스트**(65535 수용 / 65536 거부)와 **무회귀 테스트**
  (정상 범위 Id 는 동작 완전 불변)를 함께 둔다. 셋을 같이 써라.

### 6-2. 패턴 B — 합성 스윕형 (`tests/issue_3311_malformed_cfb_no_panic.rs`)

컨테이너 포맷이라 최소화가 안 되거나, 결함이 **값의 조합**일 때 쓴다.

```rust
/// 리포터가 보고한 실측 값 — 이 조합이 구 커밋에서 패닉을 냈다.
const REPORTED_FAT_ENTRIES: u32 = 824;
const REPORTED_POISON_SECTOR: u32 = 1_851_072_928;
const REPORTED_LEN: usize = 3072;

fn synth_malformed_cfb(len, fat_count, difat_count, first_difat, poison) -> Vec<u8> { ... }

fn malformed_cases() -> Vec<(String, Vec<u8>)> {
    // ① 리포터 조건 정확 재현 1건
    // ② 경계 스윕: len 6종 × fat 5종 × poison 5종 = 150건
    // ③ 실 샘플 헤더 뮤테이션 + 절단
}

#[test]
fn malformed_cfb_returns_err_instead_of_panicking() {
    let cases = malformed_cases();
    assert!(cases.len() >= 150, "케이스 수가 급감했다 — ... 커버리지가 줄었는지 확인할 것");
    let mut opened = 0;
    for (name, bytes) in &cases {
        match HwpDocument::from_bytes(bytes) { Ok(_) => opened += 1, Err(_) => {} }
    }
    assert!(opened < cases.len(), "손상 입력이 모두 정상 개봉됐다 — 케이스가 더 이상 malformed 가 아니다");
}
```

이 패턴의 세 가지 발명:

1. **리포터 실측값을 상수로 명시** — 6개월 뒤 "왜 824인가"를 답할 수 있다.
2. **케이스 수 하한 단언**(`>= 150`) — 샘플 경로가 바뀌어 케이스가 조용히 사라지는 것을 잡는다.
3. **"전부 열리면 실패"** — 케이스가 더 이상 손상 입력이 아니게 되는 퇴화를 잡는다.

여기서 패닉 단언은 **암묵적**이다. 패닉하면 그 지점에 도달하지 못해 테스트가 실패한다
(주석: "패닉하면 이 지점에 도달하지 못하고 테스트가 실패한다 — 그것이 이 가드가
잡으려는 회귀다").

### 6-3. 패턴 C — CLI 프로세스형 (`tests/issue_cli_test_caption_no_panic.rs`)

크래시가 **CLI 표면**에서 재현될 때. `exit 101`(Rust 패닉)을 직접 단언한다.

```rust
let out = Command::new(rhwp_bin()).args(["test-caption", sample]).output()...;
assert_ne!(out.status.code(), Some(101),
    "Rust panic(exit 101) 발생 — 범위 밖 인덱싱 회귀. stderr: {}", ...);
assert_eq!(out.status.code(), Some(0), "예기치 않은 종료 코드. ...");
```

**패닉은 exit 101 이다.** 프로세스를 띄우는 테스트에서는 이게 가장 정확한 서명이고,
동시에 [#2707 exit 사전](agent_surface_robustness.md) 과 맞물린다 — 손상 문서는
**101 이 아니라 1** 로 끝나야 한다.

### 6-4. 어느 패턴을 고르나

```
최소화 입력이 소스에 적을 만한가?
   ├─ 예 ────────────────────▶ 패턴 A (재현 입력 고정형)
   └─ 아니오
        ├─ 결함이 값 조합인가? ▶ 패턴 B (합성 스윕형)
        └─ CLI 에서 재현되나?  ▶ 패턴 C (프로세스형)
```

세 패턴은 배타적이지 않다. #3311 은 B 를 쓰면서 실 샘플 뮤테이션까지 섞었다.

### 6-5. ⚠️ 회귀 테스트가 CI 러너를 죽이면 안 된다

이건 이 저장소에서 **메인테이너가 직접 보정한 규칙**이다.
`tests/issue_2743_hml_resource_id_limit.rs` 안에 그대로 남아 있다.

> **[메인테이너 보정]** 종전에는 Id `2000000000` 을 썼다. 가드가 살아 있으면
> 안전하지만, **가드가 회귀로 사라지면 `CharShape` 120B × 20억 = 240GB** 를 요구해
> 테스트가 실패하는 대신 러너가 죽는다. 원저자도 "수정 전에는 테스트 실패가 아니라
> 프로세스가 죽는다"고 주석에 적어 두었는데, 그것이 바로 CI 에서 허용될 수 없는 성질이다.
>
> 상한(65,535)을 크게 넘는다는 성질은 Id `2_000_000` 으로도 동일하게 검증되며,
> 회귀 시 요구량은 240MB 로 진단 가능한 범위에 머문다.

**규칙: C2(무검증 할당)·C6(무한루프) 클래스의 회귀 테스트는 "회귀했을 때 어떻게
실패하는가"를 설계해야 한다.**

| 클래스 | 회귀 시 나쁜 실패 | 대신 이렇게 |
| --- | --- | --- |
| C2 무검증 할당 | 러너 OOM 킬 → 로그 없음 | 요구량이 수백 MB 대에 머무는 값을 고른다 |
| C6 무한루프 | CI 타임아웃(수십 분) | 반복 상한을 단언하거나, 별도 스레드 + 시간 예산 |
| C3/C4 오버플로 | (없음 — 깔끔히 패닉) | 그대로 두면 된다 |

원래 값(240GB·`2000000000`)은 **주석에 남긴다.** 값을 낮춘 이유를 못 적으면
다음 사람이 "왜 이렇게 어중간하지" 하며 되돌린다.

---

## 7. 이슈 → 수정 → PR

이 저장소의 기여 절차는 고정돼 있다: **이슈 등록 → 분석 → 코드 변경 → 처리결과 문서 → PR.**
퍼징 발견물도 예외가 아니다.

1. **이슈 등록** — 제목에 층·클래스를 담는다. #3311 의 제목이 좋은 형태다:
   `패닉(OOB slice): LenientCfbReader::open 이 malformed CFB FAT 에서 'range end index out of range'`
   본문에 넣을 것: 환경(커밋 해시·features·OS) / 패닉 원문 스택 / 원인 추정 /
   기대 동작 / 재현 방법. **최소화 입력을 첨부하거나 합성 코드를 적는다.**
2. **red 확인** — §6 의 회귀 테스트를 먼저 쓰고 실패를 본다.
3. **수정** — 같은 클래스의 기존 수정을 본떠라. C1 이면
   `src/wmf/parser/objects/graphics/region.rs:96` 의 `if scan_count < 0 { Err }` 형태,
   C3 이면 `read_hwp3_padding_scaled` 처럼 **i32 중간값을 거치는 헬퍼**로 통일하는 형태다.
4. **green + fmt/clippy** — PR 전 `cargo fmt --all -- --check`, `cargo clippy -- -D warnings`.
   CI `lint` 잡이 이 둘을 그대로 돌린다(`.github/workflows/ci.yml`).
5. **처리결과 문서** — `mydocs/report/task_m100_####_report.md`.
6. **PR** — 닫는 이슈 명시.

### 보안 성격 판단

원격 코드 실행급이거나 메모리 안전을 넘는 것이면 **공개 이슈로 올리지 않는다** —
[`SECURITY.md`](../../../SECURITY.md) 의 GitHub Security Advisory 경로를 쓴다.
판단 기준은 [agent_security/disclosure.md](../agent_security/disclosure.md).

단순 파서 패닉(DoS)은 #3311 처럼 **공개 이슈**로 접수된 선례가 있다. 제보자 본인이
"실사용에선 격리 서브프로세스 + fail-closed 로 무해 격리되지만, 라이브러리 견고성
차원에서 제보합니다"라고 성격을 명시했고, 메인테이너가 그대로 공개 처리했다.

---

## 8. 회귀 코퍼스에 커밋

```
fuzz/regressions/<타깃>/<이슈번호>-<짧은설명>
```

`fuzz/README.md` §트리아지가 정한 규약이다 — **`fuzz/corpus/` 가 아니라
`fuzz/regressions/`.** 둘을 섞으면 회귀 케이스가 코퍼스 최소화(`cmin`)에 먹혀
사라진다.

**이 디렉터리는 아직 존재하지 않는다**(`ls fuzz/regressions` → not found).
첫 크래시를 처리하는 사람이 만든다. 만들 때 같이 할 일:

- [ ] `fuzz/README.md` 의 코퍼스 표에 `regressions/` 행 추가
- [ ] 회귀 입력을 읽어 파서를 부르는 재생 테스트 1개
      (`tests/fuzz_regressions.rs` 같은 이름) — **cargo-fuzz 없이 `cargo test` 로 돈다**
- [ ] #3608 M21 체크리스트의 "크래시 코퍼스 회귀 스위트 편입" 체크

재생 테스트의 뼈대는 이미 저장소에 있다 — `tests/issue_3311_malformed_cfb_no_panic.rs`
가 파일을 읽어 `HwpDocument::from_bytes` 에 넣는 형태 그대로다.

---

## 9. 이 PC 에서의 제약

[operations.md §6](operations.md) 의 결론이 그대로 적용된다. Windows SDK 의
`dbghelp.lib` 손상으로 **MSVC 타깃 실행 파일을 만들 수 없어** `cargo fuzz run`·
`cargo fuzz tmin`·`cargo test` 가 모두 불가하다.

| 단계 | 이 PC | 대안 |
| --- | --- | --- |
| ① 재현 | ✗ | 퍼징 가능 환경 |
| ② 최소화 | ✗ | 퍼징 가능 환경. 텍스트 포맷이면 **손으로** 줄일 수 있다 |
| ③ 층 판별 | ✓ | 스택 문자열 + 코드 읽기로 충분 |
| ④ 클래스 분류 | ✓ | 코드 읽기 |
| ⑤ 회귀 테스트 작성 | ✓ (작성만) | 실행·red 확인은 CI |
| ⑥ 이슈·PR | ✓ | — |

**red 를 못 보고 PR 을 올리는 것은 이 저장소의 관행에 어긋난다.** 이 환경에서
작업한다면 PR 본문에 "red 미확인, CI 로 확인 요청"을 명시해라 —
`e288b0a7f` 커밋이 그렇게 했다("로컬 빌드 검증은 리뷰어 몫으로 남긴다").

---

## 10. 확인되지 않음

| 항목 | 이유 |
| --- | --- |
| `cargo fuzz tmin` 이 이 저장소 타깃에서 실제로 얼마나 줄이는가 | 실행 불가 |
| `oom-`/`timeout-` 산출물의 실제 파일명 형식 | libFuzzer 문서 기반 서술. 이 저장소에서 나온 적 없음 |
| HWPX(ZIP)·OOXML 차트 층의 전형적 결함 | 해당 층에서 확정된 결함 사례가 저장소에 없음 |
| EMF 층 | 하네스 자체가 없음(RFC #3141 §4 2순위) |
| 퍼징 발견물이 이 저장소 이슈로 접수된 총 건수 | #3311 하나만 확인. 그 외는 라벨·본문으로 식별되지 않음 |

## 관련

- [README.md](README.md) · [operations.md](operations.md) · [agent_surface_robustness.md](agent_surface_robustness.md)
- 회귀 테스트 원본: `tests/issue_2743_hml_resource_id_limit.rs` · `tests/issue_3311_malformed_cfb_no_panic.rs` · `tests/issue_cli_test_caption_no_panic.rs`
- [`fuzz/README.md`](../../../fuzz/README.md) §트리아지 절차 — 규약의 1차 출처
- [`SECURITY.md`](../../../SECURITY.md) · [agent_security/disclosure.md](../agent_security/disclosure.md)
- 이슈: [#3141](https://github.com/edwardkim/rhwp/issues/3141) · [#3311](https://github.com/edwardkim/rhwp/issues/3311) · [#3608](https://github.com/edwardkim/rhwp/issues/3608) M21
