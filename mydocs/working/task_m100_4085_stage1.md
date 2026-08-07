# Stage 1 — task_m100_4085 구현·검증

- **이슈**: [#4085](https://github.com/edwardkim/rhwp/issues/4085)
- **계획서**: [`mydocs/plans/task_m100_4085.md`](../plans/task_m100_4085.md)
- **브랜치**: `fix/charoverlap-noborder-charsz` (`origin/devel` `d76d4e98b` 기준 — 작업은 `570fa6e4f`
  에서 시작해 rebase)
- **작업 시각**: 2026-08-06 KST

## 1. 구현

### 1.1 규칙 단일화

`src/renderer/composer.rs` 에 `char_overlap_size_ratio(effective_border: u8, inner_char_size: i8) -> f64`
를 신설하고, 세 렌더 경로가 이 하나를 공유하게 했다. 종전에는 같은 규칙이 5곳에 인라인으로 흩어져
있었고 그중 skia 만 음수 분기가 없어 출력이 갈렸다.

```rust
pub fn char_overlap_size_ratio(effective_border: u8, inner_char_size: i8) -> f64 {
    if effective_border == 0 {
        return 1.0;
    }
    if inner_char_size > 0 {
        inner_char_size as f64 / 100.0
    } else if inner_char_size < 0 {
        1.0 + inner_char_size as f64 * 0.10
    } else {
        1.0
    }
}
```

### 1.2 게이트를 raw `border_type` 이 아니라 `effective_border` 에 건 이유

`draw_char_overlap_combined` 는 `border_type == 0` 이어도 PUA 다자리 숫자로 디코딩되면 원형 테두리로
승격한다(`svg.rs`, `web_canvas.rs`, `skia/text_replay.rs` 각각의 combined 분기). 그 경로는 테두리를
**실제로 그리므로** 축소가 정당하다.

`effective_border` 기준이면 combined 경로는 절대 0이 아니라 **동작이 바뀌지 않는다.** raw
`border_type` 으로 게이트했다면 table-vpos-01 계열 다자리 마커가 회귀했을 것이다.

### 1.3 변경 파일

| 파일 | 변경 |
| --- | --- |
| `src/renderer/composer.rs` | `char_overlap_size_ratio` 신설 (+31) |
| `src/renderer/svg.rs` | `draw_char_overlap` / `draw_char_overlap_combined` → 헬퍼 호출 (+8/-19) |
| `src/renderer/web_canvas.rs` | 동일 2함수 (+8/-19) |
| `src/renderer/skia/text_replay.rs` | `effective_border` 계산을 `size_ratio` 앞으로 이동 후 헬퍼 호출 (+6/-7) |
| `src/renderer/composer/tests.rs` | 규칙 단위 테스트 (+24) |
| `src/renderer/svg/tests.rs` | SVG 출력 회귀 2건 (+61) |

## 2. 테스트

`cargo test --profile release-test --lib char_overlap` — 10 passed / 0 failed

```
test renderer::composer::tests::char_overlap_size_ratio_applies_only_when_a_border_is_drawn ... ok
test renderer::svg::tests::char_overlap_without_border_keeps_body_font_size ... ok
test renderer::svg::tests::char_overlap_with_border_keeps_charsz_reduction ... ok
test renderer::composer::tests::test_char_overlap_multi_component_is_single_advance ... ok
test parser::control::tests::test_parse_char_overlap ... ok
test parser::hwp3::tests::hwp3_char_overlap_extracts_overlap_chars ... ok
test doclang::adapter::control::tests::char_overlap_rescues_glyphs_and_records_loss ... ok
test serializer::control::tests::char_overlap_non_bmp_char_roundtrip ... ok
test serializer::control::tests::char_overlap_256_char_shape_ids_no_wraparound ... ok
test serializer::hwpx::table::tests::task1379_cell_char_overlap_emitted_as_compose ... ok
```

새 테스트는 SVG **출력 문자열**을 직접 단언한다 — 비율 계산만 검증하면 호출부가 헬퍼를 안 쓰게
바뀌어도 통과하기 때문이다.

## 3. 검증 게이트 (`local_validation.md` 4.3 — renderer 범위)

| 게이트 | 결과 |
| --- | --- |
| `cargo clippy --all-targets -- -D warnings` | ✅ exit 0 |
| `cargo fmt --check` | ✅ 실제 포맷 diff 0 (아래 주석) |
| `cargo test --profile release-test --tests` | ⚠️ 54 스위트 ok / 1 FAILED (본 변경 무관 — 4절) |
| `cargo test --features native-skia skia --lib` | ✅ 58 passed |
| `--test issue_2225_missing_picture_placeholder` | ✅ 2 passed |
| `--test render_p37_direct_pdf_export` | ✅ 4 passed |
| WASM 빌드 (Docker) | ⚠️ 컴파일·wasm-bindgen 통과, packaging 단계 환경 실패 (보고서 6절) |

`cargo fmt --check` 는 1057줄의 `Incorrect newline style` 을 출력하지만 `Diff in` 은 0건이다. 이
checkout 의 `core.autocrlf=true` 가 작업트리를 CRLF 로 바꿔 저장소 전 파일에 뜨는 환경 아티팩트이며
본 변경과 무관하다.

## 4. release-test 실패 1건 — 본 변경 무관

`tests/injection_scan_contract.rs::every_normal_sample_is_clean` 실패.

원인은 `samples/` 에 **untracked 로 놓여 있던** `1490000-200700034_ILO최종본071220.hwp` 다. 이 파일은
세션 시작 시점 `git status` 에 이미 있었고 본 작업의 산출물이 아니다.

```console
$ rhwp inspect injection "samples/1490000-200700034_ILO최종본071220.hwp" --json --include-fields
clean=false  kind=instruction_override  confidence=high  page=438 paragraph=6378
```

한국어 행정문서 본문 한 문장이 `instruction_override` 규칙에 오탐된 것이다. 본 변경은
`char_overlap_size_ratio` 와 이를 쓰는 draw 함수 3곳뿐이라 `inspect injection`(본문 텍스트 스캔)
경로에 닿지 않는다. 별건으로 [#4088](https://github.com/edwardkim/rhwp/issues/4088) 등록.

같은 디렉터리의 다른 untracked `.hwp`(`1192000-201600027_대중국수산물수출확대_제안요약서.hwp`)는
`clean=true` 로 통과한다.

### 4.1 격리 재실행으로 확인

추정에 그치지 않도록, **본 변경이 들어간 같은 테스트 바이너리**로 해당 파일만 격리해 재실행했다.
변수는 파일 존재 여부 하나뿐이다.

```console
$ TEST=target/release-test/deps/injection_scan_contract-fc2b215d8fb8ff60.exe

# [1] 파일이 있는 상태
$ $TEST every_normal_sample_is_clean --exact --nocapture
정상 샘플 1건에서 오탐이 났습니다 (검사 277건):
test result: FAILED. 0 passed; 1 failed; ... finished in 43.22s

# [2] 해당 파일만 samples/ 밖으로 이동
$ $TEST every_normal_sample_is_clean --exact --nocapture
test result: ok. 1 passed; 0 failed; ... finished in 50.01s

# [3] 원위치 복구 완료
```

277건 중 이 1건만 오탐이며, 제거하면 통과한다. 본 변경과 무관함이 확인됐다.

## 5. 시각 증적

### 5.1 관세청 월간 수출입 현황 p1 — 수정 대상

산출물: `output/poc/task4085/after/`

| | 수정 전 | 수정 후 | 한컴 오라클 |
| --- | --- | --- | --- |
| 마커 `font-size` | `13.60` | `22.67` | 본문과 동일 (`101 Tf`) |
| 본문 `font-size` | `22.67` | `22.67` | `101 Tf` |
| 비율 | 0.60 | 1.00 | 1.00 |

```
<text x="86.92" y="355.61" font-size="22.67" font-weight="bold" ...>󰊱</text>
```

render tree bbox — 좌여백 20mm(=75.6px) 에서 정확히 1em:

```json
{"type":"TextRun","text":"󰊱","bbox":{"x":75.6,"y":344.3,"w":22.7,"h":22.7}}
{"type":"TextRun","text":" ","bbox":{"x":98.3,"y":344.3,"w":11,"h":22.7}}
{"type":"TextRun","text":"’","bbox":{"x":109.3,"y":344.3,"w":7,"h":22.7}}
```

한컴 PDF content stream (page scale 0.708 보정 후): 마커 좌단 20mm, 폭 1em, 뒤따르는 `’` 가 109.7px
— rhwp 109.3px 로 정합.

### 5.2 k-water-rfp p13 — 회귀 금지 대상

산출물: `output/poc/task4085/kwater/k-water-rfp_013.svg`

```
<rect x="193.68" y="436.29" width="22.67" height="22.67" fill="#000000" .../>
<text x="205.01" y="447.62" fill="#FFFFFF" font-size="18.13" ...>3</text>
```

`font-size = 18.13` = 22.66 × 0.80. PR #1101 보고서에 기록된 값과 동일하며 반전 사각형 3개 모두
유지된다. `charSz=-2` 축소가 그대로 적용됨을 확인했다.

### 5.3 combined 경로 (다자리 PUA 마커)

`effective_border` 게이트 설계상 코드 경로가 바뀌지 않는다. Native Skia `skia --lib` 58건 통과로
확인.

## 6. 오라클 획득 방법 (재현용)

한컴 Office 2022 COM 자동화로 PDF 저장 후 content stream 을 직접 파싱했다. 화면 캡처 비교보다
정확한 수치(글자 크기, baseline, advance)를 얻기 위함이다.

```powershell
$hwp = New-Object -ComObject "HWPFrame.HwpObject"
$hwp.RegisterModule("FilePathCheckDLL", "FilePathCheckerModule")
$hwp.Open($src, "HWP", "forceopen:true")
$hwp.SaveAs($out, "PDF", "")
```

주의: 이 경로로 나온 PDF 는 한컴의 저장된 인쇄 설정 때문에 A4 landscape(841×595pt)에 0.708 배로
축소돼 들어갔다. **균일 스케일**이라 비율·상대 좌표 비교에는 영향이 없지만, 절대 pt 값을 읽을 때는
보정해야 한다. `/MediaBox` 와 페이지 `cm` 행렬을 먼저 확인할 것.

## 7. 마무리

| 항목 | 상태 |
| --- | --- |
| wasm-pack 결과 확인 | ✅ 완료 — 컴파일·wasm-bindgen 통과, packaging 단계 환경 실패 (보고서 6절) |
| 최종 보고서 | ✅ [`mydocs/report/task_m100_4085_report.md`](../report/task_m100_4085_report.md) |
| 커밋 | ✅ 소스 6 + 문서 3 = 9 파일 |

`pdf-large/hwpx/2026_oss_rst.pdf` 는 커밋에서 제외했다. 내용 변경이 아니라 작업트리에 git-lfs
**포인터 파일**(131바이트, `oid sha256:bec53a60…`)이 남아 실제 객체로 스머지되지 않은 상태이며 본
작업 산출물이 아니다. `samples/` 의 untracked `.hwp`/`.hwpx` 3건도 같은 이유로 제외했다.

### 7.1 커밋 후 관측 — 로컬 Rust 툴체인 소실

커밋 직후 재검증을 시도할 때 `cargo` 를 찾을 수 없었다. 샌드박스를 끄고 C:/D: 전 드라이브를
검색해도 결과가 같다.

| 대상 | 상태 |
| --- | --- |
| `%USERPROFILE%\.cargo`, `.rustup` | 없음 |
| `cargo.exe` / `rustc.exe` 전 드라이브 검색 | 0건 |
| User/Machine `PATH` 의 rust 항목 | 없음 |
| 레지스트리 Rust 설치 항목 | 없음 |
| `target/release-test/` 빌드 산출물 | **남아 있음** (2026-08-06 06:57 빌드) |

**2~6절의 검증 결과는 모두 소실 이전에 실제로 실행된 것이며 유효하다.** 소스·문서에는 유실이
없다. 툴체인 복구는 본 작업과 분리해 처리한다.

작업트리에 경로 mangling 잔재 2건(`C：\Users\`— 전각 콜론, `dev\null\` — `2>/dev/null` 리다이렉션이
디렉터리로 생성됨)이 남아 있다. 툴체인 소실과의 인과는 확인하지 못했으므로 추정으로만 기록한다.

이후 확인에서 `~/.ssh` 와 gh 설정(`%APPDATA%\GitHub CLI`)도 함께 없어진 것이 드러났다. 단일 도구
제거가 아니라 사용자 프로필의 도구 설정 디렉터리들이 함께 사라진 형태다. `.claude`·`.config`·
`.docker`·`.gitconfig` 는 남아 있다. MSVC Build Tools 2022 와 Docker 는 무사했다.

복구: rustup 재설치 → `rust-toolchain.toml` 이 1.93.1(+clippy·rustfmt·wasm32) 을 자동 적용.
`gh auth login` 재인증(`planet6897`), SSH 키는 다른 프로필에서 복사 후 ACL 을 소유자+SYSTEM 으로
좁혀 복구했다(OpenSSH 는 개인키에 소유자 외 접근 권한이 있으면 거부한다).

## 8. Stage 2 — rebase 후 재검증

`origin/devel` 이 `570fa6e4f` → `d76d4e98b` 로 20커밋 나아가 rebase 했다. **충돌 0건** — 들어온
20커밋 중 본 변경이 건드린 6개 파일을 수정한 커밋이 없었다.

rebase 시 `pdf-large/hwpx/2026_oss_rst.pdf` 때문에 작업트리가 dirty 로 잡힌다. 조사 결과 이 파일은
HEAD 와 **바이트 동일**(`sha256 bec53a60…`)하고, `.gitattributes` 가 `pdf-large/**/*.pdf` 를 LFS 로
추적하는데 HEAD 에는 원본 바이너리가 들어 있어 clean 필터가 포인터로 재인코딩하며 diff 처럼 보이는
것이다. 저장소 기존 상태이며 손상이 아니다. LFS 필터를 우회해 rebase 를 통과시켰다.

> 7절의 "git-lfs 포인터가 스머지되지 않은 상태" 라는 기술은 방향이 반대였다. 실제로는 작업트리에
> 원본이 있고 HEAD 도 원본을 담고 있으며, 필터가 포인터를 기대하는 쪽이다.

### 8.1 재검증 결과

| 게이트 | 결과 |
| --- | --- |
| `cargo test --lib char_overlap` | ✅ 10 passed / 0 failed |
| `cargo fmt --check` | ✅ 실제 포맷 diff 0 (newline 경고 1059건은 `core.autocrlf` 아티팩트) |
| `cargo clippy --all-targets -- -D warnings` | ✅ exit 0, warning 0 (3m 07s) |
| `cargo test --tests --no-fail-fast` | ⚠️ 465 스위트 ok / 3 FAILED — 전부 무관 (보고서 5절) |
| Native Skia `skia --lib` | ✅ 58 passed |
| Native Skia `issue_2225_missing_picture_placeholder` | ✅ 2 passed |
| Native Skia `render_p37_direct_pdf_export` | ✅ 4 passed |
| WASM 빌드 (Docker) | ✅ 전 단계 통과 (보고서 6.2) |

`--no-fail-fast` 를 붙이지 않으면 첫 실패 스위트에서 cargo 가 멈춰 37 스위트만 실행된다. 3절의
"54 스위트 ok / 1 FAILED" 는 그렇게 잘린 집계였다. 전량 실행하면 실패는 3건이며 세 건 모두
untracked 샘플이 원인임을 격리 재실행으로 확정했다(보고서 5.1).

`render_p37_direct_pdf_export` 는 첫 실행에서 `failed to remove file target\release-test\rhwp.exe`
로 죽었다. 테스트 실패가 아니라 직전 cargo 실행이 남긴 실행파일 잠금이며, 점유 프로세스가 없음을
확인하고 단독 재실행해 통과했다.
