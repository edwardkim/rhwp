# 구현계획서 — Task #1116: sample16 목차 및 문단 간격 한컴오피스 정합

- 이슈: [edwardkim/rhwp#1116](https://github.com/edwardkim/rhwp/issues/1116)
- 수행계획서: `mydocs/plans/task_m100_1116.md`
- 브랜치: `local/task1116`
- 작성일: 2026-05-25
- 상태: 구현 및 검증 완료, 커밋/PR 준비 단계

## 1. 구현 전제

작업지시자 제공 캡처의 격자는 3mm 단위다. 기존 1차 분석 문서의 "1mm 격자 기준 육안 추정"은 폐기하고, rhwp 좌표만 재사용한다.

핵심 비교 단위:

```text
1mm = 283.465 HWPUNIT
3mm = 850.395 HWPUNIT
96dpi 기준 1mm = 3.7795 px
96dpi 기준 3mm = 11.3386 px
```

p3 본문에서 기존 분석의 주요 수치:

```text
목적 박스 하단 -> `2. 추진방향` 제목 시작:
2664 HU = 약 9.40mm = 약 3.13칸(3mm 격자)
```

따라서 구현 전 Stage 1에서 한컴 캡처상 같은 구간이 실제로 몇 칸인지 다시 확인한다.

## 2. Stage 1 — 재현 기준선 재측정

### 2.1 명령

```bash
target/debug/rhwp dump-pages samples/hwp3-sample16-hwp5.hwp -p 1
target/debug/rhwp dump-pages samples/hwp3-sample16-hwp5.hwp -p 2
target/debug/rhwp dump-pages samples/hwp3-sample16.hwp -p 1
target/debug/rhwp dump-pages samples/hwp3-sample16.hwp -p 2
```

p3 상단 문단 상세:

```bash
target/debug/rhwp dump samples/hwp3-sample16-hwp5.hwp -s 0 -p 69
target/debug/rhwp dump samples/hwp3-sample16-hwp5.hwp -s 0 -p 70
target/debug/rhwp dump samples/hwp3-sample16-hwp5.hwp -s 0 -p 71
target/debug/rhwp dump samples/hwp3-sample16-hwp5.hwp -s 0 -p 72
target/debug/rhwp dump samples/hwp3-sample16-hwp5.hwp -s 0 -p 73
target/debug/rhwp dump samples/hwp3-sample16-hwp5.hwp -s 0 -p 74
```

SVG 기준선:

```bash
target/debug/rhwp export-svg samples/hwp3-sample16-hwp5.hwp \
  -o output/debug/task1116/hwp5-p2 \
  -p 1 \
  --debug-overlay \
  --show-control-codes

target/debug/rhwp export-svg samples/hwp3-sample16-hwp5.hwp \
  -o output/debug/task1116/hwp5-p3-grid \
  -p 2 \
  --show-grid \
  --debug-overlay \
  --show-control-codes
```

### 2.2 기록 형식

Stage 1 보고서에는 다음 표를 반드시 넣는다.

| 구간 | HU | mm | 3mm 격자 칸수 | rhwp SVG y(px) | 한컴 캡처 칸수 |
|------|---:|---:|---:|---:|---:|
| `I. 사업개요` -> `1. 추진목적` | | | | | |
| `1. 추진목적` -> 목적 박스 top | | | | | |
| 목적 박스 height | | | | | |
| 목적 박스 bottom -> `2. 추진방향` | | | | | |
| `2. 추진방향` -> 첫 본문 줄 | | | | | |

## 3. Stage 2 — p2 목차 구현 방향

### 3.1 분석 대상

목차 문단의 `TabDef`와 텍스트/컨트롤 조합을 추적한다.

```text
tab_def_id=2
tabs=[
  pos=27504 (약 97.0mm) type=left fill=dot,
  pos=87720 (약 309.5mm) type=right fill=dot
]
```

점검 항목:

1. right-tab 기준 x가 문단 사용 폭, 본문 폭, 종이 폭 중 어디에 묶이는지 확인한다.
2. leader `<line>` 또는 점선 생성이 페이지 번호 glyph와 겹치는지 확인한다.
3. `TopAndBottom + treat_as_char=true` 사각형 컨트롤이 탭 측정 시작 x를 밀거나 leader 길이를 잘못 줄이는지 확인한다.
4. 기존 KTX 목차 테스트가 기대하는 right edge와 충돌하지 않는지 확인한다.

### 3.2 테스트 후보

신규 테스트:

```text
tests/issue_1116.rs
```

테스트 항목:

1. `hwp3-sample16-hwp5.hwp` p2에서 페이지 번호 text의 right edge 산포가 작아야 한다.
2. leader 점선 종료 x가 페이지 번호 시작 x를 침범하지 않아야 한다.
3. 기존 `tests/issue_874_ktx_toc_page_number_right_align.rs`는 동일하게 통과해야 한다.

구현 위치 후보:

- `src/renderer/layout/paragraph_layout.rs`
- `src/renderer/layout/text_measurement.rs`
- `src/renderer/typeset.rs`

## 4. Stage 3 — p3 본문 간격 구현 방향

### 4.1 분석 대상

우선 `pi=71 -> pi=73` 구간을 좁게 본다.

```text
pi=71 TAC 글상자 문단
pi=72 빈 문단
pi=73 `2. 추진방향`
```

현재 파일상 분해:

```text
2664 HU
= 780(pi=71 line_spacing)
+ 568(pi=72 spacing_before)
+ 300(pi=72 line_height)
+ 164(pi=72 line_spacing)
+ 852(pi=73 spacing_before)
```

구현 전에 확인할 질문:

1. 한컴은 `pi=72` 빈 문단의 `spacing_before`와 line height를 모두 진행량에 반영하는가.
2. rhwp는 `LINE_SEG.vpos` 보정 후 `spacing_before`를 다시 더해 중복이 생기는가.
3. TAC shape의 실제 bbox height와 `LINE_SEG.lh=9764`가 같은 기준인가.
4. 3mm 격자 기준으로 볼 때 실제 차이가 보정이 필요한 수준인가.

### 4.2 구현 후보

소스 수정은 승인 후 다음 우선순위로 진행한다.

1. `LINE_SEG`가 존재하는 HWP5 문단에서 다음 문단 top을 산정할 때, 직전 `vpos + lh + ls`와 현재 `vpos` 차이를 중복 적용하지 않도록 조건을 좁힌다.
2. 빈 문단이 `LINE_SEG.lh`와 `spacing_before`를 동시에 갖는 경우, 한컴 편집기와 같은 진행량인지 별도 분기로 검증한다.
3. TAC 글상자 문단 뒤 간격 보정은 `treat_as_char=true`, shape height와 `LINE_SEG.lh`가 큰 폭으로 일치하는 경우로만 제한한다.
4. 일반 HWP5/HWPX 문서에는 보정이 발동하지 않도록 구조 조건을 둔다.

구현 위치 후보:

- `src/renderer/height_measurer.rs`
- `src/renderer/typeset.rs`
- `src/renderer/pagination/engine.rs`

## 5. Stage 4 — 검증

신규 테스트:

```bash
cargo test --test issue_1116 -- --nocapture
```

기존 회귀:

```bash
cargo test --test issue_1105 -- --nocapture
cargo test --test issue_1086 -- --nocapture
cargo test --test issue_874_ktx_toc_page_number_right_align -- --nocapture
cargo test --test issue_630 -- --nocapture
cargo fmt --all -- --check
git diff --check
```

샘플별 페이지 수 확인:

```bash
target/debug/rhwp dump-pages samples/hwp3-sample16-hwp5.hwp | tail -1
target/debug/rhwp dump-pages samples/hwp3-sample16-hwp5-2010.hwp | tail -1
target/debug/rhwp dump-pages samples/hwp3-sample16-hwp5-2018.hwp | tail -1
target/debug/rhwp dump-pages samples/hwp3-sample16-hwp5-2022.hwp | tail -1
target/debug/rhwp dump-pages samples/hwp3-sample16-hwp5-2024.hwp | tail -1
```

## 6. 승인 후 첫 작업 순서

1. Stage 1 보고서 작성: 3mm 격자 환산표와 p2/p3 현재 값 정리.
2. `tests/issue_1116.rs`에 p2 목차와 p3 진행량 회귀 가드를 추가한다.
3. p2 목차 보정을 먼저 적용한다.
4. p3 TAC 글상자 이후 간격 보정을 별도로 적용한다.
5. 전체 변환본 5종과 #1105/#1086/#874 회귀 테스트를 실행한다.
6. 작업지시자 한컴오피스 시각 판정 요청 후 최종 보고서와 PR 준비로 넘어간다.

## 7. 승인 게이트

이 구현계획서 승인 전까지 소스 파일은 수정하지 않는다.
