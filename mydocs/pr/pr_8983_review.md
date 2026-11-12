# PR #8983 검토 — 첫 서식 앞 평문 구간

## 최종 판정

**수용 권고.** 원 수정을 유지하고 부족한 필수 로컬 검증을 완료했다. 원격 처리 승인을 기다린다.

## 원 기여와 선행 해결 추적

원 head `9d1f6248c4ffd9365c9a0aab79d949fabde5c567`를 merge 이력으로 보존했다. 최신 devel에서도 첫 CharShapeRef가 뒤에서 시작하면 그 서식이 앞 평문까지 적용되는 #8943을 재현했다. 원 기여의 0 시작 평문 run은 원인을 바로잡는다.

선행 #7516 → 원 PR #7521 → 통합 #7601 → 실제 적용 `cb2bd912d5afc28e01015d79caf839664c942e63`과 현재 base의 포함을 확인했다. 기존 8개 검사와 독립 Print 원 사례를 유지했다. 원 기여에서 추가 제품 보정은 필요하지 않았다. 본문/셀의 기울임꼴 삽입점, 목록·여러 문단·Unicode/그림 경계와 혼합 입력·HWP/HWPX 두 차례 저장을 확인했다.

원 기여의 한 번 HWPX 재열기 외에 두 형식의 반복 저장과 실제 Studio 경로를 보완했고 source/test의 최종 필수 게이트를 완료했다.

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
