# 2026-11-16 devel 리베이스 충돌 해소

`task/7732-split-cell-valign`을 `67b4ff8d3a55e5bb50c46a44315de749a4ef07a7` 위로 리베이스했다.
이전 head는 `0ca69cb0ee`, 완료 head는 `940db8afbc86f25557db113c99be9a6df5b396f4`다.
23개 커밋의 재적용 중 문서 커밋 `6bed604a27`에서 아래 세 경로가 충돌했고 제품 코드 충돌은 없었다.

| 경로 | 충돌 내용 | 해소 |
| --- | --- | --- |
| `mydocs/manual/pr_review/local_validation.md` | 같은 삽입 위치의 공개 4문서 검사와 Mac 글꼴 공급 재현 | 두 의무를 모두 보존 |
| `mydocs/manual/verification/visual_sweep_guide.md` | `last_verified` 날짜 | 최신 2026-11-16 유지 |
| `mydocs/manual/verification/visual_verification_governance.md` | 일반 기여자 공통 검사와 메인터너 전체 보호 재현 | 역할별 범위를 구분해 두 문단 모두 보존 |

세 문서의 `last_verified`는 2026-11-16이다. 공개 4문서 검사를 비공개 OTF 의무로 바꾸거나
메인터너의 정본 전체 보호를 공통 검사로 대체하지 않았다. 원본 폰트 확보와 실제 선택 확인도 유지했다.

[읽기 경로와 입력 신원 감사](source-read-path-reuse.json)에서 Rust 제품·Cargo 입력·기존 Sweep의
실행 Python/JS·core/view 경로의 변경 0개를 확인했다. 회귀 테스트·fixture·sample 트리도
리베이스 전후 `git diff --exit-code`가 0이다. 보호 검증의 입력/PDF 36파일을 SHA로 다시 대조했다.
이 근거로 소스 `2749e491e8`에서 완료한 Rust 13개 게이트와 해당 원본의 시각 산출물을 재사용한다.
현재 확인된 sample10 474쪽의 99.96%를 수용하거나 보호 하한을 낮춘다는 뜻이 아니다.

유입된 About 대화상자·CI 분류 계약·공개 4문서 실행과 새 base 정책 검사는 별도로 수행한다.
아직 실행하지 않은 검사는 이 충돌 해소를 근거로 통과 처리하지 않는다.
재현 명령은 `git fetch upstream devel`, `git rebase upstream/devel`, 세 문서 해소 뒤
`GIT_EDITOR=true git rebase --continue`다. 로그 원문은 ignored output에 보존하며 SHA는 감사 JSON에 기록한다.

## 유입 범위 실제 검증

[리베이스 후 검사](incoming-validation.json)에서 새 base의 suite 수집·unit tier·CI 분류 계약을
통과했다. Python 계약은 `PYTHONPATH=scripts/tests`를 지정해 109건을 실제 통과했다.
최초 호출은 모듈 경로 누락에 따른 import 오류였으며 통과 기록과 구분해 보존했다.
About 대화상자는 Mac의 별도 Vite 7787에서 800×360·360×640·1280×900 화면의 닫기·스크롤·
Escape·overlay 종료를 실제 확인했다. 사용자 서버 7700은 건드리지 않았다.

[공개 4문서 검사](public-four-validation.json)의 Native/fresh WASM 8건이 모두 통과했고,
각 1쪽의 공식 raw 두 자리 표시는 모두 100.00%다. crop 문서의 반올림 전 값은 99.99756%이며,
픽셀 자체가 전부 동일하다는 뜻은 아니다. 각 backend의 review·overlay 16개를 직접 판독해
영문 줄 소속·좁은 글상자의 꺾쇠와 연도·그림 자르기·그룹 도형과 그림자 보존을 확인했다.
이 8건은 전체 정본 3,542행 및 각주·실제 소비 경로의 남은 검증을 대체하지 않는다.
