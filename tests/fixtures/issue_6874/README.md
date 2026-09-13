# issue_6874 표·캡션 변환 검증 입력

원본은 기존 committed `samples/issue6266/seizure_list_form_button.hwp`다. HWP3 `objtype=3`의 1×1 표를 버튼으로 오분류하던 경로의 실제 roundtrip 검증에 사용한다.

- `seizure-list-hancom-2020.hwpx`: 원본을 한컴 MCP engine 2020으로 독립 변환한 기준 구조. job `22fba6ba-6c39-4ed6-a271-558804a12168`, 실제 한컴 버전 `12.0.0.4605`(한컴오피스 2022).
- `seizure-list-candidate.hwp`, `seizure-list-candidate.hwpx`: 통합 code `568b210d9`의 HWP5/HWPX 변환 결과. 기준 정답지가 아닌 검사 대상이다.
- 두 HWPX 모두 표 2개·버튼 0개이고 HWP5의 표 control도 2개다. 두 HWPX의 `hp:t` 하위 텍스트와 tail을 함께 추출하고 공백을 정규화하면 `-581-13-` 본문이 모두 보존된다. 한컴의 `hp:fwSpace`를 후보는 가시 공백으로 내보내므로 문서 전체의 원시 문자열·공백 동등성을 주장하지 않는다.
- 시각 기준은 기존 `pdf/pr_planet6897_open_ci_20260828/by_saved_version/pr6281_issue6266_seizure_list_form_button-2020.pdf`이며 새 기준값으로 덮어쓰지 않았다.

| 파일 | bytes | SHA-256 |
| --- | ---: | --- |
| `seizure-list-candidate.hwp` | 6144 | `7032583b0fe09510d7d1e0ae6bfbf6cbad04d153199a7507fe415af5c41475de` |
| `seizure-list-candidate.hwpx` | 11103 | `69311f338d115cddbdca89faf497f1d0284ce9d97a566c2f3a0031acbd9e4c51` |
| `seizure-list-hancom-2020.hwpx` | 33269 | `28e95b27718f727c97ac0aca20abea5a253d4429cd92f78146205a25d74d43f9` |
