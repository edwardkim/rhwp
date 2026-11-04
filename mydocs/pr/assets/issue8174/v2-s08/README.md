# #8174 T01·T02 실행 증거

제품 source는 `9cabbff39d`이며 T01의 Native 단독 제품 source는 `fd4386a5cc`다.
상세 계약과 실행은 [연산자 버퍼 보고](../../../../working/task_m100_8174_t01_operator_budget.md)와
[S08 보고](../../../../working/task_m100_8174_v2_stage8.md)를 따른다. 전체 PR 완료 판정은 아니다.

- [Native producer](t02-native-producer.json), [최적화 WASM producer](t02-wasm-producer.json).
- [공개 입력·해시](public-inputs.json), [독립 생성기](probes/prepare_clip_controls.py),
  [텍스트 생성기](probes/prepare_text_clip_control.py).
- [전쪽 시각 요약](visual-summary.json), [동일 실제 글꼴 환경](dejavu-family-environment.json).
- [WASM 실제 가져오기](t02-browser-clip.json), [두 저장 형식 재열기](t02-browser-text-batch.json),
  [Worker 거절 뒤 복구](t02-worker-clip.json), [Studio 상태 보존·복구](studio-summary.json).
- [연산자 보정 전](t02-operators-before.json), [800만·+1·밀집 입력](t02-operators-candidate.json).
  이 단회 시간·linear memory capacity는 계약 관찰이며 통제한 성능 비교나 전체 RSS가 아니다.

공개 이미지 5쪽·batch 2쪽은 Native/fresh WASM 모두 100%다. 텍스트 3쪽은 같은 실제 글꼴과
환경에서 Native 최저 99.88074%, WASM 100%다. `tsv/<runtime>/<case>/silhouette.tsv`는
실행 원문을 바이트 그대로 보존하며 `images/`는 대표 review·standalone overlay다.
독립 PDF와 직접 비교해 사각 crop·삼각형의 빈 alpha 영역·Form 회전·텍스트 순서와 간격을 확인했다.
렌더러 출력을 정답 PDF로 재생성하거나 점수 기준을 낮추지 않았다.

사용자 15쪽 원본과 식별 가능한 PDF 내용·경로·PNG·진단 전체는 ignored output에만 보존한다.
공개 자료에는 페이지 수·저장 바이트 수·성공/거절과 상태 보존만 요약했다. Native 전쪽 최저는
99.13508%, 저장 재열기를 포함한 WASM 전쪽 최저는 99.11167%다. 기존 글꼴 outline·굵기 차이를
원본과 동일하다고 확대 해석하지 않는다. 이번 clip 수정이 그 차이를 새로 만들었는지도 별도로 대조한다.

정식 회귀·추가 직접 판독 및 이후 source 변경의 검증 상태는 S08 보고를 따른다.
`manifest.json`은 이 디렉터리의 제출 자료 hash를 고정한다. 실행 로그·중간 tree·제품 바이너리는
커밋하지 않는다. 대표 이미지의 PR 본문 URL은 최종 제품 source 검증 뒤 정확한 PR head에 고정한다.
