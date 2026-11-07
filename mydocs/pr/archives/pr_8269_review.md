# PR #8269 승인 리뷰 — 메모 읽기·편집 API

**판정은 승인이다.** pblsketch의 원본 head `3cd739252372622bfd8a24b0be43bbe94a9a290d`까지의 13커밋을 유지했다. postmelee가 `5077b216cb`의 공통 삭제·Storage 경계와 원 기여를 merge한 뒤 세 차례 보정했다. 코드 검증 head는 `b05781cff5d11c578bebf664964fad51e001ccb8`이다. [원 PR](https://github.com/edwardkim/rhwp/pull/8269) · [메인터너의 생성일 손상 재현](https://github.com/edwardkim/rhwp/pull/8269#issuecomment-6512733758)

## 원 기여와 보정

원 기여는 메모 조회·삽입·본문 수정·삭제, 복사 시 번호 재배정, Studio의 기준 글자 삭제와 Undo·Redo를 구현했다. 생성일 손상의 원인은 HWP 명령의 FILETIME 두 필드에 식별자와 해시를 쓴 데 있었다. Native와 WASM이 새 메모의 실제 시각을 기록하고, 복사할 때는 원래 시각·작성자를 보존한다.

HWP5는 각 구역 끝에 확장 바탕쪽을 두고 마지막 구역 끝에 문서의 메모 목록을 둔다. 읽을 때 메모 번호로 앞 구역의 기준 글자에 본문을 연결하고, 쓸 때 마지막 구역에 한 번 기록한다. 미편집 메모는 원본 레코드를 재사용하고 편집된 메모는 새 본문을 저장한다. 바탕쪽 목록의 이중 해석을 제거했다. [한컴 HWP 5.0 사양](https://cdn.hancom.com/link/docs/한글문서파일형식_5.0_revision1.3.pdf)

## 검증 결과

| 검사 | 관측 결과 |
|---|---|
| 한글 2024 실제 재저장 | Native·fresh WASM이 생성한 HWP→HWPX와 HWPX→HWP 모두 첫 구역의 범위 5..11, 작성자·본문·생성일을 보존했다. 원래 번호 1의 메모를 3번으로 복사한 파일에서도 두 메모의 생성일이 `2026-04-14T21:36:44Z`였다. |
| HWP 원본 레코드 | 스타일 삽입, 메모 편집 후 스타일 삽입, 변경 추적 ViewText, 재저장 시 줄 정보·문단 헤더를 기존 회귀와 함께 확인했다. 집중 검사 56건 통과. |
| Rust | 최소 글꼴 환경의 전체 nextest 12,858건 통과·45건 건너뜀. fmt, Native·WASM·workspace Clippy, workspace 빌드 및 Native Skia 공식 3종 통과. |
| Studio | 실제 WASM과 CI 선언 타입 검사 통과. 단위 검사 2,762건 통과·2건 건너뜀. Chrome에서 기준 글자 삭제·Undo·Redo 확인. |
| 확장 바탕쪽 | 시험지 원본의 목록 1개가 편집 저장 뒤에도 1개였다. 한컴 Print 1쪽과 96dpi로 비교한 raw는 Native 99.97532%, WASM 99.97611%다. 수식·본문 순서와 하단 분할 쪽 번호를 확대 판독했다. [Native TSV](../assets/pr8269/exam-native.tsv) · [WASM TSV](../assets/pr8269/exam-wasm.tsv) · [비교 PNG](../assets/pr8269/exam-review.png) |
| 편람 | Mac 사용자 글꼴이 개입한 실행은 기준에 미달했다. 지정 KoPub·함초롬 글꼴로 전체 383쪽을 다시 측정해 최저 99.96465%, 기존 기준 대비 하락 0쪽을 확인했다. [전쪽 TSV](../assets/pr8269/handbook-native.tsv) |

이전 CI의 lint는 오래된 base `0bb36e6f97`와 비교하면서 Git 출력 버퍼 한도를 넘었다. 같은 코드 head를 `7f76cee60e` 기준으로 다시 검증한 실행에서 lint와 모든 테스트 묶음이 통과했다. [CI](https://github.com/edwardkim/rhwp/actions/runs/42419836854) · [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/42419836803) · [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/42419836781) · [Adapter](https://github.com/edwardkim/rhwp/actions/runs/42419836944) · [Proptest](https://github.com/edwardkim/rhwp/actions/runs/42419836862) · [최종 정책 감사](https://github.com/edwardkim/rhwp/actions/runs/42422189490)

일반 글자 삭제 시 앵커 확인을 위해 전체 메모 목록을 조회하는 비용은 남아 있다. 새 Studio 메모 UI는 별도 작업이다.
