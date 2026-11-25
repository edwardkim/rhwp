# 보호 문서 글꼴 공급 자료의 공식화와 새 base 검증 계속

사용자의 최신 지시인 **검증 완료 후 PR·후속 처리까지 진행**에 따라, stage19의 PR 준비 후
대기 계획을 변경한다. 필수 검증을 완료하기 전에는 PR을 제출하지 않는다. #9316의 Windows
직접 PDF 기능 검사는 계속 제외한다.

글꼴 공급 자료는 [공식 자료 디렉터리](../verification/protected-font-supply/README.md)에
모았다. 23개 문서의 Native·WASM 46조건은 문서 key와 backend로 선택한다. 환경 JSON과
수용 기록은 사람이 읽을 수 있는 파일명으로 구분했다. 자료 파일 82개의 참조·SHA와 변경한
Markdown 5개의 링크를 확인했다. CI classifier는 해당 문서 변경 40개를 `review-only`로 판정했다.
원 TTF → 검증 TTC face → CFF의 우선순위와 보호 정본 TSV는 유지한다.

sample16의 글꼴 파일만 공급하고 환경 JSON을 생략했던 첫 실행은 수용하지 않았다.
정본 환경 JSON과 Gulim·Batang TTC, Arial 원본을 명시 공급한 새 base WASM의 18·35쪽에서
HWPX는 99.99%·100.00%, HWP 2022는 100.00%·100.00%로 복원됐다. 이 두 쪽의 개선을
전쪽 보호 통과로 확대하지 않으며 세 저장본을 전쪽 재검증한다.

#9318로 유입된 원본 6종 167쪽은 새 base와 후보 모두 Native·fresh WASM에서 쪽마다 98% 이상이다.
후보 fresh WASM 최저는 고베 지진 문서의 98.15%이며, base에서 하락한 쪽은 없다.
[전쪽 TSV와 생산 SHA](https://github.com/edwardkim/rhwp/blob/17b17c934b527aa523f037ca12d8713394e43c82/mydocs/verification/issue7754-nine-fidelity/incoming-pr9318/coverage.json)에 각 조건을 기록했다.
두 역할의 OLE 입력은 HWP·HWPX 각각 편집 후 두 번 저장·재열기를 실행했다. 독립 CFB/ZIP 검사에서
OLE 속성, HWPX payload, 편집 표지의 보존을 확인했다.

중첩 표의 Native 검증은 HWP·HWPX 입력에 각각 30줄과 70줄을 삽입해 HWP·HWPX로 저장하고
다시 열었다. 두 생산본의 12개 대응 상태에서 페이지 소속과 PNG가 일치했다.
[Native 편집·저장·재열기](https://github.com/edwardkim/rhwp/blob/17b17c934b527aa523f037ca12d8713394e43c82/mydocs/verification/issue7754-nine-fidelity/nested-table-native-edit-save-reopen.json)

원래 아홉 이슈의 Native 대상 583쪽은 최저 98.08%다. Native 정본 보호 23종 2,023쪽은
새 base와 후보의 PNG SHA-256이 전부 일치하며 기존 하한보다 낮은 쪽은 없다. 두 실행에서
화학 문서 74·93쪽이 100.00%로 올랐다. [전쪽 비교와 생산 기록](https://github.com/edwardkim/rhwp/blob/17b17c934b527aa523f037ca12d8713394e43c82/mydocs/verification/issue7754-nine-fidelity/protected/native/native-validation.json)

공개 글꼴 검사에는 기존 검증의 격리 절차를 사용했다. 공개 글꼴 497개를 확인했고, 4개 문서의
Native·fresh WASM 8조건이 모두 통과했다. 전쪽 공식 표시값은 100.00%다.
[전쪽 TSV, 비교 및 오버레이](https://github.com/edwardkim/rhwp/blob/17b17c934b527aa523f037ca12d8713394e43c82/mydocs/verification/issue7754-nine-fidelity/public-font-gate/public-font-validation.json)

원래 이슈의 fresh WASM, 보호 정본 fresh WASM, 최소 글꼴 Rust·lint, 남은 Studio 검증은 진행 중이다.

시장구조조사 100쪽의 Studio Canvas2D 화면과 독립 한컴 Print를 직접 대조했다.
후보의 제목과 본문이 빈 줄 묶음 아래로 복원됐다. 제목의 첫 잉크 행은 새 base의 217px에서
한컴 Print와 같은 279px로 옮겨졌다. [원본 PNG와 측정 근거](https://github.com/edwardkim/rhwp/blob/17b17c934b527aa523f037ca12d8713394e43c82/mydocs/verification/issue7754-nine-fidelity/studio/market-page100-screen/review.json)

고정 좌표·치수 회귀에 쓰는 안내서 2쪽과 규제분석 HWP 10·28·29쪽, HWPX 26쪽을 최종 생산본의
Native·fresh WASM으로 재대조했다. 안내서 2쪽은 양쪽 99.99%, 규제분석 네 쪽은 양쪽 100.00%다.
[전쪽 TSV와 확대 판독](https://github.com/edwardkim/rhwp/blob/17b17c934b527aa523f037ca12d8713394e43c82/mydocs/verification/issue7754-nine-fidelity/wasm/regression-coordinate-review/review.json)

배포용 문서 3쪽은 Native·fresh WASM 모두 100.00%였고, 단위 문구와 예산 표의 위치를 직접
확인했다. 복학원서·143E 공개 HWP·온새미로 HWP와 HWPX의 대표 구간도 새 생산본으로 대조했다.
[대표 구간의 전쪽 TSV와 PNG](https://github.com/edwardkim/rhwp/blob/17b17c934b527aa523f037ca12d8713394e43c82/mydocs/verification/issue7754-nine-fidelity/representative-print-review/review.json)

보호 fresh WASM의 첫 `sample10` 실행은 연결 그림 옵션을 빠뜨려 1쪽 raw가 31.61%였다.
[기존 지침](../manual/verification/visual_sweep_guide.md#macos-외부-연결-그림-공급)과
[수용 기록](../pr/assets/issue8856/final-20261110/sample10-linked-assets.json)에 따라
`oracle.gif`, `rdb02.gif`, `s1.jpg`의 SHA-256을 확인하고 하나씩 `--linked-image`로 공급했다.
재실행한 후보 1쪽은 99.96%로 보호 하한을 충족했고, 첫 32쪽에서 하락은 0쪽이다. 이 공급 조건으로
`sample10` 763쪽을 새 base와 후보에서 재검증한다.

Source155의 frozen 후보 소스는 `b75ce19774a4b059b26c2d134d49d5f7ce4a93b2`다.
공식 자료 정리 커밋 `433eba9d7011807d57cf235311afff15036b3ceb`까지의 변경이 모두 `mydocs/` 안에 있음을 확인했다.
제품 코드 바이트는 같으며, 검증 결과에는 실제 실행 producer의 SHA를 기록한다.

중단된 보호 검증은 완료 쪽의 TSV·PNG 및 출력 설정을 확인해 이어서 실행한다. 후보 Native의
`sample10`에서 먼저 완료된 320쪽은 체크포인트와 파일 SHA가 일치했다. 같은 조건의 미완료 쪽을
추가로 실행하고 전체 763쪽의 순서와 누락 여부를 확인한다. 중단된 Rust 실행은 기존 성공 기록을
확인한 뒤 종료 결과가 없는 검사부터 다시 실행한다.

Native 대상·보조 사례와 정본 보호 검증에 사용한 원본·기준 PDF 132개는 후보 Git blob과
바이트 단위로 비교했다. 모두 일치했고 신규 대용량 PDF나 글꼴 바이너리 추가는 없었다.
