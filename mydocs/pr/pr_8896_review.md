# PR #8896 리뷰 — 표 끝 조각의 불필요한 빈 쪽 제거

## 최종 판정

**메인터너 보정 후 수용 가능 — 필수 로컬 검증 완료.**
작업지시자의 “예외 없이 기존 차이도 먼저 해결” 결정에 따라 과거 보호값 미재현도 보정했다.
최신 기준 Native/fresh 최적화 WASM 전 3,542쪽의 공식 보호값 하락은 0이다.
전체 회귀 13,326건·Native 공식 3종 4,073건·추가 공급 protocol 22건과 모든 lint를 통과했다. 원격 push·승인 리뷰·PR 생성·병합은 아직 수행하지 않았다.

적용 경로는 외부 PR의 `maintainer_general.md`이며, `intake_and_review.md`·
`local_validation.md`·`visual_fixture_evidence.md`·`post_merge.md`를 함께 읽고 적용했다.
작업지시자가 제출 후보 `f6acac4df1`의 원격 처리 전체를 승인했다. 첫 push 전에 이 문서의
판정 제목과 절차 연결만 정리하며, 최종 검사 source 이후 제품 코드·검사는 변경하지 않는다.

| 항목 | 실제 값 |
| --- | --- |
| 원 PR·작성자 | [#8896](https://github.com/edwardkim/rhwp/pull/8896), hyunmyungJaneLee |
| 원 head 보존 | `51a960e392a3d79a4d28f78fa90a2aa88ad8c0d7` |
| 최신 devel | `eb84afc0364c51031bb07cbbb013476293367990` |
| 최종 검사 source | `759212053e4b289db93034c7b2b32f5b50bdcd4e` |
| 실제 Native 생산자 | `1749b66d704b3ea0443ae9792ff8a1d65e26a9c1` |
| 실제 fresh 최적화 WASM 생산자 | `68e8190d334fccc5a2846dcfead4a7c60330774a` — 재사용된 library와 Native 생산자의 제품 source 동일 |
| 관련 완료 대상 | [#8693](https://github.com/edwardkim/rhwp/issues/8693) |
| 별도 잔여 범위 | [#7620](https://github.com/edwardkim/rhwp/issues/7620)은 이 PR로 닫지 않음 |

2026-11-11 메인테이너 측 완료 처리 지시를 받았다. fork `kidsnote/rhwp`의 push 권한이 없어
원 기여 SHA를 ancestor로 보존하는 upstream 수용 PR을 준비한다. fork branch와 기존 다른 worktree는 보존한다.

## 원 주장과 독립 기준

내용과 이어받을 물리 공간이 모두 끝난 마지막 표 조각에서 다음 단·쪽으로 먼저 전진해
빈 쪽을 남긴다. 최신 devel의 공개 세 입력은 `2 / 3 / 3`쪽이며 후보는 독립 Print의
`1 / 2 / 3`쪽이다. 앞 표 뒤 글줄 자리가 없어 실제로 이월된 빈 host 글줄은 세 번째 입력에서 보존한다.
기준은 공개 합성 입력을 실제 웹한글기안기로 연 **Print PDF**다. 데스크톱 한컴 출력이나
기여자의 비공개 코퍼스 보고로 바꿔 주장하지 않는다.

## 실제 소비 경로와 보정

| 단계 | 검토한 실제 경로 |
| --- | --- |
| 입력·점유 | `table/block/prepare.rs:1053` 주변은 같은 문단의 앞 표·첫 줄 점유와 이월 직전 공간을 읽는다. `1442`에서 단·쪽이 실제 바뀐 사실을 `host_line_trails_table`로 전달한다. 빈 문자열을 점유 없음으로 대체하지 않는다. |
| 공통 전달 | `table/continuation.rs:213`의 prepared state를 기존 scan·emit 경로가 소비한다. |
| 종료 전진 | `continuation/fragment/emit.rs:2083`은 마지막 행·완결 컷에서만 전진을 지연한다. 블록 컷·첫 조각 빈 띠·실제 뒤따르는 host 줄은 제외한다. |
| 물리 잔여 | `2196` 이후 일반 칸과 끝 행의 병합 칸 잔여를 계산·결합한다. `2606`에서 잔여가 있으면 전진한다. 내용 완료를 물리 높이 완료로 대체하지 않는다. |
| 후속 소유 | `2625` 이후 내용 컷·빈 띠·이어받기 상태를 기존 계약으로 갱신한다. 표 끝의 불필요한 전진만 제거하며 좌표 clamp·내용 숨김·문서 ID 분기는 추가하지 않는다. |

원 기여의 종료·host 줄 판정은 보존했다. 기존 세 회귀에 모든 행 라벨의 단일 소유,
두 표의 내용 순서·누락·중복 방지를 보완했다.

과거 보호값 미재현은 실제 글꼴 공급의 256개 상한·선택 메타데이터와 출력 경로를 추적했다.
문서에 필요한 원 프로그램과 명시 환경을 공급하고, 두 SEO Native 문서는 원 검증의 전체 공급을 유지했다.
쪽별 최댓값을 취하지 않았다. 공통 제품 보정은 다음 세 가지다.

- 원 PUA 작은 삼각형 U+F02FB를 지원 목록 마지막에 추가해 기존 공급자 탐색 순서를 보존한다.
- 구형 수식의 가족 이름을 독립 준비 도구의 `RHWP Hancom Legacy Equation`과 연결하고 이전 fallback을 유지한다.
  실제 독립 한컴 Print의 `×`·`÷` 프로그램을 검증해 준비 도구에 선택 옵션을 추가했다. 기본 v1은 유지한다.
- Native PNG Print의 WMF 전용 가족도 같은 renderer snapshot에서 공급한다. 기존 공통 요청·등록·예산·paint 갱신을 재사용한다.

제공된 `/home/edward/mygithub/ttfs/windows/HYHWPEQ.TTF`는 Native/WASM의 실제 공급에서 확인했다.
제공된 프로그램의 ×·÷ 전진 831/1024와 독립 구형 Print의 616/1000·623/1000을 구분한다.
제공 파일의 제작 시기를 추정하지 않는다. 글꼴 프로그램은 저장소·배포물에 넣지 않았으며
[공급 원인·독립 프로그램 근거](assets/pr8896/font-supply/README.md)에 hash와 경로를 기록했다.

## 최종 시각·기능 보존 결과

| 범위 | Native | fresh 최적화 WASM |
| --- | --- | --- |
| 원 #8693 세 입력·전 6쪽 | 모든 쪽 raw 100.00000% | 모든 쪽 raw 100.00000% |
| #8945 대조군 다섯 입력·전 8쪽 | 모든 쪽 raw 100.00000% | 모든 쪽 raw 100.00000% |
| 정본 보호 18개·1,771쪽씩 | 최저 97.99083%, 공식 하락 0 | 최저 94.65178%, 공식 하락 0 |
| 같은 공급의 최신 devel 대조 | PNG 1,760쌍 동일, 최악 변화 −0.00112%p | PNG 1,766쌍 동일, 최악 변화 −0.00181%p |
| 기존 base 기능 일곱 입력·전 19쪽 | PNG 19쌍 동일 | PNG 18쌍 동일, 한 쪽 작은 삼각형 복원으로 개선 |
| 새 devel의 #8958 39개 기능 | 44개 조건·주장 88쪽 PNG 모두 동일 | 같은 88쪽 PNG 모두 동일 |
| 필수 편람 HWPX Native gate | 공개 지원 웹폰트·383쪽 PASS | Native 필수 gate와 별도 정본 보호에서 확인 |

PDFium 96 DPI·print·문서별 compat 2022/2024·색상 경계 보정 전 2px
`silhouette_raw_match_percent`다. 공식 두 자리 TSV를 Decimal로 비교했으며 정본·허용치·색상 판정을
바꾸지 않았다. 전체 쪽수·누락도 확인했다. raw 100%를 RGB 전체 동일로 표현하지 않는다.

원 표 끝·마지막 행 글자·아래 괘선과 빈 host 줄을 직접 대조했다. 구형 분수·산술 기호·작은 삼각형·
WMF와 주변 본문도 확대 판독했다. 이전 생산자와 달라진 보호 7쪽씩은 최신 Native/WASM과 독립 Print를
다시 같은 배율로 판독했다. 다른 각 1,764쪽은 이전 PNG와 hash로 연결한다.
#8958의 기존 Mac/비최적화 생산자를 실행 증거로 재사용하지 않고, 이 환경의 실제 base/head 생산자를 각각 준비했다.
88쪽의 기능 보존과 전체 문서 쪽수 동등성을 확인했으며 해당 문서의 미선정 쪽까지 새로 개선했다고 주장하지 않는다.

현재 원본·PDF 154개 파일은 Git blob과 실제 SHA-256을 대조했다. 임시 출력이나 글꼴 bytes를 재현 원본으로 바꾸지 않았다.
전쪽 TSV·실제 생산자·확대 판독·비교 원장은 [최신 통합 증적](assets/pr8896/latest-eb84/README.md)을 따른다.
이전 base `4c876654d8…`의 완료 자료는 같은 폴더 상위에서 **선행 검증**으로 구분한다.

## 회귀 검사와 실행 연결

- 원 종료 경계 3건·원 PUA protocol 19건·실제 구형 가족 1건과 공용 중첩 표 9건을 최소 글꼴 환경에서 통과했다.
- 새 WMF 합성 관계 검사는 수동 등록 대조·미공급 음성 대조·자동 공급·중복 억제·저장 IR/WMF 보존을 확인한다.
  기존 `render_p37_direct_pdf_export`의 Native CI 선택기에 연결해 별도 workflow 호출을 추가하지 않았다.
- 기존 focused-case 22개와 source/root contract 참조도 재점검했다. 유효한 완료 쪽 ±1/2px와 원본 단위 계약은 유지했다.
  공용 중첩 표 두 검사에서 고정 영역 선택을 원본 9×2 표의 소유·폭·하단·조상 clip 관계로 교체했다.
  비종료 2쪽의 종료선 부재, 종료 3쪽의 단일 괘선과 SVG 소비 검증은 유지·보완했다.
- 새로운 원본·golden·baseline 변경은 없다. 실제 변경과 유지 범위는 [회귀 점검](assets/pr8896/regression-audit.md)에 있다.

최소 환경의 실제 노출은 공개 글꼴 5개다. 시스템·개인 글꼴 디렉터리는 숨기고 저장소 소유 MIT fixture를 명시적으로 공급한다.
Native snapshot의 합성 검사가 설치된 글꼴에 따라 건너뛰지 않게 고유한 가족 이름을 사용했다.
Rust 1.93.1, Chromium revision 1660786, locked wasm-pack과 wasm-opt 실행 영수증을 보존했다.

전체 회귀는 4,466 / 4,398 / 4,462건의 서로 겹치지 않는 기본 hash partition으로 합계 13,326건을 통과했다.
Native 공식 3종은 library 4,035건·placeholder 2건·direct export 36건으로 합계 4,073건이다.
새 WMF 검사의 실제 PASS를 기존 Native CI 선택 경로에서 확인했다. 추가 Native protocol 22건과
해당 feature의 두 integration target Clippy도 통과했다. 기존 기본 ignore/filter 설정은 유지했다.
fmt·manifest·unit-tier policy·host/WASM/workspace all-target·Native Skia Clippy와 workspace build는 현재 검사 source에서 통과했다.

## 재현성과 원격 처리 경계

잘못 재사용된 초기 Native 후보는 base와 SHA가 같은 것을 확인해 제외하고 실제로 재컴파일했다.
중단된 최적화는 완료로 계상하지 않았고 같은 제품 source로 복구했다. 체크포인트 재개는 입력·생산자·
완료 쪽 SHA를 검증했다. 공개 웹폰트 변환 중 잡힌 크기 0의 임시 파일은 유효한 프로그램과 구분했고
실제 font 프로그램의 순서·경로·SHA는 양쪽이 같다. 원본 receipt는 보존한다.
검사·문서 commit은 제품 source를 바꾸지 않았음을 각 실제 생산자와 대조했다.

원격 단계에서는 최신 base·원 head·본인 열린 PR과 병합 시뮬레이션을 재확인한다.
준비가 끝난 upstream 수용 PR의 원격 처리 범위를 승인받은 뒤 exact-head CI·self-review·merge commit으로 통합한다.
#8693 완료를 확인하고 #7620은 OPEN을 유지한다. 현재 base 대비 CI workflow 변경은 없어 후속 기록은 옵션 M이다.
병합 뒤 duration → 운영 기록·archive·devel push → 동기화 → 이슈·감사 댓글 → 전용 branch/worktree와
루트 output·사용 종료 debug 정리 순서를 따른다. 다른 작업과 fork branch는 보존한다.

## Merge 후 contributor PR comment 계획

원 PR의 세 입력과 독립 웹한글기안기 Print 전 6쪽을 기준으로, 표의 불필요한 종료 이월 제거와
실제로 이월된 빈 host 글줄 보존을 먼저 설명한다. 최종 merge SHA와 해당 head CI 주소,
로컬 13,326건·Native 공식 4,073건 및 Native/fresh WASM 전 6쪽 raw 100%를 함께 적는다.
메인터너의 원 글꼴 공급 보정은 원 기여의 종료 수정과 구분한다.
`assets/pr8896`에 보존한 각 원본의 review·overlay를 최종 merge SHA의 raw URL로 연결하고,
게시 전에 실제 이미지 형식·SHA-256과 입력·쪽 대응을 확인한다. #8693 완료를 확인하고
#7620의 별도 잔여 범위는 유지한다. 운영 기록·devel 동기화 뒤 중복 댓글을 확인해 게시한다.

이 PR의 큰 diff는 전쪽 TSV·manifest·이미지 등 재현 증거를 포함한다. 대형 PR 보조 절차에 따라
코드 검토·병합 시뮬레이션·독립 시각 검증과 작업지시자의 수용 결정을 각각 완료했다.
