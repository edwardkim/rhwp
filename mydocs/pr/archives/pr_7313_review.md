# PR #7313 리뷰 — HWP3 변환 HFT 한글 face ASCII 반각 전진

## 접수 정보

| 항목 | 값 |
| --- | --- |
| PR | [#7313](https://github.com/edwardkim/rhwp/pull/7313) |
| 작성자 | `planet6897` 외부 contributor |
| 관련 이슈 | [#7051](https://github.com/edwardkim/rhwp/issues/7051) |
| 원 contributor head | `60978595346facd3f46361fc8c22d4d6c43eab26` |
| 메인터너 보정 head | `7c51a07d36f0edc98fde27737c7da09e2595927d` |
| 최신 code candidate CI | [CI #35584051980](https://github.com/edwardkim/rhwp/actions/runs/35584051980) 성공 |
| reviewer | `jangster77` 지정 |

원 변경은 HWP3에서 변환된 HWP5 문서의 HFT 한글 전용 face를 TTF로 치환할 때 ASCII를
치환 글꼴의 비례폭으로 측정해 저장 LineSeg 폭을 넘던 문제를 고친다. 메인터너 보정은 oracle PDF를
새로 추가하지 않고, 기존 추적 PDF의 원본·분할 provenance와 489쪽의 분할본 쪽 매핑을 주석에 고정한다.

## 구현과 검증 입력

- `FontSubstitutionBoundary::Hft`이고 HWP3 변환본인 경우에만 ASCII graphic의 advance를 `em/2`로
  측정한다. 진짜 영문 HFT 경계와 일반 HWP5의 legacy HFT 이름은 반례로 제외한다.
- 입력 `samples/hwp3-sample10-hwp5.hwp`는 이미 추적된 파일이며 SHA-256은
  `a660a0d41898c8316479392c9687d81a15555431a581bdda988bf3598f6f0d51`이다.
- 기준은 이미 추적된 `pdf/pr7268/hwp3-sample10-hwp5-p301-600-2024.pdf`의 189쪽(원문 489쪽)이다.
  이 파일을 새로 추가·복제하지 않았다. PDF 원본과 분할 provenance는
  [PR #7268 기록](pr_7268_review.md#기준-pdf-보존과-검증-경계)에 있다.

`pdftotext -f 189 -l 189 -bbox`로 확인한 `TABLESPACE(ROLLBACK_DATA),`의 폭은 156.64px이고,
최종 Native render tree의 문제 런 폭은 554.9px로 저장 LineSeg 폭 566.9px 안에 들어간다.

## 완료한 검증

| 검증 | 결과 |
| --- | --- |
| `cargo fmt --all -- --check` | 통과 |
| `regression_suite_027`의 #7051 focused 3개 | 통과 |
| Native / WASM Clippy | 통과 |
| workspace build·test-suite manifest | 통과 |
| Native Visual Sweep, 489쪽 | 구조 flag 0, overflow·line-band·frame 이상 0 |
| fresh WASM Visual Sweep, 489쪽 | 구조 flag 0, overflow·line-band·frame 이상 0 |
| GitHub Full CI | lint, Native Skia, Canvas visual diff, frontend package, archive A–D와 shard, CodeQL, Render Diff, Proptest, Adapter 모두 성공 |

Native와 WASM 모두 문서 489쪽을 한컴 2024 기준 PDF와 대조했다. 전체 문서의 기존 글꼴 raster 잔차를
이번 반각 전진 변경으로 해결했다고 주장하지 않으며, 수용 근거는 문제 런의 폭·저장 줄 소속·우단 초과 해소다.

## Visual Sweep 증적

| backend | review | overlay |
| --- | --- | --- |
| Native | [489쪽 review](../assets/pr7313_review/native_review_489.png) | [489쪽 overlay](../assets/pr7313_review/native_overlay_489.png) |
| fresh WASM | [489쪽 review](../assets/pr7313_review/wasm_review_489.png) | [489쪽 overlay](../assets/pr7313_review/wasm_overlay_489.png) |

## 최종 판정

**메인터너 보정 후 수용 가능.** 원 contributor 구현의 측정 범위와 HWP5 반례 보호는 적절하며,
PDF oracle provenance 보류 사유는 `7c51a07d3`의 주석 보정으로 해소했다. 이 기록만 포함한 trailing
head의 fast-pass CI, `MERGEABLE`/`CLEAN` 재확인과 작업지시자의 merge 지시가 남았다.
