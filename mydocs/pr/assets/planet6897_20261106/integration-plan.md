# planet6897 열린 PR 27건 통합 검토 계획

## 현재 단계와 권한

사용자 요청은 원 PR별 reviewer 지정 → 누적 체리픽 → 필요시 메인터너 보정 → 실제 검증 → 개별 최종 리뷰 작성이다.
27건 모두 Open·Non-Draft·base devel이며 `jangster77` 배정을 API users로 재확인했다.
현재 브랜치: `review/integration-20261106-planet6897-open`.
고정 base: `08963f6825956984d38b0d9392f44777db7107ab`. 기존 작업 트리는 clean이었다.
완료되지 않은 검증은 통과나 최종 수용 판정으로 쓰지 않는다. 원 기여자 fork head는 수정하지 않는다.

## 적용 순서와 독립성

원 PR 커밋 표기는 총 64개, 고유 SHA 60개다. 같은 SHA 4개는 한 번만 적용한다.
PR #8654의 bridge 두 개는 각 parent의 git merge-tree와 실제 tree가 같아 새 코드 변경이 없는 순수 bridge다.
PR #8662의 첫 두 patch는 PR #8638의 두 patch와 stable patch-id가 각각 같아 중복 적용하지 않는다.
따라서 일반 커밋 56개를 아래 PR 순서로 `-x --allow-empty` 체리픽한다. source→적용 SHA는 실제 결과로 채운다.
원본의 CI 재실행용 빈 커밋과 실제 source 변경을 구분한다. 충돌 시 양쪽 계약·호출 경로를 읽고 보정한다.

| 원 PR | 주장 | 고정 원 head | 원 PR 커밋 수 |
| --- | --- | --- | ---: |
| [PR #8550](https://github.com/edwardkim/rhwp/pull/8550) | fix: 글자 없는 host 의 나란한 양수 오프셋 자리차지 표를 자기 오프셋에 배치 (#8516) | `48b1c5c8fed6c8a6dcb7ff0a3a1c28648558d8eb` | 1 |
| [PR #8552](https://github.com/edwardkim/rhwp/pull/8552) | feat(edit): 글자 서식 props 에 글꼴 이름 fontName 을 받는다 | `423b369e9d81cbfd6f31ec527d53cbea5d43d5e1` | 1 |
| [PR #8559](https://github.com/edwardkim/rhwp/pull/8559) | fix(layout): 쪽·용지 기준 자리차지 표 띠에 밀린 빈 host 줄은 줄높이와 줄간격을 모두 전진 (#8554) | `efa20ac2eafe1137b370d270f459eac54109b5ca` | 2 |
| [PR #8563](https://github.com/edwardkim/rhwp/pull/8563) | fix(typeset): 빈 host 자리차지 표 뒤 다음 문단이 빈 자리차지 표 host 여도 저장 host 줄 프레임을 받는다 (#8560) | `bc8a04e056ccfbbe371b21239ecfd13696cea0b2` | 2 |
| [PR #8581](https://github.com/edwardkim/rhwp/pull/8581) | fix(renderer): 줄 머리 TAC 표 뒤 문단 끝 공백에 줄을 주지 않는다 (#8574) | `f25efcf2c9dba5793f78a5e5d9c845944c562dfb` | 1 |
| [PR #8589](https://github.com/edwardkim/rhwp/pull/8589) | fix(layout): 어울림 표가 뒤 문단을 띠로 흡수하면 흐름상 다음 문단으로 host 뒤 사다리 일치를 판정 (#8567) | `a68704a019cf56f4dca8db0ee5ff8ae5103febb8` | 2 |
| [PR #8600](https://github.com/edwardkim/rhwp/pull/8600) | fix(renderer): 도형 그림 채우기 크기와 한컴 윤고딕 250 연결 (#8575) | `983933bc448e5ed4f7c8d15be97d894e129a5b9b` | 1 |
| [PR #8604](https://github.com/edwardkim/rhwp/pull/8604) | fix(typeset): #8596 앞 TAC 표 host 줄 안에 저장된 자리차지 표 프레임은 다음 쪽에서 시작한다 | `013459bff4ff5313af53ece5646e79c48647a3aa` | 3 |
| [PR #8605](https://github.com/edwardkim/rhwp/pull/8605) | fix(render): WASM Canvas 합성 굵기를 실제 face 윤곽의 fill+stroke로 그린다 (#8430) | `5d37286267136c9978d7ac2f804913730ab6becf` | 3 |
| [PR #8607](https://github.com/edwardkim/rhwp/pull/8607) | fix(typeset): #8606 빈 host 의 자기 오프셋 자리차지 본표는 머리 표에 밀린 표가 아니다 | `b7b93df883ef0eed535d1e80b5c45ea6d60ce3a9` | 3 |
| [PR #8611](https://github.com/edwardkim/rhwp/pull/8611) | fix(renderer): Native 경로 글꼴을 매핑으로 공유해 64MiB 넘는 파일도 싣기 (#8597) | `0cbe6a9ae5ee4ef8d994d21c170682f95f4bf236` | 2 |
| [PR #8615](https://github.com/edwardkim/rhwp/pull/8615) | fix(layout): 칸 안 빈 host 의 글 위아래 표 띠가 뒤 문단 앞 간격을 흡수한다 (#8610) | `7d1d9b51f2f2cc9f92aef46eefe101a6d4b433a5` | 1 |
| [PR #8617](https://github.com/edwardkim/rhwp/pull/8617) | fix(visual-sweep): 글꼴 catalog family 이름의 앞뒤 공백을 Native 와 같이 정규화 (#8612) | `0fd746a87d28cd70fe7297149882cacd84074475` | 1 |
| [PR #8621](https://github.com/edwardkim/rhwp/pull/8621) | fix(renderer): 각주 안에서 줄을 통째로 차지하는 캡션 달린 TAC 표를 소유 줄에 그린다 (#8551) | `7057aaccfaa0a5bd9bc80304d4ccdd07f220beff` | 3 |
| [PR #8623](https://github.com/edwardkim/rhwp/pull/8623) | fix(native-skia): 최소 글꼴 환경의 native-skia 회귀 4건 — 완결 SVG 문서 예산·저장소 글꼴 공급 (#8613) | `169b5cd2d0bb7d0a25c755477e60dad9c96228e7` | 1 |
| [PR #8625](https://github.com/edwardkim/rhwp/pull/8625) | fix(typeset): #8620 글자처럼 취급 표 host 의 컨트롤 전용 꼬리 줄은 쪽을 넘으면 다음 쪽에 놓는다 | `39ceb92ace1a6ad2ca9578c5b72b6940bb096281` | 2 |
| [PR #8638](https://github.com/edwardkim/rhwp/pull/8638) | fix(renderer): 돋움 ․(U+2024)은 Haansoft Dotum 이 측정·그리기 공급 (#8619) | `71bd8abf48dbad4afa09d66803f6011546add734` | 2 |
| [PR #8642](https://github.com/edwardkim/rhwp/pull/8642) | fix(typeset): 앞 조각만 들어가는 각주는 앞 조각만 비용으로 잡고, 블록 표 각주 조각은 paint 측정으로 잰다 (#8468 후속) | `7d40345bc014f3cb4eca4235783afcfb670899f9` | 6 |
| [PR #8644](https://github.com/edwardkim/rhwp/pull/8644) | fix(composer): 다음 문단 저장 vpos 가 간격 전량을 닫으면 TAC 표 줄 뒤 간격을 전량 남긴다 (#8544) | `1ea144e29c04613c5e7170fc74e622bb03fbd9a0` | 2 |
| [PR #8645](https://github.com/edwardkim/rhwp/pull/8645) | fix(layout): TAC 표 앞 저장 줄 — 쪽 기준 상자에 밀려 누락되고 그리기가 줄 밖 개체 높이로 되감던 것 (#8641) | `2aedb2a8580da9bb1b2d97643d3a97defa4f1660` | 2 |
| [PR #8649](https://github.com/edwardkim/rhwp/pull/8649) | fix(layout): 칸 안의 쪽 기준 글 앞으로 표를 칸 흐름에 쌓지 않는다 (#8648) | `c546460f55cd9cf94d8106cde271750e0a131c0c` | 2 |
| [PR #8650](https://github.com/edwardkim/rhwp/pull/8650) | fix(layout): 한 줄의 TAC 표 여러 개는 선언 폭만큼 전진한다 (#8647) | `068250c0e8bc9b07b8a25911ab8002d5504f63eb` | 1 |
| [PR #8654](https://github.com/edwardkim/rhwp/pull/8654) | fix(layout): 같은 줄 TAC 표 공유 기준선과 rowspan 초과분 — #8645·#8649 위에 쌓음 (#8641) | `d971025c2bb85ebcd2453578800e6130445b81b8` | 8 |
| [PR #8655](https://github.com/edwardkim/rhwp/pull/8655) | fix(layout): TAC 표만 담은 문단의 동반 표를 줄 소유자 기준선에 앉힌다 (#8653) | `cd4cc992f30fdc50478217d3b954c9fd9e1d34e7` | 2 |
| [PR #8662](https://github.com/edwardkim/rhwp/pull/8662) | fix: 저장 줄 없는 HWP5 TAC 표 행 높이·넘친 공백 줄 끝·변경 추적 0x08 저장 줄 유지 (#8639) | `f84e0f3ffe3b154d286bc878fc90facdb7ced153` | 5 |
| [PR #8663](https://github.com/edwardkim/rhwp/pull/8663) | fix: 되감긴 표 제목 줄과 각주 번호 옆 표의 내용 소실 (#8652, #8657) | `a0f7203d788415f378f3ea61d3679a9107983666` | 3 |
| [PR #8681](https://github.com/edwardkim/rhwp/pull/8681) | fix(layout): 자기 자리차지 표 띠 위끝에 걸친 저장 줄을 띠 끝으로 내린다 (#8679) | `3cc76d84d41b40f47304b62ff2b8b5010828b86b` | 2 |

## 구현 전 분석과 검증 순서

1. 원 PR·관련 이슈·기존 해결 PR과 통합 보정의 이력을 대조한다. 원본 저장값과 독립 한컴 Print의 기대값을 구분한다.
2. 공통 값의 생산 → 측정 → 실제 배치 → 이후 덮어쓰기를 추적한다. 표/각주 조각의 컷·소유·요구/예약/실제 높이·종료와 후속 본문을 함께 확인한다.
3. 원 PR에 첨부된 HWP/HWPX/PDF와 기존 저장소 입력을 PR별 output 디렉터리에 미리 준비했다. 현재 196개 실제 파일이며, 약칭·코드 예시·실제 미확보 경로는 별도로 분류한다.
4. 누적 소스를 먼저 컴파일하고 최소 공개 글꼴 환경의 전체 Rust 회귀를 먼저 확인한다. 실패마다 현재 구현 회귀인지 과거 검사 오류인지 독립 출력으로 판정한다. 올바른 기대값을 완화하지 않는다.
5. 빌드 job 8개, 공유 `target/pr-review`, test thread 12개, Cargo는 순차 실행한다. 파생 suite는 별도 검증 checkout에서 prepare하고 제출 diff에 포함하지 않는다.
6. Native/fresh WASM 각 전쪽 TSV·쪽수·변경 영역 확대 판독을 수행한다. 시각 검증은 전체 시스템 글꼴과 승인된 대체 환경, Rust 회귀는 최소 글꼴 환경으로 구분한다. 글꼴 프로그램을 새로 배포하지 않는다.
7. 새 base와 후보는 같은 입력·독립 Print PDF·DPI·글꼴·backend로 비교한다. 사용자 지시의 0.1%p 초과 하락을 허용하지 않는다. 보호 PR #8389·#8397의 TSV는 소수 셋째 자리 반올림 표시값 하락 0을 확인한다.
8. 조판 변경의 공식 Native 편람 HWPX 383쪽 gate, frontend·실제 공개 API·Canvas2D/CanvasKit 등 최종 소비 경로, 필수 fmt/Clippy/타입/정책 검사까지 마친다.
9. 모든 관련 페이지 Native/fresh WASM 90% 이상 후 회귀를 추가/변경한다. 관계 검사를 우선하며 양쪽 98% 이상·독립 확대 판독 조건을 충족한 완료 페이지에서만 적격 좌표 핀을 사용한다.
10. 각 원 PR의 최종 리뷰에 같은 주장 입력·페이지의 PNG(review와 overlay)·전체 TSV와 source/build/input/PDF SHA를 연결한다. 다른 PR 대표 이미지로 대신하지 않는다.

## 원 PR 기록의 알려진 제한

PR #8611은 interview WASM 측정 불가를 PR #8617이 다루며, 최소 글꼴 Native Skia 기존 실패는 PR #8623이 다룬다.
PR #8642는 각주 실문서 일부 기존 미달을 별도 이슈로 적었다. 원 PR의 높은 대표 점수나 기존 이슈 등록만으로 현재 수용 완료를 판정하지 않는다.
PR #8662는 폰트 없는 2022와 원본 서울체가 있는 2020의 출력 조건을 구분한다. 현재 실제 공급 글꼴과 같은 독립 기준을 먼저 선택한다.
PR #8605의 기본 환경 글꼴 예외를 실제 전체 글꼴 검증의 통과로 재표기하지 않는다.
이 문서는 계획이다. 실제 결과와 최종 판정은 개별 리뷰·후속 단계 기록에 갱신한다.

## 단계 1 실제 결과

27개 원 PR의 고유 선형 커밋 56개를 모두 적용했다. -x 원 SHA footer는 정확히 56개이며 중복이 없다. 동일 SHA/동일 patch와 자동 병합 tree가 같은 bridge는 위 계획대로 한 번만 반영했다.

코드 충돌 3곳과 IR 원장 삽입 충돌 1곳을 양쪽 계약을 보존해 해소했다. 아래는 코드 분석 결과이며 회귀·시각 PASS는 아직 아니다.


## PR #8621 — 각주 번호 후처리와 소유 줄 표 배치

- 원 source `afab535863b77d59b1f8686d1ca8a85852dae2ec`, 파일 `src/renderer/layout/picture_footnote.rs`.
- 이전 base의 각주 번호 보정은 PR #8659의 `cf35e7a3a4`에서 ScriptMetricsContract를 소비하도록 보정돼 있다.
- 현재 각주 글줄 발행 뒤 `shrink_footnote_number` 호출과 새 `layout_footnote_line_owned_tac_tables` 호출이 같은 삽입 위치에서 충돌했다.
- 번호 후처리를 먼저 유지한 뒤 같은 first_new_child 구간의 소유 줄 상단으로 표를 그리게 했다. 번호 후처리는 번호 run의 글자 크기·기준선·폭과 뒤 run의 x를 바꾸며, TextLine bbox.y를 바꾸지 않는다. 새 표 원점에 쓰는 글줄 y 계약은 그대로다.
- 각주 번호·캡션 표의 소유 줄·후속 내용 검사를 함께 실행할 예정이다. 이 단계에서는 코드 충돌 해소만 완료했으며 동작 PASS를 주장하지 않는다.

## PR #8654 — 새 원본 두 개의 IR 원시 필드 원장

`tests/fixtures/ir_field_sweep_baseline.tsv`의 같은 삽입 위치에 PR #8563의 안건표지와 PR #8654의 근로기준 보고서 레코드가 각각 추가됐다. 서로 다른 원본의 raw_ctrl_extra 길이 기록이므로 네 행을 모두 보존했다. 기존 행·수치·허용치를 바꾸거나 원장을 재생성하지 않았다. 구조 roundtrip 검증과 새 원본 시각 검증은 이후 전체 검증에서 실제 확인한다.

## PR #8655 — PR #8654와 같은 TAC 공유 기준선 규칙 통합

PR #8654의 로컬 closure와 PR #8655의 tac_line_owner_frame + companion_table_top이 같은 두 배치 지점에서 충돌했다. 두 원안 모두 저장 줄의 다른 표가 바깥 상자를 소유하면 줄 위 + owner 위 여백 + 0.85×높이차를 사용한다. 새 공통 helper를 소비하도록 두 지점을 통합하고, 기존 같은 control_line_seg_index 조건도 helper에 보존했다. 원래 소유 판정은 단일 저장 줄에서만 성립하며 control_line_seg_index는 이때 0이므로 두 원안의 적용 입력은 같다. 중간 표가 폭 때문에 새 줄로 넘어갔을 때 이전 줄 소유자를 쓰지 않는 PR #8655의 반례 조건도 유지했다. 로컬 closure를 중복 잔존시키지 않았다. 공유 기준선·rowspan과 동반 표 검사 양쪽을 이후 실제 검증한다.

## PR #8663 — 각주 표 helper 추가 위치 충돌

PR #8621의 공통 표 측정·줄 가로 원점·소유 줄 표 선택 helper 세 개와 PR #8663의 여백 0 각주 표 소유 판정 helper가 같은 메서드 사이에 추가돼 충돌했다. 네 helper를 모두 보존했으며 기존에 공통화한 측정/가로 원점 경로를 되돌리지 않았다. 번호 옆 표와 캡션 표는 별도 실제 원본·독립 Print·회귀로 검증한다.

추가 코드 검토 항목: PR #8663 number_right는 run의 폭을 문자 수 비율로 나눠 번호 뒤의 너비를 추정한다. 실제 글자 메트릭과 장평·자간에서 같은 근거인지 확인하고, 필요하면 최종 글리프/글줄 결과를 소비하도록 메인터너 보정한다. 현재 이를 실제 검출 회귀나 완료로 판정하지 않는다.

## fixture 목록 정정

최초 203개 후보 중 본문의 glob 표기 7개에 Git show가 빈 출력/exit 0을 반환해 파일로 잘못 계수한 것을 헤더·실제 blob 확인에서 검출했다. 이 7개 빈 임시 파일을 제거하고 실제 파일 196개로 정정했다. 원본/기준 PDF나 source commit을 바꾸지 않았으며 측정에 사용하지 않았다. 실제 glob 항목은 기준 파일로 취급하지 않는다.

## 단계 2 — 최초 전체 Rust 회귀 실제 결과

통합 source `46aca162bcf7fbcbc114cc42a251e6047598b371`에서 최소 공개 글꼴 5개만 노출해 전체 Rust 회귀를 실행했다. 실제 inventory 12,841건 중 기존 ignored 45건을 제외한 **12,796건이 모두 통과**했다. 실패 0건, 실행 시간 736.635초, 최종 exit 0을 확인했다. 컴파일·목록 생성은 별도로 17분 13초가 걸렸고 exit 0이다. 검증 이후의 `947232068fd8182cfd52417ae40986cf41f158ab`는 문서만 변경한다.

기본 feature에서 실행되지 않는 Native Skia 전용 검사는 별도 실행 중이다. 전쪽 Native/fresh WASM TSV·독립 PDF 쪽수·직접 확대 판독 및 lint는 아직 완료하지 않았으므로 개별 PR의 머지 보류 판정은 유지한다. 2026-11-06 원격 fetch 후 base `08963f6825956984d38b0d9392f44777db7107ab`가 바뀌지 않아 리베이스하지 않았다.
