# HFT 막대 부분 메트릭의 서식 범위 보정

기록: 2026-11-15T11:44:02.229230+09:00.

## 독립 근거와 원인

- `samples/basic/issue2007_nested_cell_pagination_42065.hwp`16쪽의 일반 en dash는 독립
  Print T6/HFT376의1000/1000em이다. 기존1em 폭·중심 배치를 유지한다.
- `samples/issue6764/1613000-202200037-air-traffic-controller-cbta.hwp`47쪽·문단501의
  굵은 en dash는 독립 PDF F4/CID83(HCI Poppy)의500/1000em이다. 원본 글자모양36의
  크기는1400HU다. 이 공급 경로에 일반1em 부분 표를 상속해 뒤 영문이 약9.3CSSpx 밀렸다.
- 새 base와 후보의 실제 전389쪽씩 비교와 [같은 쪽 직접 판독](rebase-f823-analysis.md)의 반례를
  사용한다.2%p 허용폭이나 기존 전체13,582 PASS로 실제 배치 결함을 면제하지 않는다.

## 공통 생산·소비 경계

`font_metric_legacy_hft::rejects_regular_bar_style`가 일반 en dash의 검증된 서식 범위를
소유한다. `find_metric_decision_for_character`는 명시적 HFT 메트릭에서 확인하지 않은 서식을
배제하고 종전 표시 대체 조회로 내려간다. 새500 폭이나 문서별 좌표를 코드에 고정하지 않는다.
`glyph_fit::canvas_cluster_fit`도 같은 서식 근거를 확인해 일반 전각 칸의 중심을 미확인 굵게에
전달하지 않는다. 일반·별칭·미확인 원본·양수/음수 자간의 기존 계약은 대조군으로 유지한다.

기존 일반 부분 표 자체와ASCII·다른 기호의 폭을 바꾸지 않는다. 실제 Bold 프로그램이 제공되는
다른 face의 조회도 같은 HFT 부분 표로 우회시키지 않는다. 굵게·기울임을 일반 coverage의
인증 근거로 간주하지 않는 변경이며 기울임의 새 독립 출력까지 검증했다고 주장하지 않는다.

## 수정 전과 집중 검사

새 이름의 관계형 반례 두 건을 먼저 추가했고 수정 전에는 두 건만 실패했다. 기존 일반1em·
표시 별칭·중심 배치·미확인 원본의 네 검사는 통과했다. 수정 뒤 같은 여섯 검사가 모두 통과했다.
설치 글꼴·비공개 OTF에 의존하지 않으며 기존MIT 합성 CFF 테스트 자원만 사용한다.
원본 em의 비율·동일 공급 glyph의 상대 원점으로 검사하고 실물 절대 px를 고정하지 않는다.

| 실제 실행 | exit | 결과 | 로그 SHA-256 |
| --- | ---: | --- | --- |
| 수정 전 | 100 | 4 PASS·새 반례2 FAIL | `f040581f92901d2e6aa16cf1b48188b16dbc5db5280ef809eff3d1c21845adce` |
| 수정 후 | 0 | 6 PASS | `ad006e938384236eee9ed399730e27eec8faf5a9aba00ccf62dded24701f570e` |

실제 명령은 최소 공개5개 글꼴 격리의 `cargo nextest run --locked --cargo-profile release-test
--target-dir /home/tsjang/rhwp/target/pr-review --features native-skia --test regression_suite_007
-E 'test(/(^|::)hft_declared_en_dash_advance::/)|test(/(^|::)hft_centered_bar_glyph_fit::/)'
--test-threads 12 --no-fail-fast`다. Cargo 빌드 잡은8개다.

주석의 갱신 시각은 집중 검사 종료 뒤 실제 기록 시각이며 실행한 assertion과 제품 코드는 같다.
이 집중 검사 단계에서는 실제 출력·전쪽 보호·전체 회귀와 필수 lint가 후속 조건이었다. 완료한 최종 결과는 아래 절과 최종 검증 기록을 따른다.
이 집중 검사 통과를 PR 생성·원격 CI·통합 완료로 표시하지 않는다.

## 실제 출력과 최종 검증 완료

제품 source `c95f9ade4e8acf9d901c01136ef2b9bacd60c497`에서 Native 47쪽은 98.61% → 99.18%로 개선됐으며 뒤 영문 원점은 같은 Native base의 x384.8로 복구됐다. WASM은 자체 base와 같은 x394.1·98.54%를 유지한다. backend 사이의 좌표 동일성을 요구하지 않는다. 두 경로의 실제 review·overlay를 직접 판독했다.

최종 CBTA 47쪽: Native [review](final/rebase-f823-reading/en-dash-p047/final-native-review-047.png)·[overlay](final/rebase-f823-reading/en-dash-p047/final-native-overlay-047.png), fresh WASM [review](final/rebase-f823-reading/en-dash-p047/final-wasm-review-047.png)·[overlay](final/rebase-f823-reading/en-dash-p047/final-wasm-overlay-047.png). 이전 보정 전 그림과 구분한다.

선행 진단에서 두 backend 모두 Native base 좌표를 요구했던 오류는 같은 backend끼리 비교하도록 정정했다. WASM의 기존 후보와 새 후보 PNG 차이는 막대 가장자리 1픽셀이며, 공식 raw와 TextRun 원점은 같다. 제품 코드나 TSV·보호 하한·허용폭을 바꿔 진단을 통과시키지 않았다.

일반 중첩 표 17쪽은 Native·fresh WASM 모두 최저 98.59%로 상향된 보호값을 유지했다. 원 PR 문서는 양쪽 전 14쪽 raw 100.00%이며 누락·중복 검사도 통과했다. 전체 보호·최소 글꼴 회귀·Native 3종·필수 lint·Studio 결과는 [최종 검증 기록](verification-record.md)에 연결했다. 원격 CI와 병합 완료는 이 로컬 검증 기록으로 대신하지 않는다.
