# PR #6809 검토 기록

## 판정: 승인

부분 속성 변경 시 수식의 저장 크기를 불필요하게 재계산하는 문제의 보류 사유를 실행 검증으로 해소했다. 수식 스크립트 또는 글자 크기가 실제로 바뀌는 경우의 재계산과 명시적 치수 설정은 유지된다. 로컬 수용 판정이며 원격 병합 승인을 대신하지 않는다.

## 출처와 기준

- 검토일: 2026-09-07.
- 원 PR: [#6809](https://github.com/edwardkim/rhwp/pull/6809), 작성자 `lpaiu-cs`, 원 저장소 `lpaiu-cs/rhwp`.
- 원 head: `9233278ba49b9b52ea8341938cfc7a21ce590f75`. 이번 API 확인에서도 동일하다.
- 체리픽: `31343038f`, 출처와 작성자 보존.
- 브랜치: `review/ci-green-batch-20260907-2`.
- 기준 devel: `a3a30d99d4aefb15ed0dbed96645eb1ca3595099`.
- 실행 검증 코드 HEAD: `2c864284557caba4b97d07f104ce957bb756311f`.
- 관련 이슈: [#6807](https://github.com/edwardkim/rhwp/issues/6807), API 확인 시 OPEN.

## 검토 및 검증 결과

| 계약 | 결과 |
| --- | --- |
| 색상만 변경 | 저장 폭·높이 보존 테스트 통과 |
| 빈 속성, baseline·fontName 변경 | 불필요한 크기 재계산 방지 테스트 통과 |
| 같은 script·fontSize 재적용 | 재계산하지 않음 |
| 실제 script·fontSize 변경 | 필요한 크기 재계산 유지 |
| 명시적 폭·높이 | 지정 치수 적용 유지 |
| 폭만 지정하는 추가 실물 통제군 | 기존 높이 보존 |

신규 테스트 7개가 전체 회귀 및 추가 집중 검증에서 통과했다. 추가 실물 통제군은 `samples/3-09월_교육_통합_2023.hwp`의 s0p1c0 수식에 기존 폭+100만 적용하고 높이가 그대로인지 검사했다.

전체 회귀: 9,194개 실행·9,194개 통과, 46개 건너뜀, exit 0. 추가 집중 검증: #6808 10개·#6809 7개·보안 코퍼스 6개, 합계 23개 통과, exit 0. 필터 제외분 9,217개는 미실행이다. fmt, native/WASM clippy, workspace build, workspace all-targets clippy, suite manifest check 통과.

수식 저장 치수 보존이라는 계약을 직접 검사했으며 별도의 새 한컴 PDF 시각 대조는 하지 않았다. intrinsic 수식 크기의 한컴 대비 정확도 전체나 모든 수식 표현의 렌더링 일치를 해결한 것으로 확대하지 않는다. 추가 제품 보정은 필요하지 않았다.

## CI와 병합 조건

원 head의 [Build/Test](https://github.com/edwardkim/rhwp/actions/runs/34021570173/job/101456425955), [Rust CodeQL worker](https://github.com/edwardkim/rhwp/actions/runs/34021570226/job/101455045318)는 intake 확인 시 성공했다. 별도 CodeQL NEUTRAL은 devel의 JS/TS·Python 구성 2개를 찾지 못했다는 경고다. 이를 정책상 expected skip이나 완전한 CodeQL 성공으로 취급하지 않는다. 통합 PR을 만들 경우 최신 head의 필수 체크를 별도로 확인한다.

## 병합 후 코멘트 계획

승인된 통합 PR의 merge SHA, source SHA, 실제 PR/devel CI 링크, 위 치수 보존 검증과 범위를 원 PR 및 본문의 closing references로 확인한 이슈에 기록한다. #6807이 자동 CLOSED가 되었더라도 동일 merge 증적 코멘트의 유무를 확인한다. 실제 상태와 `post_merge.md`에 따라 처리하며 현재는 comment/close를 하지 않았다.

이번 증거는 상태 단언 기반으로 새 PNG/PDF는 생성하지 않았다. 존재하지 않는 시각 자료를 코멘트에 인용하지 않는다. UTF-8 body file을 사용하고 기존 후속 코멘트가 있으면 수정 후 API로 재조회한다. 원 기여자 fork 브랜치와 임시 로그는 각각 보존·커밋 제외한다.

## PR 제출 시점 확정 기록 (2026-09-07)

- 통합 브랜치: `review/ci-green-batch-20260907-2`.
- 원본 체리픽 누적 head: `2c864284557caba4b97d07f104ce957bb756311f`.
- 메인터너 코드 보정 및 검증 대상: `fe6d157c0044dd4b402edd3b6435ad0f6edb7c6b`.
  앞선 준비 단계의 미커밋 표기는 당시 상태이며, 보정 코드는 이 커밋으로 확정했다.
- 전체 Rust 9,194개 통과/46개 skip은 보정 전 결과다. 보정 후 Rust 집중 12개,
  Studio 1,493개 통과/2개 skip, TypeScript, fmt, native/WASM lib/workspace all-targets clippy,
  workspace build와 manifest 검사를 완료했다. 보정 후 전체 Rust 회귀는 재실행하지 않았다.
- 새 WASM 제품 빌드와 브라우저 실동작 검증은 미실행이다. WASM lib clippy 및 Studio 테스트를
  실제 새 WASM 제품 실행 결과로 대체하지 않는다. 최신 통합 PR CI는 제출 뒤 확인해야 한다.
- 시각 증적 게시 절차는 [Visual Sweep의 GitHub merge comment 규약](../../manual/verification/visual_sweep_guide.md#github-merge-comment)을 따른다.
  위 증적의 실제 페이지·측정 범위만 인용하고, 미측정 flagged/시각 정확도 수치를 임의로 만들지 않는다.
  메인터너 보정 후 실제 Undo 화면을 새로 촬영한 증적은 없으며, 기존 동일 값 setter 대조 PNG를 그 증적으로 주장하지 않는다.
- merge 이후에는 이 문서에 기록한 대표 PNG를
  `https://raw.githubusercontent.com/edwardkim/rhwp/<실제-merge-SHA>/<해당-PNG-저장소-경로>`로 Markdown 이미지에 직접 삽입한다.
  자산이 devel에 포함되고 실제 devel CI가 성공한 뒤 UTF-8 body-file로 게시하고 API로 본문을 재조회한다.
  동일 목적 댓글이 있으면 이전 댓글을 수정하며 중복 등록하지 않는다. 원 PR의 직접 merge나 전체 관련 이슈 해결로 표현하지 않는다.
