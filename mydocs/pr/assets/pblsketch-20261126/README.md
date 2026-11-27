# 중첩 칸과 글상자 편집 검증

검토는 pblsketch의 [#9401](https://github.com/edwardkim/rhwp/pull/9401)과 [#9403](https://github.com/edwardkim/rhwp/pull/9403)을 대상으로 했다. 같은 입력으로 `devel` `48a6783990`, 두 원 기여를 합친 `2647cad077`, 메인테이너 보완본을 비교했다. Rust 편집과 포인터 적중 로직의 최종 변경은 `995c962841`, Studio E2E는 `7a09b7faca`, 공개 JS·타입 선언 동기화는 `13c1512794`에 있다.

| 실제 조작 | 원 기여 누적 | 메인테이너 보완 후 |
| --- | --- | --- |
| 인라인 수식 다음 `B\|OX`를 Enter로 나누고 Backspace로 합치기 | 병합이 커서 위치 1을 반환 | 수식과 B를 센 위치 2를 반환 |
| 위 병합 실행 취소 | 빈 문단과 `BOX` | `B`와 `OX` |
| 다시 실행하고 K 입력 | `KBOX` | `BKOX` |
| 표 칸 안 둘째 글상자의 글 끝 5px 오른쪽 클릭 | 묶음의 선택 손잡이가 나타남 | 둘째 글상자 끝에 커서가 생김 |
| 해당 위치에 K 입력 | 둘째 글상자 글이 그대로 남음 | `둘째 상자K`, 실행 취소 시 `둘째 상자` |

[수정 전후 비교값](before-after.tsv) · [수정 전 클릭 화면](group-click.png) · [수정 후 같은 위치](group-click-corrected.png)

## 사용한 입력과 검증 경로

- 표 칸 안의 두 글상자는 `tests/fixtures/group_textbox_cell/cell-group-textbox.hwp`, 글상자 안의 표 칸은 `samples/issue8513/textbox_floating_table_offset.hwpx`다. 본문 일반 글상자는 `samples/issue6974/synth_textbox_vgrow.hwp`를 썼다.
- `tests/fixtures/group_textbox_cell/table-inline-equation.hwpx`는 글상자 안 표 칸의 `BOX` 앞에 공개 수식 `x+1`을 둔 후 rhwp로 저장한 문서다. 입력 SHA-256은 `fc8a54555598959f381f549d551b713cc2c4cf4df6df694d1ed0e5dddb16169b`다. 커서 기대값은 수식 한 자리와 문자 B 한 자리에서 정했다. HWP와 HWPX 각각에서 두 번 저장하고 다시 연 뒤에도 수식 1개와 `BKOX`가 남았다.
- 브라우저는 Chrome 156에서 고정 `Cargo.lock`과 `wasm-opt`로 만든 각 빌드의 WASM을 로드했다. 메인테이너 보완본에서 포인터 11곳과 크기를 달리한 글자 16곳의 조회값, 두 저장 형식의 문단 비교 32건이 일치했다. 공개 배포용 JS에서도 같은 비교를 통과했다. [WASM 해시와 위치별 집계](wasm-builds.tsv)
- Studio에 실제 마우스·키보드로 입력한 8사례를 확인했다. 선택 중인 묶음에서 다른 글상자로 들어가기, 자식 사이 빈 곳과 테두리에서 묶음 선택하기, 수식 뒤에서 직접 입력하기도 확인했다. 재현용 자동 검사는 `rhwp-studio/e2e/nested-text-box-editing.test.mjs`다. [조작과 저장 결과](keyboard-roundtrips.tsv)
- 같은 WASM의 기존 E2E로 383쪽 행정업무운영 편람의 묶음 글상자에서 입력·실행 취소·Esc, 중첩 표의 포인터 조작(#7442), 머리말·꼬리말의 편집 이력을 확인했다. #9424의 본문 수식·인접 문자 선택 사각형도 기준과 일치했다. 수식 단독 선택 폭은 35.1px였다.
- 최신 기준의 32비트 BMP 인쇄 옵션 `setPrintBmpFullColor`가 배포용 JS에 빠져 있어 생성본에 맞췄다. `pkg/rhwp.js`와 `rhwp-studio/public/rhwp.js`, 양쪽 WASM의 SHA-256이 각각 같고 위 편람 E2E도 통과했다.
- 기존 저장 무효화 검사는 중첩 칸 삽입·삭제가 공통 치환 함수에 위임된 뒤 두 진입점을 미분류로 보고했다. `e078ea99d3`에서 두 경로의 실제 위임을 등록했다. 검사는 위임 대상이 해당 구역의 원본 스트림을 무효화하는지 추적한다. 무효화 검사 5건과 실제 저장·재열기를 각각 검증했다. [Native 원본·양 형식 재열기 전후](native-undo-corrected.tsv)

## 인쇄 출력 대조

공개 입력 4개를 실제 `devel`과 보완본에서 열어 Native Skia의 `export-png --profile print`와 브라우저 Canvas의 인쇄 프로필로 각각 전쪽을 그렸다. 같은 백엔드의 기준과 보완본 사이에는 8쌍 모두 RGB 차이가 0픽셀이었다. GroupNode의 소유 칸은 편집기의 클릭 주소에 쓰이며, 직렬화하는 렌더 트리와 인쇄 위치에는 들어가지 않는다. [입력·백엔드별 쪽수와 픽셀 해시](print-comparison.tsv)

## 검증 환경

Ubuntu 24.04.5 LTS에서 Rust 1.93.1을 사용했다. Rust 회귀 검사에는 `bwrap`으로 시스템 글꼴 경로를 가리고 저장소의 `ttfs/opensource`만 공급했다. Fontconfig에 노출된 파일은 Noto Sans KR ExtraLight·Regular, Noto Sans Symbols 2 Regular, RHWP Old Hangul Regular, RHWP Public Batang Regular 5개였다.

Studio는 같은 소스의 WASM을 Node와 Chrome에서 각각 로드했다. Node 테스트 2,837건이 모두 통과했고 건너뛴 검사는 0건이었다. TypeScript 검사와 Vite 배포 빌드도 통과했다.

Native Skia는 CI의 `release-test` 프로필을 사용했다. 라이브러리에서 4,033건, CI가 지정한 21개 모듈에서 194건이 통과했다. 기존 라이브러리의 `ignored` 12건은 합계에서 제외했다. [모듈별 실행 수](native-skia-results.tsv)

Rust 서식, Native·WASM32 Clippy와 워크스페이스 전체 타깃 Clippy(`-D warnings`), 워크스페이스 빌드 및 문서 테스트를 통과했다. 문서 테스트는 8건 통과, 기존 `ignored` 3건이었다.

일반 Rust 회귀는 `cargo nextest run --locked --cargo-profile release-test --tests`로 14,204건을 실행해 모두 통과했다. 기존 45건은 건너뛰었다. 최초 실행에서는 중첩 칸 편집 위임의 등록 누락과 일반·Skia 빌드가 같은 CLI 경로를 사용한 영향으로 2건이 실패했다. 등록을 보완하고 Native Skia 검사가 끝난 뒤 일반 빌드로 전체를 다시 실행했다. 이때 사용한 nextest 0.9.137은 저장소 지정 최소 0.9.91 이상이다. 권장 버전은 0.9.140이다. 기본 프로필을 사용했으며, 별도의 CI 소요 시간 관찰 프로필에 있는 JUnit 설정은 경고로 표시됐다.

## 11월 27일 제출 직전 devel 대조

최초 전체 검증 뒤 중첩 표 재저장과 그림·쪽 번호 수정이 `devel`에 병합됐다. 최신 `devel` `e46501863d71706304aafdc3d3f6ca35c0b2b1f1`과 후보 `98a060eb9d6e17f76b91b02869ebf312b7bd73e0`의 병합 트리는 `ec464f73f291875a1ee6327303a01dc7c2ffb105`다. 각 소스에서 Native와 배포용 WASM을 새로 빌드했다. [선행 통합 PR #9434](https://github.com/edwardkim/rhwp/pull/9434) · [중첩 표 PR #9306](https://github.com/edwardkim/rhwp/pull/9306)

- 최신 `devel`의 집중 회귀 15건을 양쪽에서 실행했다. 병합본에서는 이번 두 건의 회귀 8건을 더해 23건이 모두 통과했다. 공개 원본 4개의 편집 전 인쇄 픽셀도 두 버전의 Native·Canvas 8쌍에서 모두 일치했다. [입력별 해시](current-own-prints.tsv)
- 직전에 병합된 문서 4종의 쪽수는 독립 한컴 Print PDF와 각각 138·7·11·54쪽으로 일치했다. Native RGB와 브라우저 Canvas RGBA는 같은 백엔드의 두 버전 사이에서 각각 전 210쪽이 같았다. 독일 법령은 물리적 11쪽에 표시 쪽 번호 1~5, 7~12가 남았다. [문서별 쪽수와 기준 PDF](new-base-pages.tsv) · [쪽별 픽셀 해시](new-base-page-hashes.tsv)
- 중첩 표에 숫자 120자를 입력하고 HWP·HWPX 각각 두 차례 저장해 열었다. 편집 직후와 두 재열기에서 두 버전 모두 115쪽이었다. 102쪽에는 74자, 103쪽에는 46자가 남았다.
- 병합본의 배포용 WASM에서 포인터 11곳, 글자 크기 대조 16곳, 저장 비교 32건, 실제 Studio 입력 8사례를 다시 확인했다. 383쪽 편람의 묶음 글상자, 중첩 표 포인터, 머리말·꼬리말 편집 이력 E2E도 통과했다. 본문 맨 앞 수식의 단독 선택 폭은 양쪽 모두 35.1px였다. [빌드 해시와 회귀·저장 결과](new-base-validation.json)
- 원 PR의 두 공개 표본에서 중첩 문단을 합친 뒤 Backspace의 실행 취소를 확인했다. 첫째 글상자의 두 문단이 복원되고 둘째 글상자의 내용은 유지됐다. 글상자 안 표 칸도 `B`와 `OX` 두 문단으로 돌아왔다. [표 칸 안의 글상자](restored-group-left.png) · [글상자 안의 표 칸](restored-table-in-box.png)
