# 본문 호스트에 속한 저장 표의 첫 물리 프레임

Issue: #7445, 관련 기존 검사 #6860. 기준 `fe5725dc384689b45d60ef8ea4c83138bc2701d8`,
부모 `02a716c85da647ee6c8f3800d3668cfb2634a4e7`와 emitter 보정으로 검증했다.

## 원인과 소비 경로

원본 `samples/issue6860/3067979-road-lighting-photometric-hancom2020.hwpx`
(SHA-256 `028c3d318e121120ed4b8a71ba11bbb343d724da3563d4133bd354ca4fb61b2a`)의
독립 한컴 PDF `pdf/3067979-road-lighting-photometric-hancom2020.pdf`
(`d2ef6b035034036f769a94a550efbfaf6a542d14846e16ad88ad3b53c1e59182`)는 2쪽이다.
원본 표 높이 62656HU는 첫 물리 프레임을 선언한다. 호스트 문단에 본문이 있다는 이유로
그 프레임의 빈 밴드를 버려 논리 높이가 835.413333CSSpx에서 821.08CSSpx로 줄었다.

`saved_multirow_opening_frame_height`의 저장 프레임·컷 인증 → fragment emitter의
`first_fragment_blank_band` → `end_row_height_override`/`partial_height` → 흐름의 점유 예약과
`layout_table_partial`의 실제 셀 배치로 같은 높이를 전달한다. 호스트의 가시 텍스트는 앵커를
정하지만 표의 저장 프레임을 무효화하지 않는다. 따라서 object-only 호스트를 추가 요구하던
가정만 제거했다. 저장 출처·컷·재조판 여부·가용 예산 검사는 유지한다.

기존 정식 2개 검사 안에서 첫 물리 프레임, 마지막 셀의 CENTER 정렬, 두 쪽의 원문 순서·
누락·중복을 함께 확인한다. 프레임은 원본 HU 선언과 논리 셀 끝점의 관계다. 표 노드 bbox는
괘선 반폭을 포함하므로 초기 진단의 bbox 직접 비교는 잘못됐으며 논리 셀 끝점으로 정정했다.
정정한 동일 검사에서 수정 전 1FAIL/1PASS(exit100), 수정 후 2PASS(exit0)다.
1쪽이 98% 미만이므로 화면 절대 좌표를 고정하는 새 px golden은 추가하지 않았다.

## 실제 검증

96DPI·같은 원본/독립 PDF·canonical raw 실루엣·2px 관용으로 비교했다.

| 출력 | 1쪽 수정 전 | 1쪽 수정 후 | 2쪽 수정 후 |
| --- | ---: | ---: | ---: |
| Native | 93.88067% | 96.77948% | 99.97851% |
| fresh WASM | 93.91628% | 96.83194% | 99.97851% |

전후 review/overlay에서 마지막 라벨과 표 하단, 이어받기 내용을 직접 판독했다. 제목과 글자
윤곽의 작은 차이는 남아 있으며 100% 일치로 주장하지 않는다.

- Native producer `9af46f338545152d72ef358f751c9a1c5c750d1dab49370c3ea3eea2e0dfab04`.
- fresh WASM JS `9ec13c6960f52856ca2c17686c3b47f814887cece0e2185072a2400dc7dfc7a4`,
  WASM `c6f6b77dae4e4eb1e79c99b9ac4a094229626a5f6b19bb23cd7a346d734d2ed4`.
- Mac locked wrapper의 `--target web --out-dir pkg --no-opt` 로컬 대체 빌드다.
  Docker 최적화 검증으로 기록하지 않는다.
- Native/fresh WASM 각 48조건·1745쪽 전쪽 산출 완료. 45조건은 전쪽 90% 이상이다.
  sample16 HWP/동일 별칭/HWPX의 18·35쪽은 여전히 미달이다.
  전체 최저는 Native 76.25384%, WASM 70.65059%이며 PR 준비 완료가 아니다.
- Native 전쪽 C85 대비 0.01%p 이상 변동은 이 조명 1쪽 개선뿐이다.
  방향 대조군 Native의 나머지 22 PNG는 동일하다. WASM 로드맵 295쪽은 30픽셀 차이와
  99.13488→99.13485%의 미세 변동이 있어 byte-identical로 기록하지 않는다.
- `cargo nextest run --locked --cargo-profile release-test --target-dir
  /Users/tsjang/rhwp/target/pr-review --tests --test-threads 8 --no-fail-fast`:
  exit0, 10514PASS/0FAIL/45skip, 실행 451.727초(컴파일 포함 797.771초).
- 실제 Studio 파일 열기에서 screen/print × Canvas2D/CanvasKit × 2쪽, 8캡처 완료(exit0).
  초기 JS/WASM 응답 hash를 확인했다. CanvasKit 4캡처 모두 runtime readiness/completed,
  unsupported ops 없음, 숨은 Canvas2D overlay 없음. 마지막 셀·첫 줄·제목을 직접 판독했다.
  편집기 커서·모서리 표시는 숨기지 않았다. 별도 실제 소비 경로 36건 완료를 뜻하지 않는다.
- 추가 HWP5 저장 왕복에서 base PASS/0px 대비 후보 OVER/121.053333px를 발견했다.
  전체 nextest는 정상 종료했지만 후속 gate driver는 이 실제 저장 출처 결함 수정을 위해
  중단했다. 최종 lint/Skia/build·전쪽/실제 소비 검증은 후속 수정본에서 갱신한다.

## 최종 이미지 사용 계획

후속 PR의 조명 표 원인 설명과 검토 댓글에 다음 전후 패널을 사용한다.

- Native: [수정 전 review](before-native-review.png), [overlay](before-native-overlay.png),
  [수정 후 review](after-native-review.png), [overlay](after-native-overlay.png).
- fresh WASM: [수정 전 review](before-wasm-review.png), [overlay](before-wasm-overlay.png),
  [수정 후 review](after-wasm-review.png), [overlay](after-wasm-overlay.png).

TSV·실행 로그·생산자/진단 JSON은 커밋하지 않는다. 로컬 정본은 ignored
`output/pr-review/7445-completed-20261009/regression-policy-audit-20261015/rebase-fe5725dc-20261017/lighting-frame-c87/`다.
