# #6936 — 일반 face만 있는 글꼴의 PDF 굵게 복원

Issue: [#6936](https://github.com/edwardkim/rhwp/issues/6936)

- 확인일: 2026-09-13 KST
- 기준: `upstream/devel` `70bf40af2a2818e72bd58b4fa66e2d4c06de2b51`
- 작업 브랜치: `fix/6936-pdf-synthetic-bold-20260913`
- 구현·시험·증적 커밋: `01cd447a4`
- 판정: 기본 SVG→PDF 경로에서 문제를 재현했고 로컬 수정 및 범위별 검증을 완료했다.
  PR 전체 게이트와 Windows 네이티브 실행까지 완료했다는 판정은 아니다.

## 재현과 원인

돋움·바탕·굴림처럼 Bold face가 없는 family는 SVG의 `font-weight="bold"`에도 usvg가
Regular face를 선택한다. 기존 svg2pdf는 이 글꼴을 채우기로만 기록하여 굵기가 사라진다.
SVG에 stroke만 붙이면 별도의 fill/stroke 텍스트 show 연산을 내보내 추출·검색 문자가 중복된다.

새굴림은 별도의 기존 오류도 있었다. layout-name 규칙이 `새굴림`을 `함초롬돋움`으로 바꾸어
원래 New Gulim 대신 HCR Dotum/실제 Bold를 사용했다. 이 경우 굵어 보이는 것만으로는
원래 글꼴의 굵게가 정상이라고 판정할 수 없다.

## 수정

1. PDF 전용 SVG 준비 단계에서 **실제 glyph에 선택된 face의 weight**를 확인한다.
   일반 face만 사용하는 굵게 text에 한컴 실측 비율 `stroke-width = font-size × 0.02`를 적용한다.
   원본 문자·face·glyph transform을 보존하며 실제 Bold face와 일반 텍스트는 건드리지 않는다.
2. vendored svg2pdf의 단색 fill+stroke를 PDF `Tr 2`로 결합한다.
   하나의 glyph를 한 번만 기록하며 색·불투명도·선 속성을 보존한다. 역순 paint는 같은 불투명 색일
   때만 결합한다. 서로 다른 역순 색·반투명 역순·gradient/pattern은 기존 paint 순서를 유지한다.
3. `새굴림 → New Gulim`의 layout-name 변경을 append-only change set으로 적용했다.
   기존 rule은 retired 이력과 successor를 남기고 새 rule을 추가했다. 기존 New Gulim metrics를
   사용하며 나머지 네 projection의 semantic hash는 동일하다. 봉인 v1/과거 기준선은 갱신하지 않았다.

rhwp가 내보내는 균일한 text cluster가 적용 대상이다. 임의의 혼합 굵기·실제 Bold와 Regular가
섞인 fallback·복잡한 paint·자식 stroke override는 잘못 합성하지 않고 unsupported로 보고한다.
PDF 합성 굵게를 브라우저용 SVG에 삽입하지 않는다. 원래 이슈의 부수적인 HML Bold 파싱 의문은
이번 HWP/HWPX 및 PDF 경로 검증으로 해결했다고 주장하지 않는다.

## 독립 기준과 visual sweep

새 합성 HWPX를 한컴으로 HWP 저장한 뒤 같은 HWP를 engine 2020으로 PDF 변환했다.
한컴 버전은 **12.0.0.4605**, 기준 PDF는 1페이지다. MCP `start → status → download`를 사용했다.
검증 입력·변환 job ID·SHA·로컬 Windows 글꼴 출처는 [fixture 설명](../../samples/issue6936/README.md)에 있다.
HWP/HWPX/기준 PDF와 세 대조 PDF 모두 Git에 포함했다. 기존 파일을 이름만 바꾸어 중복 추가하지 않았다.

[한컴/수정 전/수정 후 비교](../pr/assets/issue_6936/comparison.png)에서 12개 행 전체를 확인했다.

- 돋움·바탕·굴림의 보통과 굵게가 수정 전에는 같은 두께다. 수정 후 한컴 기준처럼 구분된다.
- 새굴림은 함초롬 치환이 없어지고 한컴 기준의 New Gulim 모양과 굵게를 사용한다.
- 함초롬돋움·함초롬바탕의 실제 Bold와 모든 일반 행에 불필요한 합성 획이 추가되지 않는다.
- 새굴림 두 행을 제외한 모든 글자 원점은 수정 전후 동일하다. 새굴림 두 행은 원래 face의
  advance를 사용하며 최대 원점 차이는 5.200012pt다. 이를 합성 획 자체의 위치 변경으로 숨기지 않는다.
- 전체 페이지 픽셀 동일 판정은 하지 않았다. 최종 비교 패널을 Git에 보존하고 전체 PNG는 임시 경로에서 재생성할 수 있다.

| 결과 | 공백 제외 추출 문자 | text show 연산 | 판정 |
| --- | ---: | ---: | --- |
| 한컴 기준 | 158 | 60 | 문자를 묶어 기록, 일반 face 굵게에 Tr 2 사용 |
| 수정 전 devel | 158 | 158 | Bold face 없는 돋움·바탕·굴림의 굵기 소실 |
| stroke만 추가한 대조군 | 207 | 207 | 49자 중복, 겉모양만으로 발견할 수 없음 |
| 최종 수정본 | 158 | 158 | 49개 합성 굵게 Tr 2, 추출 텍스트가 한컴 기준과 동일 |

[PDF 측정 기록](../pr/assets/issue_6936/README.md),
[재검증 스크립트](../../samples/issue6936/inspect_pdf.py),
[한컴 PDF](../../pdf/issue6936-bold-faces-2020.pdf), [최종 PDF](../../pdf/issue6936-after.pdf).

## 실행한 검증

전용 target: `/Users/tsjang/rhwp/target/issue6936-20260913`.
Mac에서 `win10-ted`에 설치된 해당 글꼴 파일을 읽기 전용으로 복사해 `--font-path`로 제공했다.
글꼴 파일은 배포하지 않고 해시와 원래 설치 경로만 기록했다.

| 검사 | 결과 |
| --- | --- |
| 기본 CLI build / 같은 HWP의 3개 코드 상태 export-pdf | 모두 exit 0 |
| #6936 focused nextest | 9/9 PASS |
| #5874 기울임 회귀 | 7/7 PASS |
| #3772 bold ExtraLight fallback 회귀 | 2/2 PASS |
| #7077 gradient/opacity mask 회귀 | 2/2 PASS |
| font rule v2 / projection / mutation rehearsal | 45/45 PASS |
| font projection generator check | PASS |
| native `cargo clippy --locked --target-dir … -- -D warnings` | PASS |
| `cargo fmt --all -- --check` | PASS |
| suite prepare 후 `rust-test-suite-manifest.mjs --check` | PASS |
| PDF 추출/Tr/단일 show/행별 원점/1페이지 검사 | PASS |

원본 실행 로그는 `/private/tmp/rhwp-6936-pr-prepare-20260913/archived-evidence/validation.log`에 보관한다.
파생 integration suite/manifest는 검증용이며 커밋하지 않았다.

`font_rule_projection_baseline.test.mjs`는 7개 중 1개가 과거 W7 snapshot 불일치로 실패한다.
수정 전 primary checkout `70bf40af2`에서도 동일한 검사와 오류를 재현했다
(원본 로그: `/private/tmp/rhwp-6936-pr-prepare-20260913/archived-evidence/baseline-existing-failure.log`).
관측된 차이는 기존 Studio webfont 공급/요청 목록이며 이번 변경의 다른 네 projection은 동일하다.
이 실패를 통과로 기록하거나 과거 snapshot을 덮어쓰지 않았다.

WASM Clippy, workspace build/all-target Clippy, full Rust/Native Skia/교차 호스트 검증 및 원격 CI는
이번 로컬 개선 단계에서 실행하지 않았다. PR 준비 시 변경 범위에 해당하는 필수 게이트를 완료해야 한다.
