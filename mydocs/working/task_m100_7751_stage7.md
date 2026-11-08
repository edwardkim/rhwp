# #7494·#7751 최종 제출 전 검증 — 7단계

시작 HEAD `619a12d91`, 기준 `552514f423ec91f1b7171f6e6b0882a4efe034f2`.

## 확인한 상태와 독립 근거

원본 편람과 기존 KoPub 한컴2024 Print PDF를 유지한 전383쪽 검증은 Native
최저92.53354%, fresh WASM 최저92.45371%이며 누락·90% 미만은 양쪽 모두0쪽이다.
원시 TSV는 `output/pr-review/pr7494-20261016/wmf-rgb555-v2-{native,wasm}-all/handbook/silhouette.tsv`다.

동일 입력·PDF·글꼴의 기준 Native 전383쪽 비교에서는224쪽 −0.01876%p,
226쪽 −0.01823%p가 검출됐다. 점수가99%대여도 하락 gate를 면제하지 않는다.
두 쪽은 바깥 표의 셀 안에 있는 TAC 표 원점만 수평 이동했다. 저장 줄 영역,
문단 들여쓰기, 표 바깥여백의 실제 소비를 추적하고 독립 PDF 괘선·글자를 대조한다.
기준 fresh WASM 빌드는 완료했으며 전쪽 실행 결과도 같은 방식으로 비교한다.

## 구현·검증 순서

1. 두 하락 페이지의 저장 줄·문단 속성에서 기대 원점을 구하고, pen 생산부터
   중첩 표 배치까지의 중복 적용 또는 잘못된 가정을 수정한다. 문서 ID나 점수에
   맞춘 상수·조건은 추가하지 않는다. 영향 페이지를 먼저 재출력한다.
2. 기존 완료 경계3건은 유지하고, 양쪽98% 이상인297·375쪽의 독립 기대값과
   소유·포함 관계를 확인해 중요한 경계를 보호한다. 130쪽은 WASM93.34522%이므로
   px 핀을 추가하지 않는다. 실제 WMF의 공통 payload·지연 로딩 계약도 정식
   `tests/cases/`에서 검사한다. 로컬 진단 파일을 fixture로 참조하지 않는다.
3. 최종 생산 소스에서 전쪽 Native/fresh WASM TSV·devel 하락 비교·대표 PNG
   확대 판독, Rust 전체·Native Skia·lint·Studio 적용 검증을 완료한다.
4. upstream head 재확인 후 준비가 끝나면 승인된 통합 PR을 생성한다. 정확한
   code head CI 성공 뒤 같은 PR에 개별 review·오늘할일 trailing commit을 넣고,
   trailing CI·merge·`post_merge.md`의 증적 보존·원 이슈/PR·정리 절차를 따른다.

WMF 저색수 팔레트 선택의 미확정 경계는6단계 기록을 유지한다. 편람의 전쪽
통과를 모든 WMF 입력의 완전한 호환으로 보고하지 않는다.

## 기준 전쪽 비교와 Studio 실행 결과

기준 fresh WASM 전383쪽까지 완료했다. 모든 비교 행을 Decimal로 계산했고,
`stage7-all-pages-base-head-comparison.tsv`에 네 원본 TSV를 페이지별로 결합했다.
동명의 JSON에는 원본 TSV SHA-256과 아래 하락을 기록했다.

| 경로 | 쪽 | 기준 raw | 후보 raw | 변화(%p) |
| --- | ---: | ---: | ---: | ---: |
| fresh WASM | 130 | 93.35693 | 93.34522 | −0.01171 |
| Native | 224 | 99.14080 | 99.12204 | −0.01876 |
| Native | 226 | 99.16137 | 99.14314 | −0.01823 |
| fresh WASM | 226 | 99.20548 | 99.17155 | −0.03393 |

이는3쪽·4경로의 하락이며 현재 판정은 `re_review_required`다. PR을 아직
생성하지 않는다. 기준 Native의 exit1은241·290쪽의 기존90% 미달 검출이고,
383행 누락이나 프로세스 중단이 아니다. 후보에서는 이 두 쪽도90% 이상이다.

Studio `npm test`는2,023 PASS /0 FAIL /0 SKIP,43.49초에 exit0이다.
`npx tsc --noEmit`도5.41초에 exit0이다. 로그는
`logs/stage7-studio-{unit,types}.log`, 명령·종료코드는
`logs/stage7-studio-results.json`에 보존했다. 아직 전체 Rust·Skia·lint의
최종 통과를 대신하지 않는다.

정식 `issue_7751_wmf_picture_payload`는3건을 실제 실행해3 PASS였다.
`issue_7751_handbook_completed_boundaries`는 기존3건과 추가2건을 실행해5 PASS다.
297쪽은 주의표·연도·법령명의 내용 순서·같은 줄 소유와 독립 Print 시작 좌표를,
375쪽은 두 단 서식의 외곽 ±2px·셀 내용 보존·공백 포함 가운데 정렬·바깥여백
한 번 소비를 검사한다. 해당 페이지의 현재 Native/WASM raw는 각각
297쪽99.23737/99.24360%,375쪽99.37858/99.42850%다.

첫 경계 검사 호출은 새 테스트 편집 뒤 자동 suite 배정을 갱신하지 않아0건이었다.
이는 통과로 세지 않았다. manifest를 다시 준비한 후 실제5건을 실행한
`logs/stage7-completed-boundary-tests-r2.log`만 통과 증거다.
Visual Sweep 도구 검사는 `python3 -m unittest discover -s scripts/tests -p
'test_visual_sweep*.py'`에서156 PASS다(`logs/stage7-visual-sweep-tests.log`).

검증 뒤 원격 devel에 `fe5725dc384689b45d60ef8ea4c83138bc2701d8`(#7954)의
조회 CLI 통합이 들어온 것을 확인했다. 이 단계 변경을 먼저 커밋하고 리베이스한다.
기존 검증의 기준 SHA를 새 기준으로 바꿔 적지 않으며, 리베이스 이후 실제
diff·생산 소스 동일 여부와 적용 검증을 별도로 기록한다.

## 리베이스와 후속 검사

리베이스는21커밋 모두 충돌 없이 완료했다. 새 HEAD는
`c2daea095b5d0a33767a086907feec8ba5b33a27`이며, 원본·기준의 생산 경로 변경이
없음을 `rebase-fe5725dc3-provenance.json`과 range-diff에 기록했다.
원시 TSV의 실제 생산 SHA는 그대로 보존한다. 원 PR #7494는 여전히
OPEN/non-draft, source head `a3f06bedb3bbd5402082349b955fdf1312a2f4bf`다.

기본 clippy 첫 실행은 새 JSON fallback에서 이미 `Option<&Payload>`가 된 값을
다시 `as_deref`한 두 곳을 검출했다. 해당 중복만 제거한 재실행은2분33초에 exit0이다.
WASM clippy도3분25초에exit0이다. workspace 빌드·전체 target clippy·전체 Rust
회귀는 같은 절대 `target/pr-review`에서 순차 진행한다.

기존 v2 fresh WASM을 실제 Chrome에서 로드한 공개 API 진단도 통과했다.
130쪽 inline/omit-bytes JSON이 같은 `bin:0:376:wmftone_-5_0_0`를 내고,
`getSourceImageBytes`의505,441바이트가 inline과 완전히 같았다. SHA-256은
`0ec93fda910b722833af5f4ea538075123112a10ad13fa9bcfbde32c269cebc7`이다.
이전 epoch와 잘못된 variant2건도 거부했다. 이는 기존 생산 WASM의 공개 API 실행이며
리베이스 후 fresh WASM 빌드를 완료했다는 주장은 아니다.
`wasm-source-image-api-observations.json`, `logs/wasm-source-image-api.log`에 보존했다.

회색조 정식 검사의 기대 색242는 원본 실사 Print의 기대값으로 쓰지 않는다.
수정한 검사는 inline/lazy 일치·동일 회색 채널·크기 보존을 확인하고, 실제 실사
원본에서는 독립 PDF의 흰색249와 검은 글자 픽셀 보존을 함께 확인한다.
이 편집 뒤의 정식 실행 결과는 별도로 갱신한다.

독립 PDF path 재검토에서 초기224·226쪽 좌표 일부가 `W*` clipping 경계였음을
확인했다. 이를 stroke 중심으로 해석하지 않는다. `read-independent-print-strokes.py`는
CTM·q/Q 상태를 적용하고 실제 S/s만 추출해224/226/375쪽의17/24/58개 stroke를
`stage7-independent-print-strokes.json`에 기록한다.375쪽 회귀의 논리 서식틀
좌표는 clipping frame을 근거로 하며, 괘선 자체의 직접 비교와 구분한다.
226쪽 실제 수직 괘선은 현재 출력과0.144…0.578px의 차이가 있고, 기존 clip 비교만으로
괘선이 개선됐다고 판정하지 않는다. 이 차이의 원인 확인과 재검증은 남아 있다.

동일 공개 API를 기준 fresh WASM에서 실행하면 실제로 그림376의
`bin:0:376:src`가 반환돼 새 색조 신원 계약 검사에서 실패했다(exit1).
빌드·환경 실패가 아니라 `WMF tone identity missing` assertion이므로
`logs/wasm-source-image-api-red.log`를 해당 API 계약의 수정 전 실패 근거로 남긴다.
기준의 모든 그림 출력 결함을 이 한 검사로 입증했다는 뜻은 아니다.

workspace 빌드는4분25초에 exit0이다. 현재 전체 target clippy와 이후 전체
Rust 실행은 `logs/post-rebase-fe572-remaining-results.json`에서 최종 종료코드로 확인한다.

전체 target clippy는4분18초에 exit0이다. 마지막 테스트 편집 뒤 생성 harness의
배정이 달라진 manifest 첫 검사는 실패했으며, 다시 prepare한 뒤 check는
1,508 sources /6,554 static test attrs /48 integration targets 모두 통과했다.
이후 전체 Rust 실행에는 새 생성물을 사용한다. 파생 manifest·harness를 커밋하지 않는다.

용지 폭·높이를 정수 PDF pt로 맞추는 별도 Canvas 진단도 실행했다. 실제 PNG가
바뀌는 두 번째 실행에서130쪽은93.35865%였지만,224/226쪽은98.57665/98.55720%로
하락했다.375쪽도99.41633%로 하락하므로 이 일괄 변환은 기각하고 제품 코드를
바꾸지 않았다. 첫 진단은 scale=1일 때 hook이 실행되지 않아 결과가 같았던 것으로,
가설 판정 근거에서 제외했다. `paper-point-diagnostic-decision.json`에 구분했다.
