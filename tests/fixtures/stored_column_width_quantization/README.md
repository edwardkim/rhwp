# 단 너비 정수화와 공백 재조판

`header.hwpx`는 원 PR #7168의 합성 입력이며 바이트를 변경하지 않았다. 문구는 `Left 10`,
공백 95개, `Right`다. 페이지 폭 43202 HU, 여백 합계 7200 HU, 저장 줄 폭 36002 HU로
한 줄을 수동 기록했지만, 독립 한컴 출력은 단 폭을 36000 HU로 정수화하고 두 줄에 배치한다.
따라서 원래 한 줄 기대는 정상 조판의 기준으로 사용하지 않는다.

한컴 12.0.0.4605가 저장한 `header-hancom-2020.hwp`의 원시 줄 시작은 0/89이며,
16-unit 제어를 제외한 가시 문자 경계는 0/73이다. 첫 줄 폭을 초과한 첫 공백은 앞 줄에
흡수하고 나머지 공백은 다음 줄로 넘긴다. 별도 단어가 없는 trailing 입력도 두 줄이며,
명시적 줄바꿈을 넣은 입력은 0/89/119의 세 줄이다. 이 저장 줄은 rhwp에서 편집하지 않았다.

메인터너 보정은 단 폭 불일치 캐시의 재수용을 제거하고 공통 interval filler에서 공백 폭
초과를 처리한다. 같은 offset의 가시 인라인 개체는 공백처럼 줄 밖으로 흡수하지 않고 함께
이월한다. 테스트는 독립 저장본과 편집 후 재조판, 잔여값 0~3을 비교한다.

`trailing-spaces-input.hwpx`는 기존 입력에서 `Right`를 제거하고 저장 줄 배열을 제거했다.
`spaces-before-break-input.hwpx`는 `Right` 앞에 명시적 lineBreak를 추가하고 저장 줄 배열을
제거했다. 그 외 문구·폭은 원 입력을 유지했다. 저장본은 각각 이 입력을 한컴에서 저장한 결과다.

## 변환 출처와 파일 해시

모두 `hwp2024-mcp-convert start → status → download`, engine 2020,
32bit direct DLL host, input_preprocess=none, font scope verified로 완료했다.
PDF는 one-up이다. 인증 정보는 증거에 포함하지 않는다.

| 입력 | 한컴 저장 결과 | job ID |
| --- | --- | --- |
| `header.hwpx` | `header-hancom-2020.hwp` | `77567cf6-2445-4128-b7c7-c65c30dc5c08` |
| `trailing-spaces-input.hwpx` | `trailing-spaces-2020.hwp` | `0ae5778f-eb8c-4611-b34a-3aa878fdbb0c` |
| `spaces-before-break-input.hwpx` | `spaces-before-break-2020.hwp` | `d0a17f66-f4a6-4d1f-979d-62f891d11e98` |

기준 PDF는 기존 [stored-column-header-2020.pdf](../../../pdf/stored-column-header-2020.pdf)를 재사용한다.
원 입력의 PDF job은 `5ba21e32-ca5c-43c2-88a8-ab8f2e7c1776`이다. 한컴 재저장 HWP의
PDF job `63b0acb8-f9b6-403e-865a-33ba65dd5029`도 성공했고 96dpi RGB raster가 기존 PDF와
완전히 동일하여 중복 PDF를 추가하지 않았다. PDF `Right` bbox 시작은 x=180.121552pt,
yMin=51.7152pt이며 첫 줄 Left/10의 yMin은 35.7552pt다.

| 파일 | SHA-256 |
| --- | --- |
| `header-hancom-2020.hwp` | `d660f4cd8fb90eea07b7ad742a2e121edc159e5e74df73fd5c62b2e5bea7c560` |
| `header.hwpx` | `b0ac06d1333b9c8ea263ec184e782d58d043916b7e6b9a6f39dfdeb31102437b` |
| `spaces-before-break-2020.hwp` | `155d32290affa2b8ce6e441a32d82516d430e9739a950af41303a478984c22f7` |
| `spaces-before-break-input.hwpx` | `fe15787f59591e41794aca84943cae997390e8aeb4fd0688df61e1d4ad23053d` |
| `trailing-spaces-2020.hwp` | `41ce9015722d0730d5e21a64da80250861729f7833cfd7881c6bd605c800fb61` |
| `trailing-spaces-input.hwpx` | `baee8aa27c0eda9ef51a7a1e4e14a52a2ac3753cb0f0044919031aa24f56fc98` |

변환 결과·입출력 해시는 [hancom-evidence.json](hancom-evidence.json)에 기록했다.
기존 HWPX/PDF는 이름 변경으로 중복 추가하지 않았다. 비공개 실제 문서의 개선은 주장하지 않는다.

`rhwp info --json`에서 독립 저장 HWP는 `format=hwp5`,
`lastSavedWith.product=hancom-office-2022`, `version=12.0.0.4605`로 확인했다.
파일명의 `2020`은 변환 engine 선택이며 저장 제품 연도가 아니다. 2022 이하 저장본이므로 engine 2020을 사용했다.

## 메인터너 검증 결과

2026-09-15, macOS 전용 `target/pr7155-7167-review-20260915`에서 검증했다.
`stored_column_width_quantization` 2개(독립 한컴 3종 × 폭 잔여 0~3 ×
저장/편집 무효화의 24조건 포함), 공통 줄 나눔 12개와 frame 9개 테스트가 통과했다.
Native visual sweep에서 원 HWPX의 Right가 PDF와 같은 둘째 줄에 배치됨을 확인했다.
fmt check, Native/WASM/workspace all-targets Clippy(`-D warnings`), workspace build,
test suite manifest 및 unit-tier 검사가 통과했다. 전체 회귀는 사용자 지시에 따라 중복 실행하지
않았으며, 통합 head의 GitHub CI는 PR 생성 후 별도로 확인한다.

최종 fresh WASM 빌드와 visual sweep도 완료했다. 머리글·표 4경우·#6122 p6, 총 6페이지의
Native/WASM raster는 바이트 동일하다. 대표 증적은 통합 code CI 후 같은 PR의 review trailing
commit에 포함하며, 입력 HWP/HWPX/PDF와 이 결과보고는 보정 코드와 함께 커밋한다.

## PR #7171 CI 정책 보정

첫 code candidate `95d631f0a`의 CI lint job은 Clippy 전에 source-side test 총량 검사에서
실패했다. 로컬의 `rust-unit-test-tiers.mjs --check`만으로는 PR base와의 증가를 검사하지
못했고, CI의 `--base-ref` 비교에서 4206 > 4205 및 fill_cursor_tests 3 > 2가 검출됐다.

인라인 개체가 동반된 공백 반례를 기존 공백·탭·개행 cursor 테스트에서 호출하도록 묶었다.
assertion이나 입력은 삭제하지 않고 동일하게 실행하며 source-side 총량은 4205로 유지한다.
기준선·정책을 완화하지 않았다. 이후 사전 검사에는
`node scripts/rust-unit-test-tiers.mjs --check --base-ref upstream/devel`을 사용한다.
생산 코드 변경이 없는 cfg(test) 내부 구성 보정이므로 기존 Native/WASM 시각 결과를 유지한다.

보정 후 base 비교 4205 tests, 기존 cursor 테스트 2개(모든 반례 포함), fmt check,
Native/WASM/workspace all-target Clippy, workspace build와 suite manifest 검사를 모두 통과했다.
