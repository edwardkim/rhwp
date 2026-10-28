# #3587 Stage 6 — B2 잔여 계약·비용 계측과 B3 저장 검증

- 일자: 2026-09-12
- 승인: 메인테이너 「다음 절차를 진행하는 것을 승인합니다」.
- 기준: `377fabae6` 및 [승인된 B 계획](../plans/task_m100_3587_impl_b.md).
- 상태: 집중 계약·저장 재열기·단회 비용 계측 완료. 메인테이너의 저장본 재편집 이상 관측으로 B 완료 판정 보류.

## 이번 절편

기존 실물 표와 연구노트 블록으로 HWP/HWPX 저장·재열기와 1/10/100회 반복 비용을 확인한다.
`table-in-tbox.hwp` pi=0의 구역 경계는 제거하지 않고 거부 결과로 별도 기록한다.
독립 HWPX가 없는 파일의 rhwp 변환본은 파서 경로 검사일 뿐 독립 정답지가 아니다.
빈 문단 0/1/2개, 명시적 쪽/단 나눔 및 사본 삭제 후 나머지 문단 보존을 추가 검사한다.

측정은 fixture/count마다 별도 프로세스에서 시행한다. 요청에는 일반(비 batch) 페이지네이션과
사전검사가 포함된다. Linux `/proc/self/status`의 RSS와 프로세스 누적 HWM을 전후 기록하며,
이 수치를 순수 복제 할당량 또는 논리 예산의 엄밀한 메모리 상한으로 해석하지 않는다.
출력 저장·SVG 생성은 시간·메모리 측정 뒤 수행한다. 한 번의 실행값은 통계적 성능 보장이 아니다.

원본과 생성 문서는 `output/3587/` 아래에서 메인테이너가 직접 열 수 있게 준비한다.
구조 계약 성공과 한컴 시각 판정은 별도로 기록한다.

## 검증 결과와 적용 범위

이번 절편의 제품 변경은 없다. 제품은 Stage 5의 `333ad5c149`이며 최종 테스트 SHA는
`698279af79`다. 동일 SHA의 detached review worktree `rhwp-review-3587`에서 suite를
준비했다. generated suite/manifest는 제출하지 않는다.

| 검사 | 결과 |
| --- | --- |
| 기존 A/B와 신규 4개 계약 | **78 PASS / 0 FAIL**, 선택 밖·ignored 합계 1,260 skipped |
| HWP 실물 2종 × 삽입 위치 4개 × 저장 형식 2개 | 16회 저장·재열기 계약 PASS |
| 파생 HWPX 입력 → 반복 → HWP/HWPX 재열기 | PASS. 독립 한컴 HWPX 정답지가 아님 |
| 빈 문단 0/1/2개 × 명시적 나눔 없음/쪽/단 | 9조합 PASS. 사본 삭제 후 다른 사본 보존 포함 |
| 그룹→글상자→표/그림, 그룹 캡션, 수식 | 소유 경로 대응·새 ID·그림 자원 참조·원형/DocInfo 불변 PASS |
| fmt / native Clippy / 신규 test target Clippy / manifest | PASS |

저장 검사는 본문/중첩 문단 텍스트·빈 문단 순서·문단/글자 스타일 참조, 표/셀 구조,
최상위 표 ID 순서, 구역 용지 정보와 바탕쪽 문단 내용을 검사한다. 모든 IR 필드의 완전 동일성이나
모든 컨트롤의 양방향 저장을 증명하지 않는다. 그룹 혼합 구조와 빈 문단 조합은 합성 API 계약이며
한컴이 정상 문서로 인정한 실물 fixture가 아니다.

### 초기 검증 오류의 원인과 정정

1. `65d9b2ed55`: 76 PASS / 1 FAIL. HWP 저장기의 기존 #1915 경로가 새 첫 문단에 구역
   정의를 보강하면서 그 안의 바탕쪽 문단까지 **본문**으로 세었던 비교 오류였다.
   `src/serializer/body_text.rs`의 `first_para_with_secd`를 확인한 후, 구역 소유 내용은
   별도 `section_content`로 검사하고 본문 비교와 분리했다.
2. `14a96c7e1b`: 76 PASS / 1 FAIL. HWPX의 기존 #1407/#1584 템플릿은 첫 문단에 단 정의가
   없으면 단일 단 정의를 추가한다. 아무 ColumnDef나 무시하지 않고 해당 경우만 정확히 하나의
   `column_count=1, same_width=true, 나머지 기본값`인지 검사하도록 정정했다.
3. `cc110f21f1`: Serialize 미구현 `DocInfo`를 JSON으로 비교하여 컴파일 실패했다.
   제품 타입을 변경하지 않고 테스트의 Debug 스냅샷 비교로 정정했다.
4. 최종 `698279af79`에서 전부 함께 재실행하여 78건 통과했다.

제품 저장기·파서·조판 규칙과 기존 baseline은 변경하지 않았다. 로그는
`output/3587/b3/focused-<SHA>.log`, `clippy-698279af7.log`,
`clippy-test-698279af7.log`(43.57초), `manifest-698279af7.log`다.
전체 회귀와 PR 직전 3종 Clippy 묶음은 이번 집중 검사와 별개다.

## 1/10/100회 비용 실측

프로브 SHA `14a96c7e1b`, `release-test` 프로파일. 이후 커밋은 테스트 비교만 변경했으며
제품 코드와 프로브 함수는 동일하다. `cost-14a96c7e1.log`에는 **서로 다른 9개 프로세스의
실제 테스트 1건 및 BLOCK_COST 1행씩**이 있다. 일반 CI에서는 프로브를 ignored로 둔다.

| 입력 | 사본 수 | 사전검사 µs | 반복 API µs | RSS 전→후 KiB | 프로세스 HWM 후 KiB | 반복 후 쪽 수 |
| --- | ---: | ---: | ---: | --- | ---: | ---: |
| 표 pi3 | 1 | 33 | 904 | 12,144→12,360 | 12,360 | 3 |
| 표 pi3 | 10 | 35 | 1,191 | 12,400→12,804 | 12,804 | 4 |
| 표 pi3 | 100 | 27 | 6,280 | 12,132→14,468 | 14,468 | 17 |
| 연구노트 pi12 | 1 | 37 | 441 | 11,596→11,920 | 11,920 | 3 |
| 연구노트 pi12 | 10 | 39 | 2,004 | 11,592→12,664 | 12,664 | 12 |
| 연구노트 pi12 | 100 | 37 | 18,290 | 11,800→19,864 | 19,864 | 102 |
| 글상자 pi0 — 거부 | 1 | 3 | 6 | 12,412→12,732 | 12,732 | 2 |
| 글상자 pi0 — 거부 | 10 | 3 | 6 | 12,276→12,596 | 12,596 | 2 |
| 글상자 pi0 — 거부 | 100 | 4 | 5 | 12,444→12,764 | 12,764 | 2 |

- 표: `samples/hwp_table_test.hwp`; 연구노트: `samples/rnote/labnote-001.hwp`.
- 글상자: `samples/table-in-tbox.hwp`. 세 요청 모두 `section/multicolumn paragraph boundary`
  unsupported이며 사본을 만들지 않았다. 성공 복제 성능으로 합산하지 않는다.
- 파일 로딩과 별도 preflight 뒤 repeat를 측정했다. repeat에는 내부 preflight와 정상
  페이지네이션이 포함된다. cold-start·평균/p95·최고 할당량 측정이 아니다.
- RSS 후 값은 결과 JSON 구성 과정에서 읽으므로 결과 포맷팅/allocator 영향도 포함할 수 있다.
  이 수치만으로 순수 복제 메모리나 메모리 누수 부재를 주장하지 않는다.
- 표 100회 논리 구조/대응표 예산: 1,068,400 / 444,800 bytes.
  연구노트 100회: 3,788,300 / 1,632,000 bytes. 각각 32 MiB / 8 MiB 정책 안이다.
  논리 예산은 RSS와 별개이며 이번 근거로 한도를 변경하지 않았다.

### 재실행 방법

review worktree에서 suite `--prepare` 후 `buildCaseIndex(deriveManifest())`로
`issue_3587_paragraph_block_save`의 target을 조회한다. 최종 SHA에서는
`regression_suite_019`지만 다른 SHA에 번호를 고정하지 않는다.

```bash
cargo nextest run --locked --cargo-profile release-test \
  --test regression_suite_019 -E 'test(/issue_3587_paragraph_block_save/)' \
  --target-dir /home/edward/mygithub/rhwp/target/pr-review
RHWP_3587_FIXTURE=labnote RHWP_3587_COUNT=100 \
  cargo test --locked --profile release-test --test regression_suite_019 \
  --target-dir /home/edward/mygithub/rhwp/target/pr-review \
  issue_3587_paragraph_block_save::block_cost_and_artifacts -- --exact --ignored --nocapture
```

fixture=`table/labnote/textbox`, count=`1/10/100` 각 조합을 별도 호출한다. 이번 실측에서는
`nextest list --list-type binaries-only --message-format json`의 정확한 binary를 직접
9회 실행했다(`binaries-14a96c7e1.json`). 0건 실행은 통과가 아니다.
`RHWP_3587_OUTPUT`은 count=1에서만 지정하고 존재하지 않는 명시적 경로를 사용한다.

## 메인테이너 확인용 산출물

모두 `output/3587/b3/` 아래에 있다.

| 폴더 | 내용 |
| --- | --- |
| `labnote-14a96c7e1/` | 원본 연구노트 2쪽, pi12 블록 1회 복제 결과 3쪽 |
| `table-14a96c7e1/` | 표 원본과 pi3 블록 1회 복제 결과, 각각 3쪽 |

`original.hwp`는 입력 원본, `repeated.hwp`·`repeated.hwpx`는 실제 저장 파일이다.
`original-NNN.svg`·`repeated-NNN.svg`, `mapping.json`, `measurement.json`도 있다.
SVG는 **저장 전 메모리 문서**의 렌더링이므로 저장 파일의 한컴 재열기 판정을 대신하지 않는다.
연구노트에는 내용을 아직 채우지 않았으며 빈 서식 복제 단계다.

에이전트가 연구노트 원본 2쪽/복제 3쪽 PNG를 열어 확인했다. 두 파일 SHA-256은
`9f8c9ff7927193ed8b3824c7bb7935ab35083228075773d62d8b2bb4be9487b6`로 동일하다.
한컴에서 `repeated.hwp`·`repeated.hwpx`가 정상적으로 열리고, 원본 앞뒤 서식 및 새 3쪽의
실험제목/목적·기록란·하단 서명란이 유지되는지 메인테이너 판정을 요청한다.

표 샘플 산출 과정에는 `LAYOUT_OVERFLOW` 4.5px/42.9px 경고가 남았다. 출력 로그만으로
원본 경고인지 복제 회귀인지 확정하지 않았으며 전체 시각 통과로 보고하지 않는다.

## 남은 순서

1. 생성 연구노트 HWP/HWPX의 메인테이너 실물 판정. 표 샘플 경고도 원본/결과를 구분한다.
2. B 종료 전체 회귀·workspace/WASM 포함 lint 통합 게이트.
3. B 결과 승인 후 C 내용 채우기/공개 API 계획·구현.
4. 이후 선택적 Gym에서 연구노트 내용 채우기·복제 시나리오 실행.

원격 push·PR·댓글, Gym 실행, WASM 재배포는 이번 절편에서 수행하지 않았다.

## 메인테이너 재편집 피드백 — B 완료 판정 보류

메인테이너는 원본을 Studio에서 열어 셀의 줄 수를 늘리면 표 확장과 페이지네이션이 정상이나,
복제 저장본에서는 같은 동작이 되지 않는다고 관측했다. 앞의 78 PASS와 PNG 동일성은 **저장본
재열기 후 동일 셀 편집 동작**을 검사하지 않았으므로 이 관측의 반증이나 B 완료 근거가 아니다.

초기 분리 진단은 현재 Studio dev 설정이 사용하는 `pkg/rhwp.js`·`pkg/rhwp_bg.wasm`을
Node에서 로딩하여 수행했다. 브라우저 UI 클릭·선택·키보드 경로까지 재현한 것은 아니다.
파일은 수정하지 않고 매번 새 HwpDocument에서 `splitParagraphInCell(0, pi, 1, 0, n, 0)`을
n=0..14로 호출했다. 대상은 pi12/13의 두 번째 표, 첫 번째 셀이다.

| 입력/대상 | 편집 전→후 쪽 수 | 해당 표 첫 bbox 높이 px |
| --- | --- | --- |
| 원본 labnote-001.hwp / pi12 | 2→4 | 837.8→1,027.3 |
| repeated.hwp / 원형 pi12 | 3→6 | 837.8→1,027.3 |
| repeated.hwp / 사본 pi13 | 3→5 | 837.8→1,027.3 |
| repeated.hwpx / 사본 pi13 | 3→5 | 837.8→1,027.3 |

이 제한된 경로에서는 출력본도 높이·쪽 수가 변했다. 이것만으로 올바른 분할·배치나 증상 해결을
선언하지 않는다. 메인테이너는 **2쪽 두 번째 표의 임의 셀에서 Enter만 반복**했다고 확인했다.
직접 API와 Studio 입력 경로를 구분하고 동일 조건으로 재현한 뒤 수정 계층을 결정한다.
제품 소스는 미변경이다.

후속으로 셀 index 5/17/33(6·18·34번째 셀)에 동일 API 입력을 비교했다. 모두 1→16문단,
표 첫 bbox 높이 약 837.8→1,027.3px로 증가했다(`enter-direct-cells.log`).
호스트 CDP의 별도 탭 초기화가 완료되지 않아 에이전트가 만든 context만 종료했고, 로컬
headless Chrome 152로 Studio 실제 키보드 경로를 분리 확인했다. 기존 호스트 탭은 건드리지 않았다.

`enter-reopen-probe.mjs`는 현재 `http://localhost:7700` 앱에서 세 입력을 각각 열고,
커서 API로 pi12/ci1/cell5에 진입한 뒤 **실제 키보드 Enter 15회**를 보낸다.
마우스로 셀을 선택하는 hit-test 경로는 이 실험에 포함하지 않았다.

| Studio 키보드 입력 | 셀 문단 수 | 쪽 수 |
| --- | --- | --- |
| 원본 | 1→16 | 2→4 |
| repeated.hwp | 1→16 | 3→6 |
| repeated.hwpx | 1→16 | 3→6 |

세 경우 모두 높이는 837.8→1,027.3px로 증가했고 입력 후 deferred pagination pending은 false다.
쪽 수 증가폭은 서로 다르므로 이것을 정상 조판 판정으로 간주하지 않는다. 표 분할과 뒤쪽 사본 배치,
사용자가 실제 클릭한 셀의 주소를 함께 대조해야 한다. 현재까지 **높이/쪽 수가 증가하지 않는 현상**은
이 제한된 환경에서는 재현되지 않았으며 근본 원인은 미확정이다.

증적: `output/3587/b3/enter-headless.log`, `enter-original.png`, `enter-hwp.png`, `enter-hwpx.png`.
에이전트는 `enter-hwp.png`에서 늘어난 셀을 확인했다. 사용 WASM SHA-256은
`5cd0f9fe37155593aa5e89afebbc903923935eceefbf4c50827452f1172ceba8`이며 재빌드하지 않았다.

### HWP 확정 후 마우스·키보드 재현: 확장 뒤 원형/사본 순서 역전

메인테이너가 시험 형식을 **HWP**로 확정했다. 후속 실험은 원본, 복제 없이 WASM
`exportHwp()`로 저장한 대조군, `repeated.hwp`를 각각 새로 열었다. 실제 마우스로
2쪽 두 번째 표의 6번째 셀을 클릭하고 커서 주소 `pi12/ci1/cell5`를 확인한 다음
Enter 15회를 입력했다. 세 입력 모두 셀 문단은 1→16으로 증가했다.

| 입력 | 편집 뒤 원형 본문 표 | 원형 하단 서명 표 | 사본 |
| --- | --- | --- | --- |
| 원본 | 3·4쪽으로 분할 | 4쪽, 본문 표 뒤 | 없음 |
| 복제 없이 저장한 HWP | 3·4쪽으로 분할 | 4쪽, 본문 표 뒤 | 없음 |
| 복제 저장 HWP | 3·4쪽으로 분할 | **6쪽으로 밀림** | **5쪽에 먼저 배치** |

따라서 표 높이가 전혀 증가하지 않는 증상과는 구분해야 하지만, **확장 후 원형의 서명 표가
사본 뒤로 넘어가는 배치 순서 이상**은 재현했다. 저장 대조군은 원본의 raw stream을 재사용할
수 있으므로 이 비교만으로 모든 직렬화 경로를 무혐의로 판정하지 않는다.
`rhwp dump --para 12`의 원본/복제본 비교에서는 총 문단 수 외에 표시된 원형 표 속성 및
LineSeg 차이가 없었다. 이것 역시 모든 IR 필드의 동일성을 증명하지는 않는다.

현재 소스에서 이에 대응하는 경로는 다음과 같다.

1. `typeset_table_paragraph`에서 선행 RowBreak 표가 continuation을 만들면, 같은 빈
   호스트 문단의 후행 양수 offset·문단 기준·자리차지 표를 `deferred_table_controls`에 넣는다.
   이 샘플의 서명 표는 offset=1275 HU로 이 분류에 해당한다.
2. `typeset_table_paragraph` 호출 뒤에 `flush_deferred_table_controls(Some(para_idx))`가
   실행되며, 현재 문단 자신의 보류 항목은 유지한다. 다음 표 문단의 배치를 마친 뒤에야
   이전 문단의 보류 항목을 배치한다. 원형이 마지막 문단일 때는 구역 종료 시 처리한다.
3. 따라서 원본만 있을 때와 그 뒤에 사본 표 문단이 있을 때의 배치 순서가 달라진다.
   관측 결과는 이 경로와 일치한다. 현재 WASM과 native HEAD의 정확한 빌드 동일성은 별도
   확인 전이므로 모든 원인을 이 경로 하나로 확정하지 않는다.

도입 커밋 `1048383e2d4`(#1686, 2026-07-01)은 의도적으로 후행 표를 다음 블록 뒤로
보류하여 `pr-1674`의 배치를 보정했다. `tests/issue_1686.rs`가 보호 사례다.
또한 `tests/cases/issue_6795_split_float_sibling_gets_its_own_page.rs`에는 보류 조건을
넓히면 형제 표가 다음 문단 뒤로 가는 순서 역전이 생긴다는 기존 진단이 있다.
그러므로 큐를 무조건 먼저 비우거나 offset 조건만 바꾸는 수정은 하지 않는다.

후속 수정 설계에서는 보류가 허용되는 흐름과 명시적 쪽나눔을 가진 다음 연구노트 블록의
경계를 구분하고, 원형/사본 순서와 #1686·#6795 보호 사례를 함께 검사해야 한다.
이번 진단에서는 제품 코드·baseline·입력 샘플을 바꾸지 않았으며 B 완료 판정은 보류한다.

재현: `PROBE_HEADLESS=1 PROBE_MOUSE=1 node output/3587/b3/enter-reopen-probe.mjs`.
증적: 같은 폴더의 `enter-mouse.log`, `mouse-original.png`, `mouse-resaved.png`,
`mouse-hwp.png`. 로그의 `pageControls`로 표의 소유 문단/컨트롤 및 분할 행 범위를 추적한다.

### 한컴 복사본 대조 — 복제 API 전용 결함으로 한정할 수 없음

메인테이너가 한컴에디터로 복사한 `labnote-001-cp-01.hwp`를 제공했다.
안내된 `note/`는 없었으며 실제 경로는
`/mnt/e/hwpsamples/rnote/labnote-001-cp-01.hwp`다. 이 입력은 읽기만 했고 수정하지 않았다.
SHA-256: `83f3f604192cca91bf15475f0e1de98219b6e65edd3229260fb8e83ebb0bbe79`.

두 복사본 모두 1구역·14개 본문 문단이며 pi12/13에 각각 표 3개가 있다.
CLI `dump <file> --para 13` 출력 전체를 줄 단위로 비교했을 때 다른 곳은 다음 두 곳이었다.
이는 dump에 표시되는 필드 범위의 비교이며, ID·raw record 등 전체 바이트 동등성 판정은 아니다.

| 사본 pi13 | rhwp 복사·저장본 | 한컴 복사·저장본 |
| --- | --- | --- |
| 명시적 쪽나누기 | 있음 | 없음 |
| LineSeg vertical_pos | 78,050 HU | 76,450 HU |

표 크기, RowBreak, 자리차지, 기준점·offset 및 표시된 셀 구조는 같았다.
본문 표 offset의 dump 숫자 `4294960279`는 부호 있는 값으로 **−7,017 HU(약 −24.75mm)**다.
unsigned 출력의 큰 mm 값을 실제 조판 위치로 해석하면 안 된다.
한컴의 복사 선택 범위/조작은 아직 확인하지 않았으므로 쪽나누기 차이 자체를 API 결함으로
단정하거나 기존 복제 계약을 변경하지 않는다.

현재와 동일한 Studio·WASM에서 pi12/ci1/cell5를 실제 마우스로 선택하고 Enter 15회를
입력했다. 선택 주소를 확인했으며 문단 수는 1→16으로 증가했다.

| 입력 | 편집 뒤 쪽 수 | 원형 본문 표 | 사본 | 원형 서명 표 |
| --- | ---: | --- | --- | --- |
| rhwp 복사본(앞 실험) | 6 | 3·4쪽 | 5쪽 | **6쪽** |
| 한컴 복사본(이번 실험) | 5 | 3·4쪽 | 4쪽에서 시작 | **5쪽** |

**한컴 복사본에서도 현재 rhwp의 서명 표 순서 역전이 발생했다.** rhwp의 새 복제 API를
통하지 않은 입력에서도 같은 종류의 증상이 생기므로, 복제 API의 속성 누락만으로 원인을
설명할 수 없다. 후속 표 문단이 있는 문서에서 기존 지연 배치 경로를 별도로 검증해야 한다.
명시적 쪽나누기가 없는 한컴 복사본에서도 나타나므로, 앞의 수정 방향을 쪽나누기 경계만으로
한정해서는 안 된다. 어느 경계에서 보류가 정당한지는 #1686 보호 사례와 함께 판단한다.

이 실험은 **한컴이 저장한 입력을 rhwp-studio에서 편집한 결과**다. Enter 입력 뒤
한컴에디터가 만드는 출력과 직접 대조한 것은 아니며 전체 시각 통과로 보고하지 않는다.
제품 소스는 변경하지 않았다.

재현 명령:

```bash
PROBE_HEADLESS=1 PROBE_MOUSE=1 PROBE_LABEL=hancom \
  PROBE_FILE=/mnt/e/hwpsamples/rnote/labnote-001-cp-01.hwp \
  node output/3587/b3/enter-reopen-probe.mjs
```

증적: `output/3587/b3/enter-hancom-mouse.log`, `mouse-hancom.png`.

### 한컴 2회 복사본 — 보류가 풀리는 위치 대조

추가 입력: `/mnt/e/hwpsamples/rnote/labnote-001-cp-02.hwp`.
SHA-256: `e2f939584a357f1776fa57a1bc16e696094b236154a419ac687fee70655c8e2d`.
원본 파일은 수정하지 않았다.

- 1구역·15개 본문 문단이다. pi12 원형, pi13 첫 번째 사본, pi14 두 번째 사본에 각각
  표 3개가 있다. 현재 rhwp에서 편집 전 4쪽으로 조판된다.
- pi12만 명시적 쪽나누기가 있고 pi13/14에는 없다. 세 문단의 저장 LineSeg vpos는
  모두 76,450 HU다.
- pi14의 문단 스타일 참조는 12, pi13은 0이다. 해당 두 문단의 CLI dump를 비교하면
  문단 번호와 이 참조 번호만 달랐다. 표시되는 정렬·간격·표 속성은 같지만, 이것만으로
  전체 스타일 레코드가 동일하다고 판정하지 않는다.

동일 Studio·WASM, 실제 마우스 선택 `pi12/ci1/cell5`, Enter 15회를 다시 실행했다.
셀 문단 수 1→16, 쪽 수 4→6이며 편집 뒤 문단/컨트롤 소유 주소로 배치를 추적했다.

| 쪽 | 배치된 연구노트 표 |
| --- | --- |
| 2 | 원형 pi12의 제목 표 |
| 3 | 원형 본문 표의 앞 조각 |
| 4 | 원형 본문 표의 나머지 조각 → 첫 번째 사본 pi13의 세 표 |
| 5 | **원형 pi12의 서명 표** → 두 번째 사본 pi14의 제목·본문 표 |
| 6 | 두 번째 사본 pi14의 서명 표 |

원형 서명 표는 **문서 끝이 아니라 바로 다음 표 문단 pi13 뒤에서** 배치된다.
이는 앞에서 조사한 `typeset_table_paragraph` 이후의
`flush_deferred_table_controls(Some(para_idx))` 처리 순서와 일치한다.
새 복제 API를 통하지 않은 한컴 문서에서, 사본 수를 늘려도 같은 경계 문제가 재현되었다.
마지막 쪽 수를 맞추거나 특정 사본 수에 분기하는 수정이 아니라, 보류된 표와 후속 문단의
소유·흐름 순서를 검토해야 한다는 근거다. 한컴에서 Enter 편집 후의 시각 정답은 별도이며,
이번 측정을 전체 조판 통과로 판정하지 않는다.

재현은 앞 명령의 `PROBE_LABEL=hancom-cp02`,
`PROBE_FILE=/mnt/e/hwpsamples/rnote/labnote-001-cp-02.hwp`를 사용한다.
증적: `output/3587/b3/enter-hancom-cp02-mouse.log`, `mouse-hancom-cp02.png`.
제품 코드·기존 테스트 기대값·입력 샘플은 변경하지 않았다.
