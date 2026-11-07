# #7375 최종 검증에서 확인한 글꼴 프로그램 경계 회귀 보정

## 수정 전 실제 결과

검증 HEAD는 `2f4fc5a4f7c0f0e3a2fb8714349754083f8aa4c6`, 고정 base는
`1e16a19aede5ccb2657d35974940eb596286cdf1`이다. 전체 nextest는
12,942건 중 12,938 PASS·4 FAIL·45 skip, 1,014.256초에 종료했다. 아래 네 실패를
검사 제거·공차 완화로 숨기지 않는다.

- `issue_1939_hwp5_origin_hwpx_strict_render_diff_is_stable`: 기존 HWP5-in-HWPX
  입력의 HWPX 왕복 변위가 110.72px다. 독립된 고정 base 실행은 0px이며 양쪽 82쪽이다.
- `completed_current_object_frame_keeps_the_next_table_print_origin`와
  `final_fragment_keeps_the_empty_line_box_inside_its_print_border`: 동일한 macOS 최소 글꼴
  sandbox와 `XDG_CACHE_HOME`·웹 공급 캐시에서86712의 종료 괘선이 어긋난다.
  Native 기준도 같은 환경으로 재현해28쪽 앞 표 높이 307.7px 대 후보452.9px를 확인했다.
  개인 글꼴이 보이는 다른 캐시 조건의 성공으로 이 실패를 면제하지 않는다.
- `exact_primary_program_certificate_preserves_native_program_boundaries`: 원본 프로그램
  출처가 있는 모든 슬롯에도 설치 프로그램 인증을 반환한다. 기존 원본 소유 계약 위반이다.

후보 Native 20조건 562쪽은 모두 98% 이상이지만 고정 base보다 13쪽에서 0.1%p 초과
하락했다. 분할 저장본 11쪽·개인정보 4쪽·LPG 28쪽이며 최대 0.39424%p다.
리베이스 전 후보와는 하락이 없다는 사실을 새 base 대비 하락 면제로 사용하지 않는다.

## 공통 원인과 보정 후보

1. 주 프로그램의 동일 이름 인증이 `native_program` 출처를 가진 슬롯도 받아들였다.
   문서에 이미 있는 원 프로그램 출처는 자동 설치 프로그램 인증으로 승격하지 않는다.
   다른 슬롯의 정상 인증·사용자 명시 선택·기존 원본 전진 보호는 유지한다.
2. 선택한 굵기·face 조회 이름을 family보다 먼저 보존하는 경로가 HFT의 논리 별칭에도
   적용됐다. 분할 저장본 67쪽은 base/후보 렌더 트리가 바이트 동일하고, 인증한 프로그램
   ID도 같지만 후보가 원본 한글 별칭을 앞에 넣어 다른 글리프 fallback을 먼저 조회했다.
   원본 메트릭을 보존하는 동일 주 HFT 프로그램 인증은 기존 정규화 표시 체인을 유지한다.
   실제 명시 face 선택·문서별 슬롯 선택·원본 전진 변경 요청은 구분해 보존한다.
3. 검증된 원본 TTF에 자연 leader 전진을 허용하는 판정이 HFT 원본 소유를 확인하지 않았다.
   76076의 왕복 전후 같은 하이픈 문자열이 159.3px에서 260.8px로 달라졌다. 유형 미상 원본의
   HWPX 저장 시 추가되는 TTF 표기만으로 기존 leader를 자연 전진으로 승격한 것이 원인이다.
   기본 leader 규칙을 유지하고, 명시적 자연 전진 선택만 별도 계약으로 보존한다.
   로컬 `dbebd3bd1a`의 합성 TTF 검사는 유형·폭 표 hit를 자연 전진 선택으로 일반화한
   기대값이었다. 이를 자동 leader 유지와 명시 선택의 양쪽 경계 검사로 교정한다.
   원 Task #352의 기본 동작과 사용자가 승인한 Haansoft Batang 명시 선택은 함께 보호한다.

새 관계형 검사는 `tests/cases/issue_1994_document_font_program_scope.rs`와
`tests/cases/issue_1994_natural_program_dash_contract.rs`에 추가한다. src 내부에 새
`#[cfg(test)]` 모듈을 추가하지 않는다. 기존 네 실패와 일반 TTF·실제 face·명시 선택의
음성 대조군도 함께 재검증한다.

## 증거와 현재 한계

수정 전 frozen CLI·전쪽 TSV·렌더 트리와 전체 nextest 실패가 실제 근거다. 추가한 임시
before API harness는 이전 rlib의 의존성을 해소하지 못한 E0463 컴파일 오류로 실행되지
않았으므로 제품 검사 FAIL로 세지 않는다. 구현의 성패는 새 producer의 집중·전체 검사와
같은 고정 base·리베이스 전 결과의 전쪽 Native/fresh WASM 비교로 판정한다.

상세 로그·중간 자료는 ignored
`output/open-issues-oldest-20261105/pr-final-20261107/rebase-1e16a19a-20261107/`와
`program-boundary-repair-20261107/`에 둔다. 미완료 시각 캡처는 통과로 세지 않으며 기존
PNG·TSV·원본·PDF는 보존한다. Mac WASM은 locked wrapper web/no-opt 로컬 대체 빌드다.
새 검증·필수 편람·실제 backend 확인이 끝나기 전 제출 준비 완료로 판정하지 않는다.

사용자 최신 지시에 따라 ubuntu-ted의 미병합 지침은 반영하지 않고, 원격 push·PR 생성은
하지 않는다. CBTA의 추가 보정 제외와 기존 잔여 기록 보존도 유지한다.

## 첫 집중 재검증과 후속 보정

첫 보정본 Native 빌드 후 집중31건 중29 PASS·2 FAIL이다. 기존 전체 실패 중 원본 프로그램
소유와86712 두 표 경계는 복구됐으나76076의 왕복 변위는 남았다. 새 HFT 별칭 검사도
같은 주 family의 문자열 일치라는 불필요한 조건 때문에 실패했다. 공급 프로그램의 원본 조회
이름과 보존 모드로 자동 HFT 인증을 구분하고, 유형 표기만으로 자연 전진을 부여한 확장을
제거한 다음 후보를 다시 검증한다. 이 단계의29 PASS를 최종 통과로 표시하지 않는다.


## 동일 프로그램 별칭의 현재 줄 구성 회귀

`097110d6b3`의 자동 Native 집중 검사는138건 중136 PASS·2 FAIL이다. 앞서 개별
harness에서 통과로 관측한86712 두 건은 `XDG_CACHE_HOME`·웹 공급 캐시가 달랐다.
같은 최소 글꼴 조건으로 검증한 결과가 아니므로 복구 근거에서 제외한다. 자동138건과
독립 Core 진단에서는 KoPub Light 인증을 제거했을 때만 Print 프레임으로 돌아온다.
조회 이름을 `KoPubDotumLight`에서 같은 파일의 `KoPub돋움체 Light`로 바꿔도 복구된다.

원 검사는 통합 PR #8228의 `3e02fe2749`에서 추가됐으며, 동일 원문과 글꼴 공급 한컴
Print64쪽 및25~29쪽98% 이상을 근거로 두 표의 괘선을 보호한다. 기대값을 유지한다.
기존 `source_program_projection_slots` 생산은 원본 이름과 paint 첫 이름의 문자열 일치를
요구했고, 소비자는 검증한 Light 프로그램의 한국어 전체 이름만 인정했다. 설치한 같은
SFNT의 name6 조회 이름으로 바뀌면 현재 줄 재구성이 누락되고 저장 대체 글꼴의 프레임을
그대로 사용한다. 설치 별칭 레지스트리의 같은 프로그램·같은 굵기만 공통 생산/소비 양쪽에서
인정한다. Medium/Bold·World나 계열 이름만 같은 다른 프로그램은 검증 범위에 넣지 않는다.

새 관계 검사는 `tests/cases/issue_7765_current_source_frames.rs`에서 공개 인증 API를
통해 원본·설치 별칭 선택 전후의 표 프레임·내용 소유를 대조한다. 현재 보정 전 frozen097
실행은1건 실제 FAIL:28쪽 표 아래가385.2px에서530.4px로 바뀐다. 새 빌드의 after 결과와
동일 환경 자동 검사·Native/fresh WASM 출력은 아직 미완료다. 수치나 기존 Print 핀을 완화하지 않는다.
진단·재현 로그는 `program-boundary-final-20261107/diagnosis/`에 보존한다.

`45308777e8`에서 같은 최소 글꼴·캐시의 개별32건과 자동139건은 모두 PASS다.
새 별칭 검사는 수정 전1 FAIL에서 수정 후1 PASS로 바뀌었고 기존 두 괘선 검사도 통과했다.
Native25~29쪽은96.98466/99.38866/98.13817/99.98519/99.99286%다.28·29쪽 review를
직접 열어 표 종료·다음 시작·빈 종료 줄과 후속 내용 소유를 확인했다.25쪽의96.98%를
98% 통과로 표시하지 않으며, 기존 검사 두 건의28·29쪽 복구와 구분한다.
첫 시각 드라이버는 공개 웹폰트 캐시가 sandbox 읽기 허용 경로에 없어 실패했다.
기존 최소 글꼴+공개 캐시 정책으로 재실행했으며 환경 실패 실행은 수용 근거에서 제외한다.

## 생략한 기존 설정과 명시적 자연 윤곽 선택의 분리

`097110d6b3`의 Native20조건562쪽을 고정 base와 페이지별로 끝까지 대조했다.
분할 저장본·LPG 하락은 복구됐고 개인정보4쪽만99.99006→99.73410%로0.25596%p
하락했다. 이 페이지의 마지막 `-----. ` 런은 같은 텍스트·원점에서 폭92.6→142.5px다.
기존 reference 환경에는 `preserveSourceMetrics`가 없는데, bool 기본 false를 명시적
자연 윤곽 요청과 같은 값으로 처리한 것이 원인이다. reference JSON·독립 PDF·문턱을 바꾸지 않는다.

공통 환경에서 생략과 명시 false를 구분한다. 기존 메트릭 치환의 기본값은 그대로 유지하고,
자연 glyph fit·leader 예외는 명시 false 또는 원본별 `replaceSourceMetricsFor` 선택만
소비한다. 자동 공급의 true와 사용자 선택한 원본별 예외도 유지한다. JSON 직렬화 왕복에서
이 선택을 보존하며 기존 bool 계약과 같이 null·문자열·숫자는 거부한다.
새 공개 API 관계 검사의 보정 전 frozen453 실행은1건 실제 FAIL이며, 새 후보의 after·
Native/fresh WASM 출력·필수 전체 게이트는 아직 미완료다. 이전139 PASS를 이 새 후보의
전체 통과로 재인용하지 않는다.
