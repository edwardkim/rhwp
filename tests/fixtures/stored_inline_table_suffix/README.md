# 저장 줄 끝의 인라인 표

`two_digits.hwpx`는 원 PR #7165의 합성 입력이며 바이트를 변경하지 않았다. 원시 UTF-16은
표 0..8, `x y ` 8..12, 표 12..20, `h` 20이다. 수동 저장 줄 경계 12/20은 모두 가시 위치
4로 투영되지만 표의 소속 줄은 다르다. 기존 합성 테스트는 이 투영 계약만 검사한다.
부모 셀 폭 28800 HU에서는 한컴이 두 표와 `h`를 같은 줄에 저장하므로, 합성 파일의 두 줄
기대를 실제 날짜 양식 개선 증거로 사용하지 않는다.

독립 입력은 Hancom 12.0.0.4605가 새로 저장한 HWP다. 한컴 저장 이후 줄 정보를 수정하지 않았다.

- `two-digits-wrap-input.hwpx`: 기존 입력의 부모 표·셀 폭을 세 숫자칸 폭인 6480 HU로
  줄이고 수동 linesegarray를 모두 제거했다. 자식 표 폭은 각각 2160 HU다. 한컴 저장본
  `two-digits-wrap-2020.hwp`는 0/20 경계를 기록한다. 두 표는 첫 줄, `h`는 다음 줄이다.
- `two-digits-wide-2020.hwp`: 기존 `two_digits.hwpx`를 한컴에서 재저장했다. 한 줄이며
  두 표와 `h`가 함께 놓인다. 단순 이름 변경본이 아니라 한컴이 저장 줄을 다시 작성한 HWP다.
- `two-digits-break-input.hwpx`: 기존 넓은 입력의 `x y ` 뒤에 명시적 lineBreak를 넣고
  수동 linesegarray를 모두 제거했다. 한컴 저장본 `two-digits-break-2020.hwp`는 0/13 경계를
  기록하며 두 번째 표와 `h`가 둘째 줄에 놓인다.

- `two-digits-table-wrap-input.hwpx`: 좁은 입력의 부모 폭을 두 숫자칸 폭인 4320 HU로
  더 줄였다. 한컴 저장본 `two-digits-table-wrap-2020.hwp`는 0/12 경계를 기록한다. 폭 부족으로
  두 번째 표와 `h`가 둘째 줄에 놓인다. [대응 PDF](../../../pdf/two-digits-table-wrap-2020.pdf)도 포함했다.

메인터너는 첫 저장 줄 끝의 명시적 개행 뒤에 다음 인라인 표 줄이 이미 있을 때 빈 줄을
중복 생성하지 않도록 composer를 보정했다. 기존 #6300 경계 보호를 첫 줄에도 적용한다.

테스트는 표가 각각 한 번 그려지는지, 셀 안에서 잘리지 않는지, 네 경우의 표·접미 글자 줄
소속이 독립 한컴 저장 결과와 같은지 검사한다. 원 PR의 비공개 날짜 양식은 확보하지 못했다.

## 변환 출처와 파일 해시

모두 `hwp2024-mcp-convert start → status → download`, engine 2020,
32bit direct DLL host, input_preprocess=none, font scope verified로 완료했다.
PDF는 one-up이다. 인증 정보는 증거에 포함하지 않는다.

| 입력 | 한컴 저장 결과 | job ID |
| --- | --- | --- |
| `two-digits-wrap-input.hwpx` | `two-digits-wrap-2020.hwp` | `3a302933-cae1-463a-b8de-c65f4d858d6f` |
| `two_digits.hwpx` | `two-digits-wide-2020.hwp` | `f45ddb48-f4b9-4101-b7c7-3cc286b98081` |
| `two-digits-break-input.hwpx` | `two-digits-break-2020.hwp` | `db898f7c-da0e-435e-a093-34b821a0bec3` |

PDF는 [two-digits-wrap-2020.pdf](../../../pdf/two-digits-wrap-2020.pdf),
[two-digits-break-2020.pdf](../../../pdf/two-digits-break-2020.pdf)를 새로 추가했다.
넓은 입력은 기존 [two_digits-2020.pdf](../../../pdf/two_digits-2020.pdf)를 재사용한다.
넓은 한컴 재저장 HWP의 PDF job `8112dafe-aa5f-4ece-88bb-1d76b42e03b7`도 성공했고
96dpi RGB raster가 기존 PDF와 완전히 동일하여 중복 PDF를 추가하지 않았다.
기존 PDF job은 `94add193-e677-4745-84dd-62027219c569`다.

| 파일 | SHA-256 |
| --- | --- |
| `two-digits-break-2020.hwp` | `6f78cdbb13ec414a57c17eb76c0ea0634fbecc6974afb5bfd43dfe8b955979e7` |
| `two-digits-break-input.hwpx` | `ca144f5847f75e53500120bb62b93604d2e580700313d4334dca7fa5ed2e87f4` |
| `two-digits-wide-2020.hwp` | `0d0421b2ce0c8d407f95ecc4f70fa2d7b1243fd913e028c84e2d8ec010b41ae9` |
| `two-digits-wrap-2020.hwp` | `a71d47b0a50bce04702e29ee13b108071480cef47ee30f388c94db68b77b633d` |
| `two-digits-wrap-input.hwpx` | `8c482f17b8b9b1a03003655899a3681fde7bd28063be1b97441a5034280b0575` |
| `two_digits.hwpx` | `8ad7133baab557ba00845a5733b6bd4a042833b426c8cb457d58c8794e57052d` |

변환 결과·입출력 해시는 [hancom-evidence.json](hancom-evidence.json)에 기록했다.
기존 HWPX/PDF는 이름 변경으로 중복 추가하지 않았다. 비공개 실제 문서의 개선은 주장하지 않는다.

추가 폭 부족 입력의 변환 job: `266c0a3f-6bc1-48eb-b4cc-cac824dec6cb`.

| 파일 | SHA-256 |
| --- | --- |
| `two-digits-table-wrap-input.hwpx` | `97091a831d4e784a1ef482bc038d71eaa39b4146e8a7621f08a442e2f3485053` |
| `two-digits-table-wrap-2020.hwp` | `e5d9aa020c9f76690be814a4645c14e21966c0e0b8b4636c2e595c4094646f8b` |

`rhwp info --json`에서 독립 저장 HWP는 `format=hwp5`,
`lastSavedWith.product=hancom-office-2022`, `version=12.0.0.4605`로 확인했다.
파일명의 `2020`은 변환 engine 선택이며 저장 제품 연도가 아니다. 2022 이하 저장본이므로 engine 2020을 사용했다.

## 메인터너 검증 결과

2026-09-15, macOS 전용 `target/pr7155-7167-review-20260915`에서 검증했다.
`stored_inline_table_suffix` 2개(합성 12/20 + 독립 한컴 4경우), #6122 1개,
#6706 1개, #6300 4개 집중 테스트가 통과했다. Native visual sweep에서 네 경우의
표·접미 글자 줄 소속을 PDF와 대조했다. 표 테두리/일부 높이의 기존 차이는 남아 있으며
전체 페이지 일치를 주장하지 않는다.
fmt check, Native/WASM/workspace all-targets Clippy(`-D warnings`), workspace build,
test suite manifest 및 unit-tier 검사가 통과했다. 전체 회귀는 사용자 지시에 따라 중복 실행하지
않았으며, 통합 head의 GitHub CI는 PR 생성 후 별도로 확인한다.

최종 fresh WASM 빌드와 visual sweep도 완료했다. 머리글·표 4경우·#6122 p6, 총 6페이지의
Native/WASM raster는 바이트 동일하다. 대표 증적은 통합 code CI 후 같은 PR의 review trailing
commit에 포함하며, 입력 HWP/HWPX/PDF와 이 결과보고는 보정 코드와 함께 커밋한다.
