# PR #8982 검토 — HTML 표 border 속성

## 최종 판정

**수용 권고.** 승인받은 메인테이너 보완과 필수 로컬 검증을 완료했다. 원격 처리 승인을 기다린다.

## 원 기여와 메인테이너 보완

원 head `db88c6180c9ae66a1270937a181e749ce48a0f7d`를 merge 이력으로 보존했다. 최신 devel에도 남은 #8942의 명시적 `border` 속성 선 누락을 실제로 고친다. 무테두리 표의 기본값 변경 제안 #7992는 다루지 않는다.

원 코드를 누적 적용하면 `data-border="1"` 또는 다른 속성 값 안의 `border="1"`도 실선이 되는 새 결함을 재현했다. 승인 후 숫자 속성과 기존 이미지 style 추출이 같은 따옴표 속성 lexer를 소비하도록 보정했다. 이름·값 경계를 읽고 기존 단위·따옴표 지원 범위와 셀 CSS 우선순위를 보존한다. span·그림 단위와 실제 style 속성의 소비 경로도 정식 API 회귀로 보호했다.

제품 보정 source `0aa25a6634`, 최종 회귀 source `39f0ac0477` 사이 제품 파일 변경은 없다. 원 기여 및 그 개선을 유지하면서 불충분했던 인쇄/브라우저/저장 검증을 보완했다.

## 공통 검증과 독립 근거

- 최소 글꼴 환경의 집중 23개: #8982 9개, #8983 6개, 선행 #7516 8개. 전체 Rust 회귀 13,341개·Native 공식 3종 4,073개·모든 Rust lint·workspace build가 통과했다. 기존 skip/ignored와 세션 수집기의 종료 상태는 원장에 구분했다.
- 주요 12문서와 선행 #7521 원 사례를 포함한 Native/fresh WASM 각 13쪽은 모두 raw 100.00%다. 독립 한컴 2020 Print의 method 0·one-up·11.0.0.9136을 확인했다. source로 만든 HWP/HWPX와 그 동일 저장본의 Print를 연결했으며 RHWP가 생성한 문서를 한컴 저장본이라고 부르지 않았다.
- 기준 devel `80b1c354c29b`와 후보가 같은 저장본을 열어 만든 PNG 26개가 모두 같다. 브라우저에서는 별도로 base/head의 실제 공개 HTML paste API를 실행해 원 실패와 보정 후 정상 상태를 확인했다. 수정 전 저장 결과는 진단이며 정상 golden으로 등록하지 않았다.
- Studio 실제 paste 이벤트 6개와 HWP/HWPX 두 차례 저장·재열기 24개 비교가 통과했다. 본문·셀의 서식 있는 삽입점과 복합 입력·속성 별칭을 포함한다.
- 정본 보호 3,542행 공식 값 하락 0, 필수 Native HWPX 편람 383쪽 gate passed. 예외를 적용하지 않았다.
- [정식 원장](assets/bongseop-kim-20261112/README.md), [검사 집계](assets/bongseop-kim-20261112/validation.json), [전체 보호 비교](assets/bongseop-kim-20261112/protected-comparison.tsv).

## 조판 원칙 준수와 입력 커밋 확인

HTML 속성/문자 범위 → Document IR의 border fill/CharShapeRef → 공통 측정·paint → 실제 Native·Canvas 출력을 추적했다. 조판 높이·좌표·pagination 공식을 변경하지 않아 분할/이어받기 알고리즘 변경은 비해당이다. 문서 ID·우연한 숫자 보정·clamp·backend별 예외를 추가하지 않았다. 실제 변경한 회귀는 문자·서식·소유·내용 보존 관계이며 px·쪽수 pin은 없다. 원 두 파일과 기존 #7516의 검증 목적을 재점검했다.

수용 근거의 새 입력/PDF는 samples·pdf·mydocs/pr/assets의 검토 commit에 포함한다. 대상 font 프로그램·SVG font payload는 넣지 않는다. 실제 실행 입력 hash·인쇄 경로·producer·전쪽 TSV·직접 확대 판독·반복 저장을 원장에 연결했다. 기존 보호 입력/PDF 36개 조합의 hash도 정본과 모두 같다.

## 원격 상태와 다음 절차

원 PR의 CI는 원 기여 head의 증거다. 당시 Rust 분석은 성공했으나 CodeQL composite는 configuration 3개 누락의 neutral임을 구분했다. 그 결과를 새 보정/test head의 CI로 재사용하지 않는다. 새 수용 PR의 exact-head 필수 CI와 self-review가 필요하다.

원 fork는 실제 push=false이므로 upstream 통합 수용 PR 경로를 제안한다. 원격 push·PR·공개 댓글·merge는 이번 로컬 승인에 포함되지 않아 수행하지 않았다. 새 원격 처리안은 검증 완료 후보로 따로 승인받는다. 원 fork branch·다른 작업은 보존한다.
