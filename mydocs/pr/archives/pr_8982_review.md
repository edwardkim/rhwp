# PR #8982 검토 — HTML 표 border 속성

## 최종 판정

**승인·통합 완료.** 메인테이너 보완·필수 로컬 검증과 새 head CI·self-review를 완료했다.

## 원 기여와 메인테이너 보완

원 head `db88c6180c9ae66a1270937a181e749ce48a0f7d`를 merge 이력으로 보존했다. 검토 시작 base `80b1c354c29b`에도 남아 있던 #8942의 명시적 `border` 속성 선 누락을 실제로 고친다. 무테두리 표의 기본값 변경 제안 #7992는 다루지 않는다.

원 코드를 누적 적용하면 `data-border="1"` 또는 다른 속성 값 안의 `border="1"`도 실선이 되는 새 결함을 재현했다. 승인 후 숫자 속성과 기존 이미지 style 추출이 같은 따옴표 속성 lexer를 소비하도록 보정했다. 이름·값 경계를 읽고 기존 단위·따옴표 지원 범위와 셀 CSS 우선순위를 보존한다. span·그림 단위와 실제 style 속성의 소비 경로도 정식 API 회귀로 보호했다.

제품 보정 source `0aa25a6634`, 최종 회귀 source `39f0ac0477` 사이 제품 파일 변경은 없다. 원 기여 및 그 개선을 유지하면서 불충분했던 인쇄/브라우저/저장 검증을 보완했다.

## 공통 검증과 독립 근거

- 최소 글꼴 환경의 집중 23개: #8982 9개, #8983 6개, 선행 #7516 8개. 전체 Rust 회귀 13,341개·Native 공식 3종 4,073개·모든 Rust lint·workspace build가 통과했다. 기존 skip/ignored와 세션 수집기의 종료 상태는 원장에 구분했다.
- 주요 12문서와 선행 #7521 원 사례를 포함한 Native/fresh WASM 각 13쪽은 모두 raw 100.00%다. 독립 한컴 2020 Print의 method 0·one-up·11.0.0.9136을 확인했다. source로 만든 HWP/HWPX와 그 동일 저장본의 Print를 연결했으며 RHWP가 생성한 문서를 한컴 저장본이라고 부르지 않았다.
- 기준 devel `80b1c354c29b`와 후보가 같은 저장본을 열어 만든 PNG 26개가 모두 같다. 브라우저에서는 별도로 base/head의 실제 공개 HTML paste API를 실행해 원 실패와 보정 후 정상 상태를 확인했다. 수정 전 저장 결과는 진단이며 정상 golden으로 등록하지 않았다.
- Studio 실제 paste 이벤트 6개와 HWP/HWPX 두 차례 저장·재열기 24개 비교가 통과했다. 본문·셀의 서식 있는 삽입점과 복합 입력·속성 별칭을 포함한다.
- 정본 보호 3,542행 공식 값 하락 0, 필수 Native HWPX 편람 383쪽 gate passed. 예외를 적용하지 않았다.
- [정식 원장](../assets/bongseop-kim-20261112/README.md), [검사 집계](../assets/bongseop-kim-20261112/validation.json), [전체 보호 비교](../assets/bongseop-kim-20261112/protected-comparison.tsv).

## 조판 원칙 준수와 입력 커밋 확인

HTML 속성/문자 범위 → Document IR의 border fill/CharShapeRef → 공통 측정·paint → 실제 Native·Canvas 출력을 추적했다. 조판 높이·좌표·pagination 공식을 변경하지 않아 분할/이어받기 알고리즘 변경은 비해당이다. 문서 ID·우연한 숫자 보정·clamp·backend별 예외를 추가하지 않았다. 실제 변경한 회귀는 문자·서식·소유·내용 보존 관계이며 px·쪽수 pin은 없다. 원 두 파일과 기존 #7516의 검증 목적을 재점검했다.

수용 근거의 새 입력/PDF는 samples·pdf·mydocs/pr/assets의 검토 commit에 포함한다. 대상 font 프로그램·SVG font payload는 넣지 않는다. 실제 실행 입력 hash·인쇄 경로·producer·전쪽 TSV·직접 확대 판독·반복 저장을 원장에 연결했다. 기존 보호 입력/PDF 36개 조합의 hash도 정본과 모두 같다.

## 원격 승인·CI·self-review·병합 결과

작업지시자는 제출 후보 `62a3fe4e2982961f9356a890395dc056eca85570`의 원격 push·Open PR·CI·self-review·merge와 원 PR/이슈 후속 처리·운영 기록·정리를 승인했다. [통합 수용 PR #9017](https://github.com/edwardkim/rhwp/pull/9017)을 열어 같은 head를 검증하고 [self-review](https://github.com/edwardkim/rhwp/pull/9017#pullrequestreview-5875203068) 뒤 merge commit [`dbb0dfcdaf7aebe68e712807da3ca700774fd39b`](https://github.com/edwardkim/rhwp/commit/dbb0dfcdaf7aebe68e712807da3ca700774fd39b)로 통합했다.

- [CI](https://github.com/edwardkim/rhwp/actions/runs/43221597756): lint·Archive A–E·Build & Test 성공. 실제 기본 아카이브 13,151건과 이번/선행 관련 27건이 PASS이며, 로컬 전체 workspace 13,341건과 실행 범위를 구분한다.
- [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/43221598571): 변경 대상 Rust의 실제 분석 성공. Actions·JavaScript/TypeScript·Python은 경로 분류에 따라 분석을 생략했다. 종합 check의 `neutral`·3개 구성 누락 경고를 보존하며 전체 언어 분석 성공으로 쓰지 않는다. Adapter·Proptest와 최종 CI Impact Policy도 성공했다.
- CI의 Native·Frontend 생략과 미트리거 Render Diff는 위 로컬 Native/fresh WASM·Studio 실측과 구분한다. 새 CI 재실행·보호 설정 변경·보안 분류 변경은 없었다.
- 최신 devel `1a3ed4a7cd`의 #8956 표 조회 API 4건과 실제 병합 tree `721ccfdffb`의 27건을 추가 검증했다. 첫 실행의 공용 target 기준 라이브러리 재사용은 실패로 보존했고, 내용 변경 없이 현재 소스를 재컴파일해 27 PASS를 확인했다. GitHub의 실제 merge tree도 동일하다.
- 두 원 PR은 간접 MERGED다. #8942·#8943의 자동 CLOSED를 확인했고 #7992는 OPEN을 유지한다. 원 기여 SHA와 fork branch를 보존했다.
- duration 후처리: [duration workflow](https://github.com/edwardkim/rhwp/actions/runs/43227236387)는 success로 끝났지만 `ready:false, reason:no-verified-pr-duration-measurements`로 갱신을 보류했다. 제품 검증 CI를 재실행하지 않았다.

[실제 CI·병합 영수증](../assets/bongseop-kim-20261112/remote/merge-and-ci.json). workflow·판정 정책 변경이 없는 maintainer 일반 경로이므로 옵션 M의 mydocs 운영 기록만 devel에 반영한다. 기록 push와 devel 동기화 후 완료 댓글·감사 댓글을 게시하고 전용 branch/worktree·output 전체·유휴 debug를 정리한다.

## Merge 후 contributor PR comment 계획

첫 기여 환영·감사, 원 기여와 메인테이너 보완의 구분, 실제 CI·로컬 검사, 원 기여 SHA 보존과 fork branch 유지 사실을 한국어 존댓말로 게시한다. 운영 기록 devel 반영·동기화 뒤 이 PR에 대응하는 `border-hwp` 1쪽의 Native/fresh WASM 비교를 사용한다. 두 경로의 해당 쪽 raw는 100.00%이며 독립 한컴 Print와 같은 영역으로 대조했다. raw 수치는 자동 보조값이며 RGB 완전 일치나 사람 판정 정확도가 아님을 적는다.

- [Native 비교](../assets/bongseop-kim-20261112/direct-review/native/border-hwp.png)
- [fresh WASM 비교](../assets/bongseop-kim-20261112/direct-review/wasm/border-hwp.png)
- raw 이미지 URL은 실제 merge `dbb0dfcdaf7aebe68e712807da3ca700774fd39b`로 고정하고 PNG 형식·SHA-256을 로컬 원본과 대조한 뒤 게시한다. 게시 뒤 API의 본문·UTF-8·이미지 표시를 확인한다.
