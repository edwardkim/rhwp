---
name: rhwp-layout-debug
description: 조판 실패(줄 수·쪽 수·줄바꿈 위치 불일치)를 rust-lldb 라이브 스톱에서 판별합니다. 줄바꿈 로직 결함인지 폰트 메트릭 결함인지를 추론이 아니라 피연산자로 가릅니다 — fill 상자·저장 기록·pen·거부 글자·해소된 face·새 줄 수 대 저장 줄 수를 읽고, "저장 폭에서 강제 재조판" 판별자로 둘을 분리합니다. 트리거 — 사용자가 "쪽 수가 안 맞아", "줄이 하나 더/덜 생겨", "한컴이랑 줄바꿈이 다르다", "이게 프레임 버그인지 폰트 문제인지", "실패 안을 들여다봐", "디버거로 확인해" 등을 요청할 때.
---

# rhwp-layout-debug — 조판 실패를 라이브 스톱에서 판별한다

## 왜 이 스킬이 있나

**줄 수 차이는 줄바꿈 결함과 메트릭 결함이 똑같이 생긴다.** 저장 기록과 새 측정의 줄 수가
다를 때, 원인은 우리 줄바꿈 로직일 수도 있고 폰트가 없어서 advance 가 대체 face 에서 나온
것일 수도 있다. 진단 없이 고치면 측정 결함을 조판 로직 안에 보상으로 박아 넣는다.

실제 사례: `76076_regulatory_analysis` 셀이 한컴과 한 글자 어긋난 원인은 프레임이 아니라
`한양신명조`·`한양중고딕` 이 HFT 라 로컬에 없고 모든 advance 가 대체 face 에서 나온 것이었다
(#4779, #5678). 코드를 읽어서는 구별되지 않는다.

## 절차

### 0. 이 핀이 여기서 통과한 적이 있는지 30초 안에 확인한다

디버거를 켜기 전에 물어야 한다 — **이 테스트가 이 브랜치에서 초록이었던 적이 있나.**

```sh
git cat-file -e HEAD:tests/cases/<test_file>.rs   && echo "브랜치에 있음"
git cat-file -e MERGE_HEAD:tests/cases/<test_file>.rs && echo "업스트림에 있음"
git log --oneline -S'<핵심 심볼>' <merge-base>..MERGE_HEAD | tail -3
```

머지·리베이스 실패에서는 실패의 절반이 **한 번도 돈 적 없는 핀**이다. 업스트림에만 있는
테스트 파일은 우리 코드의 회귀가 아니라 우리가 구현한 적 없는 계약이고, 메커니즘을 파기
전에 그 사실이 답인 경우가 있다. 오라클 기대값(baseline TSV, `let expected = [...]`)이
업스트림에서 개선된 것인지도 여기서 본다 — 기대값이 움직였는데 코드가 안 왔으면 그게 전부다.

이 단계를 건너뛰면 "구현된 적 없는 것"을 "고장난 것"으로 진단하게 된다.

### 1. 디버깅 가능한 바이너리를 먼저 만든다 — 게이트 프로파일로는 안 된다

**`release-test` 바이너리에는 DWARF 가 없다.** 중요한 것은 **왜** 없는가다 —
`[profile.release-test]` 에는 `strip` 설정이 **없다**. 찾으러 가지 마라. 그 프로파일은

```toml
[profile.release-test]
inherits = "release"      # ← 여기서 딸려 온다
```

이고, `[profile.release]` 가 `strip = "debuginfo"` 를 켠다. 즉 게이트 프로파일은 자기가
스트립을 지정한 적이 없는데 상속으로 스트립된다. `release-test` 블록만 읽고 "스트립 안
하네" 라고 판단하면 없는 설정을 찾아 헤매게 된다.

macOS 에서 세는 법에 주의한다. Rust 는 DWARF 를 `.o` 에 두고 실행 파일에는 **debug map**
(`N_OSO` 스탠자)만 남긴다 — `dsymutil` 을 돌리기 전에는 `__debug_info` 섹션이 원래 없다.
그래서 `otool -l | grep __debug_info` 는 디버그 정보가 **있어도** 0 을 준다. 옳은 계수는:

```sh
nm -pa <bin> | grep -c ' OSO '
# 게이트 바이너리(target/pr-review) → 0     ← 스트립됨, 디버깅 불가
# 디버그 빌드(아래)                → 354   ← .o 의 DWARF 를 lldb 가 읽는다
```

DWARF 가 0 이면 `p <local>` 도, `up` 으로 조판 프레임에 올라가는 것도, 인라인된 함수에
브레이크포인트를 다는 것도 **전부 불가능**하다. 2번의 여섯 값은 한 개도 못 읽는다.
게다가 release opt-level 에서 `stored_rows_are_stale` 같은 술어는 심볼 표에 아예 없다
(`nm <bin> | grep -c stored_rows_are_stale` → 0). 인라인됐다.

그래서 **별도 target-dir 로** 디버그 정보를 켜서 다시 빌드한다. 같은 target-dir 에
환경변수를 얹으면 게이트 캐시가 깨져 전체 재빌드를 두 번 낸다:

```sh
CARGO_PROFILE_RELEASE_TEST_DEBUG=2 CARGO_PROFILE_RELEASE_TEST_STRIP=none \
  cargo test --profile release-test --target-dir target/lldb --no-run \
  --test <suite>
```

opt-level 은 건드리지 않는다 — 게이트와 같은 코드 생성을 유지해야 같은 실패를 본다.
인라인된 프레임은 DWARF 만 있으면 lldb 가 잡는다.

바이너리 이름에는 해시가 붙고 재빌드마다 바뀐다. 고정 경로로 적지 말고 매번 찾는다:

```sh
BIN=$(ls -t target/lldb/release-test/deps/<suite>-* | grep -v '\.d$' | head -1)
```

### 2. 패닉을 무장한다 — `-n` 이 아니라 `--func-regex`

```
(lldb) breakpoint set -n rust_panic          # ✗ pending, 0 locations
(lldb) breakpoint set --func-regex 'rust_panic'   # ✓ 2 locations
```

`-n` 은 안 붙는다. 심볼이 Rust v0 맹글링이라 이름이
`__RNvCsdBezzDwma51_7___rustc10rust_panic` 이고 `-n` 은 정확 일치를 요구한다. 실측으로
확인한 형태는 `--func-regex` 하나뿐이다.

```sh
rust-lldb "$BIN" -- --exact <test::path> --nocapture --test-threads=1
```

패닉을 무장하지 않으면 어설션 메시지만 남고 프레임이 풀린다. `left: 65, right: 64` 는
숫자 두 개고, 라이브 스톱은 그 숫자가 어디서 왔는지다.

`--test-threads=1` — 병렬 스톱은 읽을 수 없다.

### 3. 결정 지점에서 여섯 값을 읽는다

```
(lldb) bt                     # 어느 소유자가 이 결정을 했나
(lldb) up                     # 어설션 프레임에서 조판 프레임으로
(lldb) p <box>                # fill 이 받은 상자
(lldb) p *stored              # column_start / segment_width
(lldb) p <pen>                # 거부 지점 누적 advance
(lldb) p <glyph_advance>      # 거부된 글자의 폭
(lldb) p <resolved_face>      # 해소된 face 이름
(lldb) p composed->lines.len() vs para->line_segs.len()
```

| 읽는 값 | 무엇을 가르나 |
| --- | --- |
| fill 상자 vs 저장 `segment_width` | 상자가 다른가, 내용이 다른가 |
| **저장 `tag`** | **이 기록을 술어가 진짜로 인정하나** (아래) |
| pen + 거부 글자 vs 상자 | 우리 측정으로 그 거부가 옳았나 |
| 해소된 face | 진짜 메트릭인가 폴백인가 |
| 새 줄 수 vs 저장 줄 수 | 차이의 크기 |

**`tag` 을 빼먹지 마라 — 여섯 값 중 가장 먼저 읽어야 한다.** `stored_rows_are_stale` 은
`tag & TAG_IMPLEMENTATION_PROPERTY == 0` 인 세그먼트만 "저장 기록" 으로 친다. 합성
세그먼트면 술어가 첫 줄에서 `false` 로 빠지므로 나머지 다섯 값은 읽어 봐야 의미가 없다.
거꾸로 `tag=0x0` 인데 `segment_width=0` 이면 "인정받는 기록인데 폭이 없다" 는 뜻이고,
그게 5번 판별자가 성립하지 않는 조건이다.

**실제로 안 읽게 되는 값들.** `pen` 과 `거부된 글자의 advance` 는 "같은 폭, **다른** 줄 수"
가지에서만 쓸모가 있다. 저장 행이 그대로 수용되거나(=fill 이 아예 안 돈다) 저장 폭이
없으면 그 지점에 도달하지 못한다 — 두 번의 실측 진단에서 모두 못 읽었다. 먼저 5번으로
어느 가지인지 정하고, 그 가지일 때만 pen 을 쫓는다. `resolved_face` 는 4번의 조회가
쪽 단위로 대신하므로 스톱에서 읽을 이유가 거의 없다.

`font_family_has_metrics` 는 **로컬 설치 여부가 아니라 rhwp 내장 메트릭 표에 있는지**를
답한다. 설치 안 된 한컴 face 에 대해 `true` 를 반환하고 설치된 시스템 face 에 대해 `false` 를
반환한다 — 재현 가능성 판별자로 쓰면 안 된다. 재확인함(2026-08): 본문이
`font_metrics_data::find_metric(primary_name, bold, italic).is_some()` 한 줄이다
(`layout/text_measurement.rs`). OS 를 묻지 않는다.

### 4. 메트릭 질문은 브레이크포인트 말고 `rhwp-q-font-trace` 로 답한다

**해소된 face 를 스톱에서 한 글자씩 읽지 마라.** 이 저장소에는 쪽 단위로 같은 질문에
답하는 읽기 전용 조회가 있고, 그게 §5 가 요구하는 "모집단" 계수기다:

```sh
./target/pr-review/release-test/rhwp-q-font-trace <파일> --page <N> --json
```

판정에 쓰는 필드는 넷이다:

| 필드 | 읽는 법 |
| --- | --- |
| `document.embedded` / `document.substFont` | `false` / `null` 이면 대체 face 가 아니다 |
| `layoutMetric.matchKind` | `exact` / `boldOnly` / `nameFirst` — 표의 어느 슬롯에 붙었나 |
| `layoutMetric.characterMatch` | `hit` / `miss` — `miss` 는 그 글자만 폴백 추정 |
| `layoutMetric.aliasResolvedFace` | `한양신명조 → HanyangSinMyeongJo` 같은 별칭 해소 결과 |

실측 예 (`hwp3-sample16-hwp5.hwp` p20, 1024 글자): `matchKind` 전부 `exact`,
`characterMatch` `hit` 1005 / `miss` 19 (1.9%), `substFont` 전부 `null`. 즉 이 문서의
**배치 advance 는 내장 메트릭 표의 HFT 전용 값**이지 대체 face 값이 아니다. 같은 조회를
`76076_regulatory_analysis.hwpx` p81 에 돌리면 `한양신명조`·`한양중고딕` 이 똑같이 `exact`
로 붙는다.

이것이 왜 중요한가: **"HFT 가 로컬에 없다" 는 사실만으로 메트릭 결함을 단정할 수 없다.**
없는 것은 *그리기* backend 쪽이고(위 조회에서 `paint.*.status` 가 전부 `unsupported`),
배치는 내장 표에서 나온다. #4779·#5678 을 인용하기 전에 이 조회로 배치 경로가 실제로
대체를 탔는지 확인한다. 남는 의심은 "내장 표의 값이 진짜 HFT 와 같은가" 이고, 그건
스톱에서 답할 수 있는 질문이 아니다.

### 5. 판별자 — 저장 폭에서 강제 재조판

이것이 두 원인을 가르는 유일한 실험이다.

새 재조판을 carve 가 낸 폭이 아니라 **저장 기록의 폭**에서 돌리고 줄 수를 다시 읽는다.

| 결과 | 결론 |
| --- | --- |
| 같은 폭, **다른** 줄 수 | 줄바꿈 로직이 다르다 — advance·자간·금칙 |
| 같은 폭, **같은** 줄 수 | 폭이 전부였다 — 조판이 아니다. **아래에서 두 갈래로 나뉜다** |

같은 폭인데 줄 수가 다르면 그 다음 질문은 "advance 가 대체 face 에서 왔나"다. 3번의
`resolved_face` 와 4번의 조회가 그 답이다.

#### "같은 폭, 같은 줄 수" 는 한 가지 결론이 아니다

이 판별자는 **세 번째 원인을 두 번째와 구별하지 못한다.** 저장 기록을 그 기록 자신의
폭에서 다시 접으면 당연히 같은 줄 수가 나온다 — 그건 기록이 옳다는 뜻이지 현재 상자가
틀렸다는 뜻이 아니다. 갈라야 할 두 경우:

| | 저장 기록이 쓰인 상자 | 지금 상자 | 결론 |
| --- | --- | --- | --- |
| **상자 유도 결함** | 지금과 같아야 함 | 잘못 유도됨 | `ParagraphBox` 생성자·호출부 |
| **staleness 미탐지** | 정당하게 달랐음 | 정당하게 바뀜 | 기록을 버릴 판정이 없다 |

두 번째는 문서가 **런타임에 바뀐** 경우다 — 쪽 설정 변경, 단 변경, 셀 분할. 저장 기록은
자기 폭에서 완벽히 유효하고, 새 상자도 옳고, 어긋난 것은 "이 기록은 더 이상 이 상자의
것이 아니다" 를 말해 줄 술어뿐이다. 실측 예: `issue_4956_page_margin_rewrap` 이 본문 폭을
절반으로 좁힌 뒤 머리말이 상자를 **1.077×** 넘는데, 우리 staleness 술어
(`stored_rows_are_stale`) 는 #2525 의 **1.8×** 과밀에서만 발화한다. 줄바꿈도 메트릭도
상자 유도도 멀쩡하고, 문턱만 굵다.

그래서 판별자를 돌리기 전에 **저장 기록이 쓰인 상자를 알아낸다**: `column_start` +
`segment_width` 가 지금 상자와 다르면, 그리고 테스트가 문서를 런타임에 고쳤다면, 이건
탐지 문제다. 이 확인 없이 "같은 폭, 같은 줄 수" 를 상자 유도로 돌리면 엉뚱한 곳을 판다.

#### 디버거의 Vec 읽기를 독립 도구로 교차 검증한다 — 이 절은 오독에서 나왔다

이 자리에는 원래 "HWP3 변환본은 `segment_width = 0` 이라 판별자가 성립하지 않는다"는 절이
있었다. **그 관측 자체가 디버거 오독이었다.** 남겨 두는 이유는 같은 함정이 다시 나오기
때문이다.

최적화 빌드에서 Rust 합성 프로바이더가 안 붙으면 `Vec<T>` 의 자식은 **원소가 아니라 필드**다:

```python
ls = para.GetChildMemberWithName('line_segs')
ls.GetNumChildren()      # → 2   ← 원소 수가 아니라 필드 수 (buf, len)
ls.GetChildAtIndex(0)    # → buf 필드. LineSeg 가 아니다.
ls.GetChildAtIndex(0).GetChildMemberWithName('segment_width')  # 없는 멤버
    .GetValueAsSigned()  # → 0   ← 조용히 0. 오류가 안 난다.
```

이렇게 해서 스톱 1735 회 전수에서 `segment_width = 0, column_start = 0, tag = 0x0` 이
"측정" 됐고, 전부 허구였다. 실제 값은:

```sh
rhwp dump samples/hwp3-sample16-hwp5.hwp --section 0 | grep -oE 'cs=-?[0-9]+, sw=-?[0-9]+' | sort | uniq -c | sort -rn
#  319 cs=0, sw=51024
#  203 cs=5000, sw=45024
#  172 cs=2500, sw=47524   … 1275 행, 전부 실제 값
```

**징후는 "모집단 값이 이상하게 일정한 것" 이다.** 1735 개 스톱 전부에서 `nsegs=2` 가 나왔다 —
실제 문서의 문단이 전부 정확히 2 줄일 리가 없다. 그 상수가 `Vec` 의 필드 수였다.

지켜야 할 규칙 둘:

1. **길이는 `len` 필드로 읽는다**: `ls.GetChildMemberWithName('len').GetValueAsUnsigned()`.
   `GetNumChildren()` 을 원소 수로 믿지 않는다.
2. **모집단 주장을 하기 전에 독립 도구로 한 줄 교차 검증한다.** `rhwp dump <파일>
   --section <N>` 이 같은 값을 파일에서 직접 준다. 디버거 관측과 어긋나면 디버거가 틀린
   것이다. 이 교차 검증은 명령 하나고, 건너뛰면 이슈 하나가 통째로 무효가 된다.

### 6. 모집단을 먼저 재고 메커니즘을 나중에 단정한다

라이브 스톱 세 번으로 메커니즘을 일반화하지 않는다. 이 저장소에서 두 번 그렇게 해서 두 번
라운드를 낭비했다 — "run 이 없는 composition" 이 사실은 빈 문단이었고, `cs=0` 이 사실은
편집된 문서의 값이었다.

브레이크포인트는 **메커니즘**에, 계수기(instrumented sweep)는 **모집단**에 쓴다. 반대로 하면
셋을 보고 전체를 말하게 된다.

**모집단은 lldb 안에서 센다 — 자동 continue 콜백.** 코드를 고쳐 `eprintln!` 을 심을 필요가
없고, 게이트 바이너리를 건드리지 않는다:

```python
# /tmp/probe.py
def on_stop(frame, bp_loc, extra, internal_dict):
    v = frame.FindVariable('inner_width_px')
    if not (v and v.IsValid() and v.GetValue()): return False
    ls = frame.FindVariable('para').GetChildMemberWithName('line_segs')
    n = ls.GetChildMemberWithName('len').GetValueAsUnsigned()   # ← GetNumChildren() 아님
    if n == 0: return False
    print("ROW box=%.1f n=%d" % (float(v.GetValue()), n))
    return False        # ← False 가 "멈추지 말고 계속" 이다
```

```
(lldb) command script import /tmp/probe.py
(lldb) breakpoint set --func-regex 'stored_rows_are_stale'
(lldb) breakpoint command add -F probe.on_stop 1
(lldb) run
```

그 다음 셸에서 집계한다 — `sort | uniq -c`. 스톱 수는 스톱 수일 뿐이고, **필드 값을
모집단으로 주장하려면 `rhwp dump` 로 교차 검증한 뒤**에 한다(§5 의 오독 사례).
값이 이상하게 일정하면 그것이 첫 번째 경보다.

`breakpoint command add -o "cmd1" -o "cmd2" ...` 로 여러 명령을 엮는 형태는 배치 모드에서
출력이 삼켜진다(실측: `continue` 만 보이고 `frame variable` 결과가 안 나온다). Python
콜백을 쓴다. 스톱 하나만 자세히 볼 때는 `-b -s` 파일에 `run` 다음 줄로 조회 명령을 쓰면
그 스톱 컨텍스트에서 실행된다.

## 함정

- `thread return <val>` 로 분기를 강제하면 프레임이 깨질 수 있다 — `line_segs=size=4294967295`
  같은 값이 나오면 그 실행의 모든 값을 버린다.
- 게이트는 `--test-threads 6`. 12 스레드면 `scan_cost_stays_linear_as_input_grows` 가 부하로
  실패한다(부하 15.2s vs 단독 1.3s).
- 조건부 브레이크포인트로 좁힌다. `-n` 은 v0 맹글링에 안 붙으므로 `--func-regex` 와 함께
  쓴다: `breakpoint set --func-regex '<fn>' -c 'stored->segment_width == 40520'`. 조건식은
  DWARF 가 있어야 평가된다 — 1번을 건너뛰면 조건이 조용히 무시되고 전부 멈춘다.
- **게이트 target-dir 에 디버그 환경변수를 얹지 않는다.** `CARGO_PROFILE_*` 를 바꾸면
  fingerprint 가 달라져 `target/pr-review` 가 통째로 재빌드되고, 게이트를 다시 돌릴 때 또
  재빌드한다. 별도 `--target-dir` 을 쓴다.
- 심볼이 있다고 브레이크포인트가 붙는 것은 아니다. `nm` 에 이름이 보여도 DWARF 가 없으면
  인라인 프레임과 지역 변수는 못 잡는다. `nm -pa <bin> | grep -c ' OSO '` 로 먼저 센다
  (macOS 에서 `otool -l ... __debug_info` 는 오답 — 위 1번 참조).

## 판정 후

- **줄바꿈 로직** → 어느 술어가 다른지 지목한다. 두 술어가 같은 양을 다르게 묻고 있는 경우가
  흔하다(하나는 가드 있고 하나는 없는).
- **메트릭** → #4439·#4779·#5678 범위다. 조판 쪽에 보상을 넣지 않는다.
- **상자 유도** → `ParagraphBox` 생성자와 그 호출부. 좌표계를 말하지 못하는 호출부가 있는지.
- **staleness 미탐지** → 술어의 **문턱**이지 술어가 보는 양이 아니다. 문턱을 옮기기 전에
  모집단을 재라(§6): 1.8× 를 낮추면 정당한 장평 압축 문단까지 재조판 대상이 된다.

### 진단은 진단으로 끝낸다

이 스킬의 산출물은 **판정과 피연산자**지 패치가 아니다. 특히 "업스트림에 있던 보정을 우리
술어에 옮겨 넣기" 는 여기서 하지 않는다 — 그 보정이 진짜 staleness 를 잡던 것인지 측정
격차를 메우던 것인지가 바로 이 진단이 답할 질문이고, 답하기 전에 옮기면 측정 결함을
조판 로직 안에 영구히 박는다. 판정을 보고한 다음, 고칠지 말지는 별도 결정이다.
