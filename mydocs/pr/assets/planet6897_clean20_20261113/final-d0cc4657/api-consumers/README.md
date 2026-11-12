# 새 보정 코드의 실제 CLI 소비 검증

`qualification.json`은 source code `d0cc46573c`의 frozen Native 실행 파일을 사용한 실제 결과다.

- #8959: 표 편집 → HWP 저장 → 재저장. 세 출력이 앞서 독립 CFB 감사한 파일과 바이트가 같아
  본문 구역과 `/RhwpLineSegTags` 표지의 보존을 확인했다.
- #9003: 실제 HWPX 저장·재열기·재저장에서157쪽을 보존했고, 기존 커밋 입력과 SHA가 같다.
- #9006/#9023: 영수증의 두 차례 저장 결과가 기존 커밋 입력과 같은 SHA다.
- #8991: 정식 HML fixture에서 중첩 `cellPath`와 네 값 추출을 확인했다. 기존 devel JSON과
  후보 JSON을 함께 보존한다. source 경로 외의 결과는 앞 검증과 같다.
- #9013: Mac APFS에서 긴 이름 입력의 SVG·text 각4쪽 출력 이름이255바이트 이내이며
  UTF-8 경계와 내용이 보존된다. Linux의 `ENAMETOOLONG`을 Mac에서 재현했다고 쓰지 않는다.

비교 기준·프로듀서·입력/결과 해시는 각 JSON에 연결한다. API에 관계없는 문서 이미지를
증거로 붙이지 않고, 새로운 렌더링 점수를 이 API 검사의 값으로 기재하지 않는다.
