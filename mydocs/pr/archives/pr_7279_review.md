# PR #7279 리뷰 — Chrome 다운로드 최초 저장 경합

## Metadata

2026-09-20 작성 시점 참고값이다. merge 전 최신 head와 CI를 다시 확인한다.

| 항목 | 값 |
| --- | --- |
| PR | [#7279](https://github.com/edwardkim/rhwp/pull/7279) |
| 작성자·경로 | postmelee, collaborator 본인 PR self-review |
| Issue | [#6988](https://github.com/edwardkim/rhwp/issues/6988) |
| Base | devel, `a3de5826c3b404bba8d7f3383d55c947f5a35aef` |
| 제출 후보 head | `a0784fe5e16689b5c2f876081efdc9f932baa512` |
| 검증한 구현 | `cf76f3120a89745fd39fe1f31b48ddeba10e28e4` |
| 최초 제출 규모 | 8 files, +401 / -27. 이 리뷰와 보고서 상태 갱신은 후속 문서 commit이다. |
| GitHub 상태 | Open, isDraft=false, MERGEABLE / BLOCKED, CI 진행 중 |

## 변경과 검토

최초 session 상태 저장보다 먼저 도착한 filename/complete 이벤트가 미추적 상태로 소진되는 결함을
수정했다. Chrome adapter의 생성·변경 listener가 첫 await 이전에 ID별 Promise 큐에 등록되며,
최초 조회·저장부터 후보 처리와 terminal 기록까지 직렬화된다. 공통 파일 분류와 설정 정책은 유지된다.

- rejection을 처리한 체인을 다음 이벤트가 이어받으므로 초기 실패가 큐를 막지 않는다.
  Map은 완료한 Promise가 마지막 항목인 경우에만 제거하며, 서로 다른 ID는 독립적으로 진행한다.
- 최초 read/write 보류, 중복 created/complete, 다른 ID 진행, 초기 write 실패 후 후속 created 복구를 확인했다.
  handled 기록 후 viewer를 여는 순서와 미추적·과거 항목·XLSX·자체 Blob·autoOpen=false 제외가 유지된다.
- 동작 기반 회귀는 실제 제품 listener 진입점을 사용한다. Node mock과 실제 Chrome의 저장 지연에서
  수정 전 결함을 검출하고 수정 후 통과했다. E2E는 browser 이벤트를 합성하지 않으며 실제 파일과 탭을 확인한다.
- 수정이 필요한 코드 결함은 발견하지 못했다. 제출 문서의 remote와 보고서 링크만 원본 저장소로 바로잡았다.

## 검증과 입력 공급

상세 명령·환경·로그와 경계별 결과는 [구현·검증 보고서](../../report/task_m100_6988_report.md)를 따른다.

| 검증 | 완료 결과 |
| --- | --- |
| Chrome/shared Node | 170 passed, 0 failed |
| JS 문법·Chrome dist 계약 | 통과 |
| 실제 Chrome E2E | HWP 8건 각각 탭 1개(대조군 3회·저장 지연군 3회 포함), XLSX 2건과 자체 Blob 탭 0개. 경합 입력 원본 바이트 보존 |
| 수정 전 음성 대조 | Node 3건 실패, 실제 Chrome 지연 사례에서 파일 다운로드 완료 후 viewer 대기 실패 |
| 검증 결과 재사용 | 구현 SHA 이후 제품·테스트 변경 없음. source/dist adapter 바이트 일치 확인 |
| 최신 base 통합 | 제출 직전 upstream/devel을 fetch했고 base SHA 유지. merge-tree 통과, tree `a10f8088453fa91e2dcfe50944358ba25fe987bf` |
| 문서 | diff --check와 상대 링크 확인 통과 |

기존 fixture 두 건의 실제 바이트를 검토 head `c61cc983e3fc347b0db655890ebb300af58e46bb`의
Git blob/LFS oid와 비교해 일치함을 확인했다. 제출 후보까지 fixture 변경은 없다.

| 저장소 입력·출처 | SHA-256 |
| --- | --- |
| 기존 `samples/re-font-dotum-empty-hancom.hwp` (8,704 bytes) | `1ee8871f37bec2e97d0928709dc411c0eacad656f35aa3d92c5cf89f61c5761b` |
| 기존 `samples/hwp3-pagedef-1915.hwp` (2,460 bytes) | `b272fdd218b4e91355167e63438a1605ef6902d75970c4ff5b8bae67087122d0` |

렌더링·조판 원칙과 Visual Sweep은 비해당이다. 측정·배치·분할·paint·출력 backend를 변경하지 않았다.
새 fixture·baseline·Rust source/helper·Studio source 변경이 없어 해당 전체 회귀와 Rust lint는 생략했다.
확장 실행용 WASM은 최신 기준 Rust로 native `--no-opt` 빌드했다. 실제 worker suspend/resume,
자연 발생 빈도, Edge/Firefox/Safari 실행은 미검증이다. 영구 storage 장애 복구는 변경 범위 밖이다.

## 최종 판정

- 판정: 승인
- 근거: 해당 변경 범위의 로컬 검증과 코드 리뷰를 완료했고 수정이 필요한 결함이 없다.
- merge 전 조건: 문서 후속 commit을 포함한 최신 PR head의 required CI 통과와 작업지시자 merge 승인.
- self-review 문서 기록이며 GitHub 원격 APPROVE 또는 merge 실행을 뜻하지 않는다. reviewer는 지정하지 않았다.
- 소형 단일 PR이며 추가 코드 보정·통합·분리 단계가 없어 별도 review_impl은 생략한다.
  이 기록을 같은 branch의 후속 commit으로 제출한 뒤 CI를 확인하고, merge는 별도 지시에 따른다.
