# PR #9393 셀프 리뷰

| 항목 | 확인 내용 |
| --- | --- |
| 검토 대상 | [PR #9393](https://github.com/edwardkim/rhwp/pull/9393), 표 배치·글줄 원점과 한컴 인쇄 호환 보정 |
| 종료 대상 | #7766, #7767, #7769, #7770, #7771, #7772, #7774, #7775, #7776 |
| 검증 소스 | 비교 기준 `dd1dc8509769df46593ca91e23c06167d0436607`, 전쪽 렌더 결과를 만든 후보 `e2e6c3dc5273f5b63be9215bdd7a34adb2d19cb5`, 제출 코드 head `5e0e5819da5b34391aba6f3375e44a3d748c742c` |
| GitHub Actions | 제출 코드 head에서 성공 35건, 조건부 건너뜀 6건, 실패 0건. [CI](https://github.com/edwardkim/rhwp/actions/runs/45423314253) · [Native Skia](https://github.com/edwardkim/rhwp/actions/runs/45423314253/job/138548788011) |

## 원본과 전쪽 산출물

15개 조건에서 Native·fresh WASM 각각 2,413쪽을 독립 데스크톱 한컴 PDF에 대조했다. 동일 인쇄
옵션에서 비교할 수 있는 쪽의 기준 대비 하락은 0쪽이다. 원본 BMP 색을 보존하는 옵션을 적용한
대조를 포함해 최저는 Native 98.14%, fresh WASM 98.16%였다. #7768은 사용자 취소이고, 기존에
닫힌 #7773은 정상 대조에만 사용했다.
[구현·입력별 TSV와 review·overlay](pr_9393_review_impl.md) · [SHA-256과 실행 기록](../assets/issue7766/mac-rebase-final/verification.json)

기여자 공통 보호 검사 8건을 통과했다. 편람 HWPX Native 383쪽을 포함한 정본 보호 비교는 경로별
23조건·2,023쪽에서 공식 표시값 하락 0쪽이었다. #9388의 제주·식량안보 기존 Print와 제주 신규
한컴 PDF 저장본을 각각 독립적으로 비교했고, 같은 기준을 사용한 쪽별 제품 PNG는 후보와 devel에서
일치했다. 제주 PDF 저장본을 선택한 근거와 실제 Studio의 Canvas2D·CanvasKit 제한은 구현 검토에
기록했다. [전쪽 결과와 실제 Studio 경로](pr_9393_review_impl.md)

## 회귀 및 CI 보정

Mac 최소 글꼴 nextest에서 14,093건을 통과했고, 45건이 건너뛰어졌다. 집중 회귀 80건, fmt·Native 및
WASM Clippy·workspace build·all-target Clippy, Studio 유형과 단위 테스트를 통과했다. Studio
#2214의 픽셀 기대 차이는 기준과 후보 모두 같은 값으로 재현됐으며, HWP/HWPX 실제 편집 진단과
합성 결과는 양쪽에서 일치했다.

첫 [Native Skia 실행](https://github.com/edwardkim/rhwp/actions/runs/45394058241/job/138446041446)에서는
연속 그림자 검사가 문서 준비 직후의 상태를 기대해 실패했다. Print 글꼴 준비는 실제 페이지 출력에서
수행된다. 검사를 구형·2024 두 모드의 실제 PNG와 같은 글꼴을 명시 등록한 대조 PNG의 픽셀 일치로
보정했다. 그림자가 빠지면 대조가 달라지고, 출력 후에도 원본 저장 바이트가 유지되는 것을 검증한다.
Mac의 전체 Skia 사전 검사에서는 #7902의 병렬 테스트가 사용하는 임시 폴더 이름 충돌도 확인했다.
폴더의 고유 순번을 추가하고 8스레드로 세 차례씩 실제 실행했다. 두 보정은 회귀 테스트 파일에만
있다. [그림자 테스트](../../../tests/cases/skia_native_glyph_replay.rs) · [병렬 출력 테스트](../../../tests/cases/issue_7902_original_frame_and_content.rs)

Windows의 WMF PNG 비교에서는 DirectWrite가 정확한 파일을 선택했지만 재사용 가능한 데이터가
없어 공급 단계를 건너뛴 것을 재현했다. #9316과 같은 원본 스트림 방식으로 선택된 파일 바이트와
face 인덱스를 읽어 보정했다. 기존 WMF fixture의 자동·명시 공급 PNG가 다시 일치했고 연속 그림자와
DirectWrite 비교도 통과했다. 재사용 데이터 경로는 그대로 두고 Windows에서만 원본 스트림을 연다.
[Windows 공급 경로](../../../src/renderer/skia/text_shadow_fonts.rs)

후속 Windows 전체 검사에서는 배포 웹폰트와 같은 family의 설치 글꼴에 센트 기호 윤곽이 있어
빈 글리프 대조를 가렸다. 기존 웹폰트와 대체 파일을 시험 전용 별칭으로 공급하고 PNG의 일치와
실제 대체 출처를 함께 검사하도록 보정했다. [센트 기호 검사](../../../tests/cases/issue_8153_symbol_fallback.rs)

최소 글꼴 환경에서 갱신한 suite의 라이브러리와 CI 통합 모듈 21개를 실행해 Windows 경로 추가 전
macOS 4,225건, Ubuntu 4,227건이 통과했다. 양쪽 모두 실패 0건, 기존 ignore 12건이다. Ubuntu에서는
설치된 Fontconfig와 버전이 같은 개발용 링크 파일을 검증 환경에 공급했다. Windows 경로 추가 뒤
최소 글꼴 Mac에선 관련 68건이 통과했다. 센트 기호 시험 보정 뒤에는 Mac에서 해당 두 건을 다시
통과했다. Windows 10의 실제 설치 글꼴 환경에서는 라이브러리와 통합 모듈 21개에서 4,224건 통과,
실패 0건, 기존 ignore 12건이었다. 최종 head에서는 센트 기호 시험만 바뀌었다. 제품 코드가 같은
직전 head의 성공 결과와 최종 head의 보정 시험·남은 모듈 결과를 합산했다.
사전 검사 절차에 OS별 글꼴 격리와 모듈별 실제 실행 건수 확인을 추가했다.
[Native Skia 사전 검사 지침](../../manual/pr_review/local_validation.md#ci와-같은-native-skia-사전-검사)

## 병합 조건

최신 devel `8a26c00869e5a15cc11efa715b3687b600a61c0b`의 #9394와 코드 후보를 병합
시뮬레이션했다. Git 충돌과 두 PR이 동시에 수정한 소스·테스트 파일은 0개였다. 로컬 결합 트리와
GitHub 임시 병합 커밋의 트리가 일치했고, 위 Native Skia job이 실제로 해당 병합 커밋
`1f7b7f4ce14ef31f68a174ee7a288729717ee5d2`을 검사했다.
최종 기록 커밋의 병합 결과에서 기존 오늘할일 항목과 이번 기록의 보존을 확인하고, 정확한 문서
head의 필수 검사 및 Fast Pass 판정까지 확인한 뒤 병합한다.
