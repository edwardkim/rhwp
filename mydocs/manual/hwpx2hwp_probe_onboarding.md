---
kind: guide
status: active
canonical: mydocs/manual/hwpx2hwp_probe_onboarding.md
last_verified: 2026-07-17
---

# HWPX2HWP Probe 추적 온보딩

이 문서는 HWPX를 IR로 읽은 뒤 HWP로 저장할 때 발생하는 한컴 호환성 문제를 추적하는 방법을 설명한다.

대상 독자:

- `hwpx2hwp` 저장 버그를 처음 맡는 컨트리뷰터
- 한컴 에디터에서는 파일 손상/읽기 오류가 나지만 rhwp-studio에서는 열리는 문제를 분석하는 사람
- 정답 HWP와 생성 HWP를 비교하며 HWP record의 암묵 규칙을 찾아야 하는 사람

이 문서의 핵심은 하나다.

```text
추측으로 고치지 말고, 정답 HWP와 생성 HWP의 차이를 작은 probe로 쪼개서 한컴 판정 경계가 어디서 움직이는지 본다.
```

## 1. 왜 probe 방식이 필요한가

`hwpx2hwp` 경로는 다음 파이프라인을 지난다.

```text
HWPX XML + BinData
  -> HWPX parser
  -> rhwp IR
  -> HWP serializer
  -> 한컴 에디터 / rhwp-studio 재로드
```

rhwp-studio에서 정상 렌더링된다고 해서 한컴 에디터가 같은 방식으로 HWP record를 해석한다는 뜻은 아니다.

대표적인 차이:

- rhwp-studio는 IR을 직접 렌더링한다.
- 한컴 에디터는 저장된 HWP record 구조를 매우 엄격하게 읽는다.
- 한컴은 일부 compact record를 허용하지만, 특정 문맥에서는 raw tail이나 header tuple 누락을 파일 손상으로 본다.
- 셀 안 텍스트는 rhwp-studio에서 정상이어도 한컴에서는 `lineSeg`/paragraph record가 맞지 않으면 셀 위에 걸칠 수 있다.

따라서 문제를 다음처럼 쪼갠다.

```text
1. HWPX parser가 IR에 원본 의미를 제대로 올렸는가
2. HWP serializer가 한컴이 기대하는 record tuple을 썼는가
3. 한컴 파일손상 경계가 어떤 payload에서 뒤로 이동하는가
4. rhwp-studio 재로드는 계속 정상인가
```

## 2. 용어

| 용어 | 의미 |
|---|---|
| 정답 HWP | 한컴 에디터가 HWPX를 HWP로 저장한 reference 파일 |
| 생성 HWP | rhwp가 HWPX -> IR -> HWP로 저장한 파일 |
| probe | 특정 HWP record payload만 정답 HWP에서 복사/보정한 실험 파일 |
| 출력 경계 | 한컴이 파일손상 판정 전까지 화면에 출력한 마지막 문단/표 |
| tuple | 한 객체를 한컴이 안정적으로 읽기 위해 함께 맞아야 하는 record 묶음 |
| host paragraph | 표/그림 컨트롤을 들고 있는 상위 문단 |
| cell paragraph | 표 셀 내부 문단 |
| raw payload | HWP record에서 parser/serializer가 보존하거나 재구성해야 하는 원시 bytes |

## 3. 준비물

샘플 HWPX:

```text
samples/hwpx/<sample>.hwpx
```

정답 HWP:

```text
samples/hwpx/hancom-hwp/<sample>.hwp
```

정답 HWP가 없으면 작업지시자가 한컴 에디터에서 HWPX를 열고 HWP로 저장해 준비한다.

판정용 파일은 반드시 `output/` 아래에 생성한다.

```text
output/poc/hwpx2hwp/task<issue>/<stage_name>/
```

이 규칙은 중요하다. 작업지시자가 시각 판정을 할 파일은 `mydocs/`가 아니라 `output/`에서 찾을 수 있어야 한다.

## 4. 첫 진단

먼저 원본, 정답, 생성본을 같은 관점에서 본다.

```bash
target/debug/rhwp dump samples/hwpx/<sample>.hwpx -s 0 -p <para>
target/debug/rhwp dump samples/hwpx/hancom-hwp/<sample>.hwp -s 0 -p <para>
target/debug/rhwp dump output/poc/hwpx2hwp/task<issue>/<stage>/<generated>.hwp -s 0 -p <para>
```

요약 차이는 `ir-diff`를 사용한다.

```bash
cargo run --bin rhwp -- ir-diff \
  samples/hwpx/hancom-hwp/<sample>.hwp \
  output/poc/hwpx2hwp/task<issue>/<stage>/<generated>.hwp \
  --summary
```

주의:

```text
ir-diff는 방향을 정해주는 도구다.
한컴 파일손상 여부의 최종 판정자는 한컴 에디터다.
```

## 5. 한컴 판정 기록법

한컴 에디터 판정은 최소한 다음 네 가지를 기록한다.

| 항목 | 예시 |
|---|---|
| 판정 유형 | 파일 열림 / 파일손상 / 파일을 읽거나 저장하는데 오류 |
| 출력 위치 | `< 업종별 동향(억 달러, %) >` 까지 출력 |
| 배치 이상 | 셀 텍스트가 셀 영역 위에 걸침 |
| 조작 후 변화 | 스페이스 하나 입력하면 정상 배치 |

`파일손상`과 `파일을 읽거나 저장하는데 오류`는 구분한다.

```text
파일을 읽거나 저장하는데 오류:
  한컴이 더 이른 단계에서 record stream 자체를 못 읽는 경우가 많다.

파일손상:
  일부 문단/표를 그린 뒤 다음 record 진입 시 손상 판정을 내는 경우가 많다.
```

출력 위치는 매우 중요하다.

```text
어떤 variant에서 출력 경계가 뒤로 이동했다면,
그 variant가 건드린 payload는 한컴 파싱 경계에 영향을 준 것이다.
```

## 6. Probe 설계 원칙

Probe는 한 번에 한 축만 바꾼다.

좋은 variant:

```text
01_table_ctrl_header
02_table_record_with_tail
03_table_all_cell_headers
04_table_full_object_with_tail
05_host_para_plus_table_full_tuple
06_tuple_plus_next_boundary
```

나쁜 variant:

```text
01_everything_maybe_fixed
```

이유:

```text
한 번에 너무 많이 바꾸면 무엇이 한컴 경계를 움직였는지 알 수 없다.
```

Stage 이름은 문제 영역을 드러내야 한다.

```text
stage16_chart_tuple_probe
stage17_industry_table_probe
stage18_country_reflow_probe
```

## 7. 기본 probe 축

### 7.1 표 객체

표 하나가 의심될 때는 보통 다음 순서로 나눈다.

| 축 | 확인 대상 |
|---|---|
| `CTRL_HEADER` | 표 컨트롤의 common object attr raw |
| `TABLE record` | 행/열/spacing/padding/border/zone tail |
| `cell LIST_HEADER` | 셀 주소, span, size, borderFill, raw tail |
| `cell PARA_HEADER` | 셀 내부 문단 header extra |
| `full object` | 표 전체 clone |
| `host paragraph` | 표를 들고 있는 문단 record |
| `next boundary` | 다음 문단/표 진입부 |

한컴 출력 경계가 다음 title 또는 다음 표로 이동하면 해당 축은 유효 후보로 본다.

### 7.2 문단 텍스트

HWPX XML entity는 반드시 보존되어야 한다.

예:

```xml
<hp:t>&lt; 업종별 동향 &gt;</hp:t>
```

IR에서는 다음처럼 보여야 한다.

```text
< 업종별 동향 >
```

`quick-xml`은 `&lt;`, `&gt;`, `&amp;`를 `Event::GeneralRef`로 줄 수 있으므로, parser가 `Event::Text`만 읽으면 문자가 누락된다.

### 7.3 셀 재조판

한컴에서 셀 텍스트가 셀 영역 위에 걸치고, 스페이스 하나 입력 후 정상화된다면 다음을 의심한다.

```text
cell paragraph lineSeg
cell paragraph header extra
cell LIST_HEADER raw tail
TABLE record attr/tail
host paragraph lineSeg
```

이 경우 표 손상 probe와 별도로 reflow probe를 만든다.

예:

```text
07_industry_record_plus_cell_linesegs
08_industry_record_plus_cell_para_records
09_industry_record_headers_plus_cell_para_records
```

## 8. Probe 구현 위치

현재는 통합 테스트에 probe 생성기를 둔다.

```text
tests/hwpx_to_hwp_adapter.rs
```

장점:

- `cargo test`로 재현 가능
- 생성 파일 경로가 고정됨
- 변경이 회귀 테스트와 같이 남음
- 정답 HWP가 있는 경우 clone/graft 실험을 빠르게 반복 가능

기본 패턴:

```rust
let bytes = load_sample("hwpx-h-01.hwpx");
let mut core = DocumentCore::from_bytes(&bytes).expect("HWPX 로드 실패");

convert_if_hwpx_source(core.document_mut(), rhwp::parser::FileFormat::Hwpx);

let reference = task903_reference_hwpx_h_01_core();
let reference_table = task903_hwpx_h_01_some_table(&reference).clone();

// one-axis patch
task903_materialize_some_payload_from_reference(&mut core, &reference_table);

let hwp_bytes = core.export_hwp_native().expect("HWP 직렬화 실패");
std::fs::write(out_path, &hwp_bytes).expect("probe 파일 저장 실패");

let reloaded = DocumentCore::from_bytes(&hwp_bytes).expect("HWP 재로드 실패");
assert_eq!(reloaded.page_count(), 9);
```

## 9. 문서 작성 방식

각 stage 문서는 `mydocs/working/` 아래에 둔다.

```text
mydocs/working/task_m100_<issue>_stage<N>.md
```

필수 섹션:

```text
1. 단계 목적
2. 기준 파일
3. 사전 관찰
4. variant 설계
5. 생성 파일
6. 내부 검증
7. raw 확인
8. 작업지시자 판정 요청
9. 판정 해석
10. 다음 stage 제안
```

판정 요청 표 예:

```markdown
| variant | 한컴 판정 유형 | 한컴 출력 위치 | 셀 세로 배치 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|
| 01_table_ctrl_header |  |  |  |  |  |
| 02_table_record_with_tail |  |  |  |  |  |
```

## 10. 해석 규칙

### 출력 경계가 뒤로 이동한 경우

```text
해당 payload는 한컴 파싱 경계에 영향을 준다.
다음 stage는 이동한 경계의 다음 객체를 본다.
```

예:

```text
업종별 동향 표 보정 후 파일손상 위치가 국가별 동향 제목까지 이동
=> 다음 stage는 국가별 동향 표를 probe
```

### rhwp-studio만 정상인 경우

```text
IR 렌더링은 가능하지만 HWP record 저장 호환성이 부족한 상태다.
serializer/record tuple/raw tail 후보를 우선 본다.
```

### 한컴에서 편집하면 정상화되는 경우

```text
한컴 내부 재조판이 저장 record를 다시 계산하면서 고쳐지는 경우다.
lineSeg, paragraph header, cell layout cache를 본다.
```

### 한컴 정답과 generated가 모두 같은데 손상나는 경우

```text
보고 있는 위치가 원인이 아닐 수 있다.
출력 경계 바로 다음 record 또는 상위 host paragraph를 본다.
```

## 11. 주의사항

- 한컴 PDF는 정답지가 아니다. 한컴 에디터의 열기/저장/속성 판정이 우선이다.
- rhwp-studio 정상 렌더링은 충분조건이 아니다.
- `output/` 밖에 판정용 HWP를 만들지 않는다.
- 정답 HWP와 generated HWP의 문단 인덱스가 같다는 보장은 없다. `dump`로 확인한다.
- `TABLE record attr`만 맞는 것과 `CTRL_HEADER raw`까지 맞는 것은 다르다.
- `raw=[10, 03, ...]`에서 `raw=[10, 23, ...]`로 바뀌는 차이가 한컴 경계를 움직일 수 있다.
- stage 문서에는 실패도 기록한다. 실패한 probe가 다음 probe의 방향을 만든다.

## 12. #903 사례 요약

`samples/hwpx/hwpx-h-01.hwpx`에서 진행한 실제 추적 흐름:

```text
Stage 16:
  차트 표 tuple + 다음 제목 문단을 맞춤
  파일손상은 남음
  XML entity 누락(<, >)은 parser 버그로 확인하고 수정

Stage 17:
  문단 0:14 업종별 동향 표 probe
  TABLE/full tuple 보정 시 한컴 출력 경계가 < 국가별 동향... > 까지 이동
  셀 텍스트가 셀 영역 위에 걸치는 재조판 문제 확인

Stage 18:
  문단 0:21 국가별 동향 표 probe
  업종별 표 cell lineSeg/paragraph record 재조판 probe 분리
```

이 흐름은 다른 HWPX 샘플에도 그대로 적용할 수 있다.

## 13. 새 컨트리뷰터 체크리스트

작업 시작 전:

- [ ] 이슈 번호를 확인했다.
- [ ] HWPX 샘플 경로를 확인했다.
- [ ] 정답 HWP가 `samples/hwpx/hancom-hwp/`에 있는지 확인했다.
- [ ] 최초 generated HWP를 `output/poc/hwpx2hwp/task<issue>/stage0/`에 만들었다.
- [ ] 한컴 판정 유형과 출력 경계를 기록했다.

Probe 작성 전:

- [ ] 정답 HWP와 generated HWP를 `dump`로 비교했다.
- [ ] `ir-diff --summary`로 큰 차이 축을 확인했다.
- [ ] 한 번에 한 축만 바꾸는 variant 목록을 만들었다.
- [ ] 판정용 파일은 `output/` 아래에 생성되도록 했다.

Probe 작성 후:

- [ ] `cargo test --test hwpx_to_hwp_adapter <probe_test> -- --nocapture`가 통과했다.
- [ ] 생성 HWP가 rhwp로 재로드된다.
- [ ] stage 문서에 생성 파일과 판정 표를 적었다.
- [ ] 작업지시자 한컴 판정 결과를 stage 문서에 반영했다.
- [ ] 출력 경계가 이동했다면 다음 stage 후보를 적었다.

## 14. 관련 문서

- [온보딩 가이드](onboarding_guide.md)
- [하이퍼-워터폴](hyper_waterfall.md)
- [ir-diff 명령](ir_diff_command.md)
- [dump 명령](dump_command.md)
- [HWPX lineseg reflow trap](../troubleshootings/hwpx_lineseg_reflow_trap.md)
- [BinData ID/index mapping](../troubleshootings/bin_data_id_index_mapping.md)
