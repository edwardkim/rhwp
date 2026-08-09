# 최종 보고서 — task_m100_4085

- **Issue**: [#4085](https://github.com/edwardkim/rhwp/issues/4085)
- **브랜치**: `fix/charoverlap-noborder-charsz` (`origin/devel` `d76d4e98b` 기준 — 작업은 `570fa6e4f`
  에서 시작해 rebase)
- **계획서**: [`mydocs/plans/task_m100_4085.md`](../plans/task_m100_4085.md)
- **단계 기록**: [`mydocs/working/task_m100_4085_stage1.md`](../working/task_m100_4085_stage1.md)
- **작성 시각**: 2026-08-06 KST

## 1. 요약

글자겹침(CharOverlap)의 `charSz`(IR `inner_char_size`) 축소가 **테두리를 그리지 않는 겹침에도**
적용돼, 한컴 대비 60% 크기로 렌더되던 결함을 고쳤다. 규칙을 `composer.rs` 한 곳으로 모아
SVG·CanvasKit·Skia 세 경로의 불일치도 함께 해소했다.

증상 문서: `관세청/156636617_240617 2024년 5월 월간 수출입 현황(확정치).hwp` 1페이지 절 제목의
번호 상자(U+F02B1, 한컴 PUA "사각 안 1").

## 2. 원인

`charSz` 는 OWPML 스키마상 **"테두리 내부 글자의 크기 비율. 단위 %"**
(`mydocs/manual/OWPML SCHEMA/ParaList XML schema.xml:571`)다. 그런데 렌더 경로는 `border_type` 을
보지 않고 항상 축소했다. 이 문서는 `border_type=0`(`circleType="CHAR"`, 테두리 없음)이라 축소할
"테두리 내부"가 존재하지 않는데도 `charSz=-4 → 0.60` 이 적용됐다.

음수 `charSz` → 10% step 축소 규칙은 PR #1101 에서 도입됐고, 당시 실측 fixture 는
`SHAPE_REVERSAL_RECTANGLE`(border_type=4) 한 건뿐이었다. PR 리뷰 문서도 미검증 가설로 명시했다:

> 음수 → 10% step 영역 가설은 권위 미입증이지만 합리적이며 실측 정합. (…) 한컴 2020/2022 영역
> fixture 추가 시 재검증 권장 — `mydocs/pr/archives/pr_1101_review.md:90,164`

본 작업이 그 재검증이다.

### 두 오라클

| fixture | border_type | charSz | 한컴 실측 | 근거 |
| --- | --- | --- | --- | --- |
| `samples/hwpx/k-water-rfp.hwpx` p13 | 4 (`SHAPE_REVERSAL_RECTANGLE`) | -2 | 0.80 배 축소 | PR #1101 시각 검증 |
| 관세청 월간 수출입 현황 p1 | 0 (`CHAR`, 테두리 없음) | -4 | **축소 없음** | 본 작업, 한컴 PDF content stream |

한컴 PDF content stream — 마커와 뒤따르는 본문이 **같은 글자 크기, 같은 baseline**:

```
/F5 101 Tf  2 Tr 2.02 w   1 0 0 -1  335 1613 Tm  [<0003>]TJ   ← 글자겹침 마커
/F6 101 Tf  2 Tr 2.02 w   1 0 0 -1  436 1613 Tm  [<0001>]TJ   ← 뒤따르는 본문
```

마커 폭 335→436 = 101 = 정확히 1em.

## 3. 수정

### 3.1 규칙 단일화

`src/renderer/composer.rs` 에 `char_overlap_size_ratio(effective_border, inner_char_size)` 신설.
종전에는 같은 규칙이 5곳에 인라인으로 흩어져 있었고, 그중 `skia/text_replay.rs` 만 음수 분기가 없어
**같은 문서가 SVG/CanvasKit 60% vs PNG 100%** 로 갈렸다.

### 3.2 게이트 기준 — raw `border_type` 이 아니라 `effective_border`

`draw_char_overlap_combined` 는 `border_type == 0` 이어도 PUA 다자리 숫자면 원형 테두리로 승격한다.
그 경로는 테두리를 **실제로 그리므로** 축소가 정당하다. `effective_border` 기준이면 combined 경로는
절대 0이 아니라 동작이 바뀌지 않는다 — raw `border_type` 으로 게이트했다면 table-vpos-01 계열
다자리 마커가 회귀했을 것이다.

### 3.3 변경 파일 (6개, +136/-47)

| 파일 | 변경 |
| --- | --- |
| `src/renderer/composer.rs` | `char_overlap_size_ratio` 신설 |
| `src/renderer/svg.rs` | `draw_char_overlap` / `draw_char_overlap_combined` → 헬퍼 호출 |
| `src/renderer/web_canvas.rs` | 동일 2함수 |
| `src/renderer/skia/text_replay.rs` | `effective_border` 계산을 `size_ratio` 앞으로 이동 후 헬퍼 호출 |
| `src/renderer/composer/tests.rs` | 규칙 단위 테스트 5케이스 |
| `src/renderer/svg/tests.rs` | SVG 출력 회귀 2건 |

## 4. 검증

| 게이트 | 결과 |
| --- | --- |
| focused test (`--lib char_overlap`) | ✅ 10 passed / 0 failed |
| `cargo clippy --all-targets -- -D warnings` | ✅ exit 0 |
| `cargo fmt --check` | ✅ 실제 포맷 diff 0 |
| `cargo test --profile release-test --tests --no-fail-fast` | ⚠️ 465 스위트 ok / 3 FAILED — 전부 본 변경 무관 (5절) |
| Native Skia `skia --lib` | ✅ 58 passed |
| Native Skia `issue_2225_missing_picture_placeholder` | ✅ 2 passed |
| Native Skia `render_p37_direct_pdf_export` | ✅ 4 passed |
| WASM 빌드 (Docker) | ✅ 전 단계 통과 — 컴파일·wasm-bindgen·wasm-opt·packaging (6절) |

`cargo fmt --check` 는 1057줄의 `Incorrect newline style` 을 내지만 `Diff in` 은 0건이다. 이
checkout 의 `core.autocrlf=true` 가 만드는 환경 아티팩트로 저장소 전 파일에 뜬다.

### 4.1 시각 증적

**관세청 p1 (수정 대상)** — `output/poc/task4085/after/`

| | 수정 전 | 수정 후 | 한컴 |
| --- | --- | --- | --- |
| 마커 `font-size` | `13.60` | `22.67` | 본문과 동일 |
| 본문 `font-size` | `22.67` | `22.67` | — |
| 비율 | 0.60 | **1.00** | 1.00 |

render tree — 좌여백 20mm(75.6px)에서 정확히 1em:

```json
{"type":"TextRun","text":"󰊱","bbox":{"x":75.6,"y":344.3,"w":22.7,"h":22.7}}
```

뒤따르는 `’` 위치: 한컴 109.7px vs rhwp 109.3px — 정합.

**k-water-rfp p13 (회귀 금지)** — `output/poc/task4085/kwater/`

```
<rect x="193.68" y="436.29" width="22.67" height="22.67" fill="#000000" .../>
<text x="205.01" y="447.62" fill="#FFFFFF" font-size="18.13" ...>3</text>
```

`font-size = 18.13` = 22.66 × 0.80 — PR #1101 보고서 기록값과 동일. 반전 사각형 3개 모두 유지.

## 5. release-test 실패 3건 — 전부 본 변경 무관 (확인 완료)

`--no-fail-fast` 없이 돌리면 첫 실패에서 cargo 가 멈춰 37 스위트만 실행된다. 나머지를 덮기 위해
`--no-fail-fast` 로 재실행한 결과가 **465 스위트 ok / 3 FAILED** 다.

| 실패 테스트 | 스위트 | 지목한 파일 |
| --- | --- | --- |
| `every_normal_sample_is_clean` | `injection_scan_contract` | ILO최종본 |
| `ir_field_sweep_does_not_regress` | `ir_field_sweep_baseline` | ILO최종본 + 대중국수산물 |
| `negative_corpus_sweep_is_clean_across_all_three_detectors` | `security_corpus_regression` | ILO최종본 |

3건 모두 `samples/` 에 **untracked 로 놓여 있던** 파일을 이름으로 지목한다. 이 파일들은 세션 시작
시점 `git status` 에 이미 있었고 본 작업의 산출물이 아니며, 커밋 대상도 아니라 CI 에는 존재하지
않는다.

- 1·3번은 같은 원인이다 — ILO 보고서 본문 한 문장이 `instruction_override` 규칙에 오탐된다.
  범위어(`모든`)·목적어(`지시`)·서술어(`무시`)가 **문장 경계를 넘어 한 창 안에** 들어왔다.
  별건으로 [#4088](https://github.com/edwardkim/rhwp/issues/4088) 등록.
- 2번은 성격이 다르다. 새 파일이 baseline 에 없는 IR 왕복 발산을 더한다
  (`char_count 0→56`, `char_offsets[] 0→907`, `list_header_width_ref 0→981` 등 5건).

### 5.1 격리 재실행으로 인과 확정

추정에 그치지 않도록, **본 변경이 들어간 같은 테스트 바이너리**로 세 스위트를 재실행했다. 변수는
untracked 샘플 3개의 존재 여부 하나뿐이다.

| 스위트 | 파일 있음 | 파일 격리 |
| --- | --- | --- |
| `injection_scan_contract` | FAILED (13/14) | **ok. 14 passed** |
| `ir_field_sweep_baseline` | FAILED (1/2) | **ok. 2 passed** |
| `security_corpus_regression` | FAILED (5/6) | **ok. 6 passed** |

세 파일은 원위치로 복구했다. 본 변경은 `char_overlap_size_ratio` 와 이를 쓰는 draw 함수 3곳뿐이라
injection 스캔·IR 왕복 경로에 닿지 않는다.

## 6. WASM 빌드 — 환경 결함 우회

문서화된 네이티브 경로(`dev_environment_guide.md:79`)의 `wasm-pack build` 가 `wasm-opt` 단계에서
무한 재실행돼 종료되지 않았다(37분). 별건으로 [#4089](https://github.com/edwardkim/rhwp/issues/4089)
등록. 저장소가 규정한 공식 경로인 Docker(`mydocs/tech/wasm_pack_version_policy.md`)로 전환했다.

Docker 경로에서 만난 환경 제약 2건도 이슈에 기록했다:

1. `/app/target` 하드링크 불가 — Docker Desktop Windows 바인드 마운트 제약
   (`failed to link or copy ... Operation not permitted`). `CARGO_TARGET_DIR` 를 컨테이너 내부
   경로로 돌려 우회.
2. Git Bash 의 MSYS path mangling 이 `-e CARGO_TARGET_DIR=/tmp/...` 를 Windows 경로로 변환
   (`/app/C:/Users/...`). PowerShell 에서 실행해 우회.

### 6.1 1차 시도 — packaging 단계에서 환경 실패

```
    Finished `release` profile [optimized] target(s) in 4m 00s
[INFO]: Installing wasm-bindgen...
[INFO]: Optimizing wasm binaries with `wasm-opt`...
Error: failed to copy README
Caused by: Operation not permitted (os error 1)
```

컴파일과 wasm-bindgen 은 통과했으나 README 복사에서 실패했고, wasm-opt 적용 여부도 확정하지
못했다(`pkg/rhwp_bg.wasm` 7,608,638 바이트가 최적화 이전과 동일).

### 6.2 원인 — 출력 디렉터리가 바인드 마운트였다

`--out-dir pkg` 는 `/app/pkg`, 즉 Windows 바인드 마운트 위다. `CARGO_TARGET_DIR` 을 컨테이너
내부로 돌렸던 것과 **같은 제약이 출력 경로에도 걸린다**는 것을 1차 시도에서는 놓쳤다. 출력까지
컨테이너 내부(`--out-dir /tmp/pkg`)로 돌리자 전 단계가 통과했다.

```console
$ docker run --rm -v D:\rhwp:/app -e CARGO_TARGET_DIR=/tmp/target \
    rhwp-wasm:latest wasm-pack build --target web --out-dir /tmp/pkg
```

```
   Compiling rhwp v0.8.2 (/app)
    Finished `release` profile [optimized] target(s) in 3m 40s
[INFO]: Installing wasm-bindgen...
[INFO]: Optimizing wasm binaries with `wasm-opt`...
[INFO]: :-) Done in 7m 47s
[INFO]: :-) Your wasm pkg is ready to publish at /tmp/pkg.
```

| 단계 | 결과 |
| --- | --- |
| Rust → wasm32 컴파일 | ✅ 3m 40s |
| wasm-bindgen | ✅ |
| wasm-opt | ✅ 완료 (1차의 미확인 항목 해소) |
| packaging (README·package.json) | ✅ 완료 |

**게이트 전체가 통과했다.** 1차에서 남았던 두 미해결 항목(wasm-opt 적용 여부, README 복사 실패)은
본 변경이 아니라 출력 경로 선택의 문제였음이 확인됐다. CI 의 Linux 러너는 바인드 마운트를 쓰지
않으므로 애초에 해당하지 않는다.

## 7. 남긴 것

- **마커 세로 정렬**: 한컴은 baseline 기준, rhwp 는 `dominant-baseline="central"`. 크기 정정 후
  시각 대조에서 차이가 관측되지 않아 범위에서 제외했다. 별도 관측 시 후속 이슈로 분리한다.
- **폰트 폴백 체인**: `'HCR Batang Ext-B'` 가 Windows 실제 패밀리명(`HCR Batang ExtB`)과 불일치하고,
  U+F02B1 은 확장/확장B 가 아니라 일반 `HCR Batang` 에 있다.
  [#4086](https://github.com/edwardkim/rhwp/issues/4086) 로 분리 — 전 문서 영향축이라 시각 회귀
  범위가 크게 달라진다.

## 8. 파생 이슈

| 이슈 | 내용 |
| --- | --- |
| [#4086](https://github.com/edwardkim/rhwp/issues/4086) | 폰트 폴백 체인 이름 불일치 + PUA 글리프 소재 오인 |
| [#4088](https://github.com/edwardkim/rhwp/issues/4088) | 한국어 `instruction_override` 규칙이 절 경계를 넘어 오탐 |
| [#4089](https://github.com/edwardkim/rhwp/issues/4089) | Windows 네이티브 `wasm-pack` 이 `wasm-opt` 무한 재실행 |

## 9. 재현·오라클 획득 방법

```bash
rhwp dump "<파일>" -s 0            # 문단 0.5 의 글자겹침 컨트롤 파라미터
rhwp export-svg "<파일>" -p 0      # 마커 font-size 확인
rhwp export-render-tree "<파일>" -p 0
```

한컴 오라클은 COM 자동화로 PDF 저장 후 content stream 을 직접 파싱했다. 화면 캡처보다 정확한
수치(글자 크기·baseline·advance)를 얻기 위함이다.

```powershell
$hwp = New-Object -ComObject "HWPFrame.HwpObject"
$hwp.RegisterModule("FilePathCheckDLL", "FilePathCheckerModule")
$hwp.Open($src, "HWP", "forceopen:true")
$hwp.SaveAs($out, "PDF", "")
```

**주의**: 이 경로로 나온 PDF 는 한컴의 저장된 인쇄 설정 때문에 A4 landscape(841×595pt)에 0.708 배로
축소돼 들어간다. 균일 스케일이라 비율 비교에는 영향이 없지만 절대 pt 값을 읽을 때는 보정해야 한다.
`/MediaBox` 와 페이지 `cm` 행렬을 먼저 확인할 것.
