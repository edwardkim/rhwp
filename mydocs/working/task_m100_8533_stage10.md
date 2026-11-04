# #8533 편람 옵션 단독 실행의 설치 캐시 보정

2026-11-04 사용자가 공개 글꼴 경로를 명시하지 않고 수정된 Visual Sweep 옵션만으로
편람이 정상 출력되는지 확인하도록 지시했다. 개인 글꼴 파일과 CoreText 조회를 차단한
Mac에서 `--handbook-gate`만 지정한 실제 실행은 7.263초 뒤 공급 글꼴이 없다는 오류로
종료됐다. 앞선 경로 명시 Native 383쪽 PASS를 이 실행의 PASS로 대신하지 않는다.

원인은 Fontconfig 설치 캐시였다. `fc-match`는 읽을 수 없는 개인 글꼴 파일도 반환했고
`is_file()` 메타데이터 조회는 성공했다. `fc-list`도 같은 파일의 family/full/PostScript 이름을
반환했다. 편람 보완과 일반 Native 웹폰트 준비 양쪽이 이 이름을 설치 근거로 받아
공개 공급을 생략했다. 실제 격리 안에서 KoPub 6 face·HCR이 모두 설치됐다고 잘못 판정되고
웹폰트 공급 0건이 되는 것을 재현했다.

공통 `readable_font_file`로 실제 파일 읽기를 확인한 뒤에만 설치 근거로 사용한다.
`fc-list`는 이름과 프로그램 경로를 함께 받고 같은 TTC 파일의 읽기 확인은 한 번만 한다.
명시 사용자 치환, 유효한 로컬 우선순위, 네트워크·원본 해시 실패의 보류 계약은 유지한다.

새 반례 2건은 실제 2 FAIL → 관련 Python 29 PASS다. 읽을 수 있는 로컬 face는 그대로
수용하며, 읽을 수 없는 캐시 face만 제외하는 정상 대조군을 포함한다. 최초 unittest 호출의
import/class 이름 오류는 결함 검출 근거에서 제외하고, 수정한 실제 assertion 실패 로그를
`option-readable-font-before-valid.log`에 보존했다.

새 옵션 단독 전383쪽 실행은 `option-only-native-v2.log`에 기록한다. 공개 경로를 외부에서
주입하지 않고 `--handbook-gate`가 자동 공급·Native 출력·쪽별 하한 판정을 끝내야 완료다.
이 보정은 Python 공급 도구에 한정되어 Rust 제품 바이너리와 WASM 바이트를 바꾸지 않는다.
이미 완료한 전체 nextest 12,537 PASS/0 FAIL/45 skip 및 Rust 필수 게이트를 중복 실행하지 않는다.
