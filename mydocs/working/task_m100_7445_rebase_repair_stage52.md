# Stage 52: 부분 셀의 문단 상대 overlay 앵커 복구

Issue: #7445. parent `f8f4cc63b6a5b91939d98b72c2aafee60f8fb96b`, base `2e087a5b4bc1515ad877e8020b30114639a59392`다. 현재 개인정보 문서의 기존 범위만 보정했다.

원본 제목에서 글 앞 표까지의 2997 HU 오프셋은 문단 앵커에 속한다. 부분 셀 경로가 글자 뒤 커서 16px를 쓰던 오류를 전체 셀의 공통 `nested_overlay_outer_frame` query와 실제 문단 시작 소유로 복구했다. 재귀 자식의 최종 원점도 같은 결과를 소비한다. P39의 셀 전체 프레임 조건은 전쪽 출력이 그대로여서 수용하지 않았다.

P40 Native 전 66쪽에서 59쪽이 46.20358→99.03871%로 개선됐다. 나머지 65 PNG는 byte 동일, 점수 하락 0쪽, 소유별 연결 문자열 1826개 변화 0이다. 59·60쪽 review/overlay를 직접 열어 앵커·본문·후속 쪽 보존과 선/glyph 잔여를 확인했다. 기존 앵커 관계의 별도 SDK 실행은 수정 전 FAIL → 수정 후 PASS다. 기존 집중 113건은 112 PASS / 1 FAIL, 테스트 20.762초이며 실패는 65·66쪽의 원본 설명 칸 소실이다.

전쪽 최저는 66쪽 60.81274%, 미달 1쪽이며 fixed base 대비 41쪽 하락도 남는다. fresh WASM·최신 전체 회귀·제출 게이트·actual36는 아직 필요하다. 검사·기대값·지침·원격은 바꾸지 않았고 새 문서를 추가하지 않았다.

[입력/producer·소비 경로·전쪽 TSV·전후 review/overlay](../pr/assets/issue7445/completed-20261009/stage52-pii-overlay-origin-20261022/verification.md). 실행 로그는 ignored output의 `repair-focused-v40/nextest.log`, `pii-overlay-native-v40.log`, `pii-overlay-anchor-cause-v40/anchor-relation-result.json`에 보존한다.
