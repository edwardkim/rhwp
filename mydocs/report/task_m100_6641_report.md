# Task M100 #6641 — 필드 편집 후 소유 문단 재조판 최종 보고서

- **이슈**: [#6641](https://github.com/edwardkim/rhwp/issues/6641)
- **브랜치**: `task_m100_6641`
- **최신 devel 기준**: `upstream/devel@900b56edcaff3c1f84567c3f7c9e398a0dd9e8bb`
- **exact product head**: `7f1174f1d59bc020aaa38ceb7e148a8ae77b2784`
- **product tree**: `1a32c5cd3f5bab1e720b974434d469504d9a8272`
- **완료일**: 2026-09-03 KST
- **최종 판정**: `qualified-owner-aware-field-reflow`

## 1. 결론

필드 값을 바꾼 뒤 기존 LineSeg를 비우기만 하고 실제 소유 문단 폭에서 다시 조판하지 않아,
메모리 문서와 저장·재적재 문서가 달라지던 원인을 해결했다. `batch fill --verify`의 diff나
종료 코드를 완화하지 않고 다음 지원 경로를 같은 원칙으로 수렴시켰다.

- 본문 HWP5 ClickHere
- HWPX 표 셀 가상 필드
- HWP5 표 셀 ClickHere
- HWP5 표 셀 안 글상자 ClickHere
- 깊이 2 HWPX 표 셀 ClickHere

편집 문단은 본문 frame, 표 셀 내용 폭과 padding, 글상자 폭과 margin에서 즉시 reflow된다.
해당 컨테이너의 vpos를 다시 잇고 표 dirty·section raw stream 무효화를 기존 편집 계약과
일치시켰다. 지원 경로에서 소유자를 찾지 못하거나 reflow 결과가 비면 정상 성공으로 반환하지 않는다.

## 2. 원인과 수정 계보

종전 `set_field_text_at`은 문자열·문자모양·범위·offset을 갱신하면서 LineSeg를 비웠다.
setter는 이후 section compose만 갱신했으므로 HWP5 저장기는 빈 줄 배열을 기록할 수 있었고,
재적재기는 최소 줄을 다시 합성했다. 그 결과 같은 편집이 메모리에서는 0줄, 재적재 뒤에는
1줄 이상이 되어 `--verify`가 exit 3과 diff를 반환했다.

수정은 다음 경계에 한정했다.

1. mutation 전에 본문 또는 중첩 소유 문단 경로의 실재를 확인한다.
2. 본문은 기존 흐름 끝을 보존하고 해당 문단 reflow 뒤 section vpos를 다시 잇는다.
3. 중첩 경로는 `TableCell`·`TextBox` 의미를 잃지 않고 최내곽 소유 폭에서 reflow한다.
4. reflow 결과가 비면 `InvalidField`로 실패해 stale 조판의 silent success를 막는다.
5. HWPX 가상 셀의 공개 합성 fieldId(`ctrl_id == 0`)도 by-name과 같은 cell mutation으로 수렴시킨다.
6. by-id 가상 셀 경로까지 section `raw_stream`을 직접 무효화한다.

renderer 일반 정책, serializer, verify diff·exit 의미와 Gym task/reference/oracle은 바꾸지 않았다.

## 3. 공개 API 지원 행렬 결과

| 소유자·형식 | setter 표면 | 결과 |
| --- | --- | --- |
| 본문 HWP5 ClickHere | by-id / by-name / by-name-at | 저장 왕복 LineSeg·값 보존 |
| 표 셀 HWPX 가상 필드 | by-id / by-name / by-name-at | 공개 ID 읽기·쓰기 대칭 복구 |
| 표 셀 HWP5 ClickHere | by-id / by-name / by-name-at | 셀 폭 reflow·저장 왕복 보존 |
| 표 셀 안 글상자 HWP5 ClickHere | by-id / by-name / by-name-at | 글상자 소유 폭 reflow 보존 |
| 깊이 2 표 셀 HWPX ClickHere | by-id / by-name / by-name-at | 합성 LineSeg·텍스트 metadata 보존 |

`char_count`, `char_offsets`, `field_ranges`, 문자모양 경계와 채움 완료 비트는 기존 #838·#3380·
#3545 계약을 유지했다. 미지정 필드의 부분 성공, 실제 mutation 실패와 usage 오류의 종료 의미도
완화하지 않았다.

## 4. 제품 검증

focused 계약은 최초 최신 devel 보존 merge 전후에 각각 56/56 통과했다. 재동기화 merge 뒤에는
그 56건과 새 devel layout 계약 4건을 합쳐 60/60 통과했다. exact product head에서
AGENTS.md의 순서대로 native Clippy, WASM lib Clippy, workspace build와 workspace all-targets
Clippy를 `-D warnings`로 통과했다.

```text
integration manifest  1,132 sources / 4,825 static attrs / 48/48 targets
nextest                8,973/8,973 PASS / 46 policy ignored
unit tier              4,221 tests / 299 modules PASS
manifest --check       PASS
git diff --check       PASS
```

release-test 컴파일은 3분 33초였다. generated integration suite·manifest는 제출 diff에
포함하지 않았다. 상세 명령과 재동기화 영향 대사는
[Stage 4 제품 검증](../working/task_m100_6641_stage4.md)에 기록했다.

## 5. #6628 Gym 인계 전수

Gym 정상화 branch를 제품 branch에 병합하지 않았다. runner를 임시 detached worktree로 열고
exact #6641 제품 바이너리를 주입해 두 계보를 분리했다.

```text
Gym runner head   374c7416a6c2d9abe7c2701969de5f377b71183f
Gym runner tree   9888fff9aac2218d988b6defac91af7ae9d3fb93
binary            rhwp v0.8.6, Linux x86_64 debug
binary SHA-256    4334e35e3bfb7e892416e663c7a52dd055a7d82040d2a89447f32d25ccd02f34
```

두 감사 축의 뜻은 다음과 같다.

- **판별력 감사(discrimination audit)**: 정답을 통과시키는 것만으로는 부족하므로, 아무 작업도 하지
  않은 제출·입력 복사·쓰레기 파일처럼 **명백히 틀린 제출을 채점기가 제대로 탈락시키는지** 확인한다.
  시험 채점표가 정답뿐 아니라 백지 답안에도 합격점을 주는지 검사하는 것과 같다. `false-pass`는
  이런 오답이 잘못 통과한 경우이며, `false-pass 0`은 준비한 모든 오답 대조군을 정상적으로
  탈락시켰다는 뜻이다.
- **수행 경로 필요성 감사(trajectory necessity audit)**: 여러 단계로 된 기준 풀이에서 마지막으로
  결과를 완성하는 핵심 단계를 일부러 제거한 뒤, 그 불완전한 결과가 여전히 합격하는지 확인한다.
  마지막 볼트를 빼도 완성품 검사가 통과한다면 그 검사가 해당 조립 단계를 확인하지 못하는 것과 같다.
  제거했을 때 탈락하면 그 단계가 실제로 필요한 `load-bearing`이고, 제거해도 통과하면 겉보기 단계에
  불과한 `theater`다. 즉 discrimination이 **결과의 오답 배제력**을 검사한다면 trajectory는
  **풀이 경로의 핵심 단계 필요성**을 검사한다.

| 축 | 결과 | 시간 |
| --- | --- | ---: |
| BO05·BO15 canary | 2/2 built, 실패·누락·score/build 오류 0 | 1초 미만 |
| positive baseline | 21 pack, 1,035/1,035 built, 실패·skip 0 | 1,697초 |
| discrimination | 1,035 task, 1,511 control, false-pass 0 | 433초 |
| trajectory | 239/239 load-bearing, theater·예외·도구 오류 0 | 150초 |

discrimination의 `scoreErrors=116`은 artifact 과제 58건에 `input-copy`와 `garbage`를 각각 넣어
의도대로 조기 거부한 음성 증적이며, 도구 오류나 회귀가 아니다. trajectory의 796개 단일-step
과제에는 제거 전·후를 비교할 이전 경로가 없으므로 정책상 경로 필요성 검사 비대상이며, 실패나
예외성 skip이 아니다. 세 보고 봉투의 자체 validation issue도 0건이었다.

모든 제출물은 저장소 밖 임시 루트에서 만들었고 실행 종료와 함께 제거했다. 임시 runner worktree도
clean 상태를 확인한 뒤 제거했으며 source diff에 Gym 파일은 없다.

## 6. 성능 영향

동일 호스트의 #6628 직전 전수와 이번 단일 실행을 비교하면 다음과 같다.

| 축 | #6628 직전 | 최신 devel 병합 #6641 후보 | 변화 |
| --- | ---: | ---: | ---: |
| positive | 1,722.16초 | 1,697초 | -25.16초 (-1.46%) |
| discrimination | 446.53초 | 433초 | -13.53초 (-3.03%) |
| trajectory | 151.90초 | 150초 | -1.90초 (-1.25%) |
| 합계 | 2,320.59초 | 2,280초 | -40.59초 (-1.75%) |

이 비교는 각각 한 번 실행한 wall-clock 기초값이고 최신 후보 값은 초 단위로 계측했다. 두 source
tree 사이에는 최신 devel 변경과 host cache·부하 차이가 있어 microbenchmark나 성능 보장을 뜻하지
않는다. 다만 전체 1,035개
positive와 두 감사 축에서 광범위한 처리량 저하는 관측되지 않았다.

제품상 추가 비용은 필드 mutation 때 편집 문단을 소유 상자에서 다시 조판하고 vpos를 잇는 비용이다.
읽기·일반 렌더 경로에는 추가되지 않는다. 별도 field-only 반복 benchmark가 없으므로 이 보고서에서
세부 호출 지연의 상한을 주장하지 않는다.

## 7. 잔여 위험과 후속 경계

- 현재 trajectory 감사는 다단계 기준 풀이의 마지막 핵심 API 호출을 제거해 필요성을 판정하며,
  핵심 API 호출의 순서를 서로 바꾸는 경우는 검사하지 않는다. 순서 의존성이 있는 호출 조합만을
  대상으로 잘못된 순서의 결과가 거부되는지 확인하는 `order-dependency audit`는 #6628 후속
  백로그로 남긴다. 이는 #6641의 필드 재조판 결함 수정·검증 범위에 포함하지 않는다.
- HWPX는 bit 31 합성 LineSeg를 파일에서 생략하는 #5847 안전 규약을 유지한다. 깊은 HWPX 편집의
  strict `edit_verify_report`가 저장 전 합성 한 줄과 재적재 뒤 부재를 차이로 셀 수 있는 일반
  저장 정규화 경계는 #6641에서 예외 처리하지 않았다. 별도 이슈 후보이며 등록은 별도 승인 대상이다.
- #6628 권위 원장에는 external oracle이 0개다. 이번 Gym 통과는 내부 benchmark 정합 근거이지
  한컴 조판 동등성이나 제품 릴리스 적합성 증명이 아니다.
- #6628 runner는 아직 로컬 branch다. #6641 병합 뒤 #6628을 최신 devel과 동기화했을 때 제품 tree,
  Gym task/reference/check 또는 공개 fixture가 달라지면 해당 전수 축을 다시 실행해야 한다.
- nextest 실행 환경은 설치 `0.9.137`, 저장소 권고 `0.9.140`이었다. 버전 검사를 우회하지 않았고
  전체 테스트는 성공했지만 도구 업그레이드는 별도 환경 관리 범위다.

## 8. 제출 경계와 최종 판정

제품 변경은 field query와 기존 integration source·가드에 한정된다. private corpus, 외부 폰트,
Gym task/reference/oracle, generated suite·manifest와 임시 산출물은 포함하지 않았다.

최종 판정은 **`qualified-owner-aware-field-reflow`**다. #6641의 원래 실패였던 BO05·BO15는 canary와
21 pack 전수에서 모두 통과했고, 제품 전체 8,973건과 Gym의 양·음·경로 세 축이 같은 candidate
binary 계보로 연결됐다. 로컬 Stage 0~5와 PR #6673 최초 head의 GitHub Actions를 완료했고,
[self-review](../pr/archives/pr_6673_review.md)는 `승인`으로 판정했다. 이 review 기록을 포함한 최신
head의 checks·mergeability 확인, 사용자 merge 승인과 issue close 후속 절차가 남는다.
