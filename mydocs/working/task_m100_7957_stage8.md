# Task #7957 8단계 — 최신 devel 통합과 PR 등록 준비

사용자가 최신 devel 병합 → 영향 검증 → 한국어 본문 → 원격 push/Open PR 순서를 승인했다.
원격 push와 PR 생성 승인은 이 순서에 포함된다. 사용자는 신규 지원 입력의 기준 TSV 불가
범위를 예외 수용하고 공통 대조군 비교로 진행하도록 승인했다. PR review·오늘할일은 정확한
code head의 GitHub CI 성공 이후 기록한다.

## 기준과 통합

- 앞 단계 로컬 checkpoint: `d562c39566f977908fa7633abcf9c000a3897ef0`.
- 새 기준 devel: `9415b920c8dec299e8ada69b01dbd1c3c044f1f1`.
- 충돌 없는 통합 commit: `f6096ff5f92ff8866e90cb646d0421a6a7b088dd`.
- 기본 Studio 7700과 그 bundle·사용자 변경은 보존한다.
- 공유 host target은 `/home/edward/mygithub/rhwp/target/pr-review` 하나를 사용한다.

새 devel은 그림 색조·WMF crop·Print 종이 프레임과 일부 셀/이어받기 배치를 변경했다.
PDF parser/요청 정책 변경만 보고 출력 무영향으로 판정하지 않고, 실제 Native/Canvas paint까지
새 통합본을 검증한다. 새 정책의 90% 및 devel 대비 0.01%p 하락 판정은
`silhouette_raw_match_percent`로 수행한다. 앞 단계 TSV는 과거 producer의 결과로 보존한다.

## 기준 producer와 적용 가능한 대조 범위

기준 전용 detached worktree `/tmp/rhwp-issue7957-base-20261017`에서 기준 SHA를 빌드했다.
실제 기준 Native CLI의 여섯 입력은 모두 각각 기존의 쪽수·operators·image·path·parse·decoded
오류로 거절됐다. 이를 빈 이미지·낮은 점수로 바꾸지 않는다. 기준 producer/입력 hash와 실제
오류는 ignored `output/pr-review/issue7957/integration-20261017/base-probes.json`에 있다.
신규 지원 입력에는 기준 TSV를 만들 수 없으므로, 공통 지원 대조군의 base/head 비교와
신규 입력의 독립 PDF 전쪽 비교를 구분하도록 작업지시자가 명시 승인했다. 예외는 기준
producer가 거절하는 신규 입력에만 적용하며, 후보의 전쪽 90%·쪽수·직접 판독 및 공통 지원
대조군의 0.01%p 하락 기준을 면제하지 않는다.

공유 host target에서 처음 요청한 통합본 빌드는 기준 바이너리와 같은 해시를 반환했다.
이 산출물을 통합본 검증에서 제외하고, root 및 PDF backend crate root의 수정 시각만 갱신해
해당 패키지의 재컴파일을 강제했다. 코드 내용·공유 cache·검사 기준은 변경하지 않았다.
통합본은 실제 재컴파일과 공개 동작·해시를 확인한 producer만 사용한다.

## 검증 입력의 정식 보존

`intake_and_review.md` 2.8에 따라 50 MiB 미만 실물 PDF를 저장소 경로에 보존하고 원본과
SHA-256 동일성을 확인한다. operators 입력은 기존 `pdf/pr7704/6778-square-2020.pdf`와
동일하여 재사용했다. 나머지는 `samples/issue7957/README.md`의 경로·해시 표를 따른다.
앞 단계에서 로컬에만 둔 상태와 이번 최종 제출용 입력 보존을 구분한다.

16 MiB 경계 임시 파일 3개는 기존 기록의 해시와 달랐다. 원본 임시 파일은 보존하고,
같은 생성식으로 기록 해시와 일치하는 세 파일을 다시 생성해 qpdf 검사를 통과했다.
현재 입력을 읽지 않고 생성 예정 바이트의 해시만 기록하던 과거 진단 생성기 대신,
저장된 파일의 동일성·실제 해시·qpdf 결과를 확인하는 `samples/issue7962/generate.py`를 보존한다.
64 MiB 경계 세 파일은 모두 50 MiB 이상이므로 `/home/edward/rhwp-local-pdf/issue7962/`에
보존하며 Git에 포함하지 않는다. 원본 Windows 폰트도 배포하지 않는다.

## 통합 검증과 PR

최종 명령·producer·전쪽 TSV·직접 판독·정책 검사 결과를 위 ignored 통합 디렉터리에 보존한다.
앞 단계의 10,599 PASS와 313쪽 결과는 새 통합본의 결과로 재인용하지 않는다.
한국어 PR 본문은 신규 지원·회귀 보정·미완료 parser 할당 증명을 구분하고, 이슈 #7957–#7963을
연결한다. 대표 Native/Canvas review·overlay는 실제 최종 head SHA에 고정해 표시한다.
이번 승인은 devel 대상 Open PR 등록까지이며 merge·이슈 종료는 아직 수행하지 않는다.


### 추가 입력 #7966과 검증 재개 경계

사용자가 `exceeds_32.pdf`를 이번 공동 처리에 포함하도록 승인했다. 선행 검색 후 별도
#7966으로 등록·담당자를 지정했다. 원본은 `pdf/issue7966/exceeds-32.pdf`, 생성 대조군은
같은 폴더의 `image-placement-controls.pdf`로 보존한다. 기준 Native/Docker WASM과
통합 전 f6096ff5 Native/Docker WASM 모두 `page image count exceeds 32`로 거절했다.
Poppler에서 6쪽의 18개 자원이 35회 배치되는 것을 확인했다. SMask를 따로 센 것으로
혼동하지 않는다. 이 신규 입력도 기준 오류·후보 독립 PDF 전쪽 검증으로 구분한다.

공통 정책·추가 보정 계획은 공동 구현계획의 #7966 절에 연결했다. 이미지 카운터는
선택한 쪽들에서 누적하고 decode/IR image vector 예약 전에 검사한다. XObject의 반복
paint를 지우지 않으며 inline 경로도 같은 정책을 소비한다. 픽셀·decoded·XObject·연산·
ID pool 제한은 유지한다. 전체 parser 할당 경계는 증명하지 않았다.

앞선 통합 검증은 fmt·manifest·unit-tier까지 통과한 뒤 추가 소스 때문에 focused compile
중에 중단했다. 중단을 실패·전체 통과로 표시하지 않고 기존 로그를 보존한다. f6096ff5
Docker 빌드는 완료돼 변경 전 producer로 보존했다. 최종 소스와 무관한 검사를 반복하지
않으며, 바뀐 parser를 소비하는 Native/fresh WASM 및 필수 최종 회귀를 다시 판정한다.


이미지 배치 보정만 적용한 Docker WASM `57c360578d8b316a843958da42b7ce14151e246b13848bc97bc30d5eac35e6da`
은 33회 생성 대조군 3쪽을 가져왔지만 원본 7쪽의 affine Form clip에서 추가 실패했다.
각 쪽을 독립 선택한 probe로 7쪽만의 실패임을 확인했다. 이 producer는 변경 전 증거로
`image-count-20261017/count-only-pkg`에 보존한다. 같은 파일의 새 오류에 대한 사용자 지시에
따라 #7966의 필수 호환 범위로 이어서 보정한다. 세부 경로·반례는 공동 구현계획에 연결했다.

볼록 clip의 진단 Native (`--profile release-test --features native-skia`)에서는 원본 19쪽을
모두 출력했고 raw 최저 96.58220%, 미달·누락은 없다. 배치 수 대조군 3쪽과 affine clip
대조군 3쪽은 각각 전쪽 100%다. 처음 점수 호출은 잘못 복사된 기준 PNG 경로로 실패했으며,
경로를 바로잡아 같은 출력 PNG를 재측정했다. 이를 제품 오류나 재출력으로 표시하지 않는다.
6·7쪽 비교를 직접 읽어 6쪽의 원형 연결선 누락과 글꼴/간격 차이를 확인했다. 높은 점수로
이를 면제하지 않고 작업지시자에게 범위 판정을 요청했다. 전쪽 비교 화면은
`E:\tmp\issue7966-pdf-review-20261017\index.html`이다.

진단 컴파일 도중 clip 꼭짓점의 중복 처리와 공유 저장을 보완했으므로 진단 producer를
최종 source와 동일하다고 주장하지 않는다. clip은 고정 크기 polygon을 Arc로 공유해
일반 vector path·그림 handle마다 큰 꼭짓점 배열을 복제하지 않는다. 이는 polygon 예약
전체가 요청 ledger에 사전 계상됐다는 증명이 아니며 할당 감사의 미완료 경계는 유지한다.
현재 source의 `cargo check --locked`는 성공했다. 미완료 WASM 최적화는 중단하고 로그를
보존했다. 최종 source를 고정한 뒤 영향을 받는 Native/Docker WASM 검증을 수행한다.
