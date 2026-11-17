# #9104 Native 후속 구현 계획

## 승인 범위와 기준

작업지시자는 2026-11-17 KST에 잔여 Native 공백·밑줄 보완과 실제 대체 진단을 한 단위로
진행하도록 승인했다. 선행 검토의 실행 지시이며 같은 구현 계획의 재승인을 반복하지 않는다.
최신 기준은 `4f44355f5ff234abd1289d340cb4a3fe13297982`다. #9143에서 수용된
#9105·#9127을 중복 적용하지 않고, #9103의 완료 상태와 원 PR의 CLOSED 상태를 유지한다.
원래 기여자 assignee를 보존하며 #9104에 메인테이너 edwardkim을 추가했다.

기존 Native 증상 중 #6117·#6028은 다른 원본의 Canvas 장식 trim 범위를 해결한 CLOSED
이슈다. 이번에는 중첩 표 말미 장식 공백이 가시 본문 fit을 압축하는 경계와 독립 입력란 보존을
다루며 기존 해결 범위를 다시 열거나 입력·기대값을 바꾸지 않는다.

## 변경 범위

1. Native 보완은 기존 로컬 후보 `a955cc508a`에서 공통 paragraph_layout delta만 이식한다.
   공통 VisibleTextLineMeasure가 실제 가시 폭과 전체 ResolvedCharStyle 소유를 생산하고,
   확정된 줄의 말미 공백 전진을 배분한다. Layer/Paint·Native·Canvas가 같은 replay positions를
   소비한다. 원문 문자·장식·가시 원점, 독립 입력란·공백 전용 줄·원고지 계약은 유지한다.
   Paint trim·좌표 clamp·문서/쪽/공백 개수별 분기를 추가하지 않는다.
2. #9104는 실제 준비된 Native renderer의 일반 텍스트 face 선택을 관측한다.
   prepare의 공급 계획이나 설치 family 열거를 실제 대체 사용으로 기록하지 않는다.
   실제 for_text가 선택한 TypefaceCandidate/최종 Font의 요청·해결 family, custom/system 등의
   출처와 요청 스타일을 기록한다. 같은 선택을 다시 계산하지 않는다.
3. 관측은 opt-in이며 CLI export-png --json에서 활성화한다. 원본 face·문서 선언 대체·명시
   font-path·앞선 family의 선택과 렌더링 메트릭은 그대로다. NativeWebfontSupply.mappings는
   실제 환경 치환으로 소비되므로 진단 목적으로 채우지 않는다.
4. JSON은 nativeFontSupply.localFallbacks에 상태·nativeTextTypefaceSelection scope·관측
   records·missingRequests·누락 수·상한을 추가한다. 성공한 실제 generic 선택의 요청 이름은
   출력 시점 genericFallbacks에도 중복 없이 합친다. 원 공급 객체/FontEnvironment는 변경하지 않는다.
   metadata-only는 notRendered, 페이지 실패 또는 상한 초과는 partial로 명시한다.
5. 보고서는 256개 항목·이름 합계 64KiB로 제한하고 문자/본문 자체는 담지 않는다.
   캐시를 재생해도 같은 명령의 renderer 관측을 보존하고 새 명령에서는 초기화한다.
   이미지 내부 SVG/메타파일의 별도 렌더러까지 관측했다고 확장하지 않는다.
6. PNG JSON·문서·실제 동작 회귀를 함께 현행화한다. 기존 provenance의 nativeFontSupply
   전체 불신 필드 표시를 보존하고 새 문자열도 같은 계약을 검증한다.

## 필수 실행과 완료 조건

- 최소 공개 글꼴 환경에서 원본/선언 대체/명시 공급 우선, 등록·미등록 요청의 오프라인 CJK,
  보통/굵게·부분 glyph fallback·미지원·관측 상한·다쪽 재사용을 실제 제품 경로로 확인한다.
- 진단 활성/비활성 PNG는 바이트 단위로 같아야 한다. 네트워크 실패 기록은 실제 관측과 구분한다.
- Native 중첩 표 전 17쪽·두 형식 편람 전 383쪽의 실제 Native/fresh WASM을 먼저 비교한다.
  60·63쪽 독립 입력란과 10쪽 본문 원점을 확대 판독한 뒤 기존 4개 관계형 회귀를 이식한다.
- 원본·독립 Hancom Print·실제 공급 조건·compat·DPI·전쪽 TSV·PNG·producer SHA를 고정한다.
  잘못된 중간 출력이나 페이지별 최댓값을 합치지 않으며 보호 하한·허용치를 낮추지 않는다.
- 새 base #9143의 7개 PR 개선을 해당 입력·독립 기대값으로 대조한다. 전체 회귀·Native 공식
  3종·fresh 최적화 WASM·Rust lint·workspace·suite/unit 정책·편람 --handbook-gate를 완료한다.
- 정본 3,542행의 하락·누락 0을 확인하고 최종 전쪽 결과의 상승값은 하한에도 반영한다.
  최신 devel·본인 열린 PR(Draft 포함)·병합 시뮬레이션을 다시 확인한다.
- 새 code 후보가 준비되면 정확한 diff·PR 본문·후속 처리안을 제시하고 원격 승인 범위를 확인한다.
  이번 단계에서 remote push·PR 생성·공개 댓글·merge·issue close는 수행하지 않는다.

## 작업 위치와 보존

브랜치 `task/9104-native-font-reporting`, 전용 worktree에서 진행한다. 기존 후보·다른 작업·
사용자 글꼴·Studio 배포 JS/WASM은 보존한다. 공유 target은 기본 작업공간의 target/pr-review,
중간 산출물은 기본 작업공간 output/issue9104 아래에 둔다. 오늘할일·PR 번호 기반 review는
실제 PR 채번과 code head CI 성공 뒤 정본 절차에 따라 기록한다.

## 로컬 구현·검증 결과

승인한 두 범위의 구현과 필수 Rust·Native·fresh WASM·전쪽 비교를 완료했다.
[결과 보고](../report/task_m100_9104_report.md)와 [최종 증거](../working/assets/issue9104/final/README.md)에
실제 source·범위·결과를 기록했다. 원 PR의 중복 수용과 원격 처리는 수행하지 않았다.
보호 상승분 2행과 consumer 해시 연결을 갱신하고 관련 267개 계약과 실제 편람 게이트를 재확인했다.
최종 제출 무결성 확인 후 새 원격 처리 승인을 확인한다.
