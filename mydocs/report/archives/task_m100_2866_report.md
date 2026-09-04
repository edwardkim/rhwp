# Task #2866 처리 결과 — 스타일 이름/영문 이름 입력 길이 미검증 (#2851/#2862 재발)

## 문제

`rhwp-studio/src/ui/style-edit-dialog.ts`의 `nameInput`(스타일 이름)과 `enNameInput`(영문 이름)에
길이 제한이 없어, `wasm.createStyle()` / `wasm.updateStyle()`을 거쳐 `serialize_style()`
(`src/serializer/doc_info.rs:665-670`) → `write_hwp_string()`
(`src/serializer/byte_writer.rs:70-77`)에서 `utf16.len() as u16` 캐스팅이 랩어라운드되어
STYLE 레코드가 손상될 수 있었다. #2851(필드 이름, PR #2854)과 #2862(책갈피 이름, PR #2865)와
동일한 원인의 세 번째 재발 경로.

## 재현 (수정 전)

1. 스타일 추가/편집 대화상자를 연다.
2. 스타일 이름 또는 영문 이름 입력란에 UTF-16 코드 유닛 65536개 이상인 문자열을 넣는다
   (예: 자동화 스크립트나 클립보드 붙여넣기).
3. 저장하면 `write_hwp_string`의 `utf16.len() as u16`이 0으로 랩어라운드되어 길이 프리픽스가
   깨지고, 실제 문자 데이터는 그대로 남아 STYLE 레코드 이후 DocInfo 파싱이 오프셋 밀림으로
   손상된다.

## 수정 (red → green)

- `rhwp-studio/src/ui/style-edit-dialog.ts`
  - `MAX_STYLE_NAME_LEN = 250` 상수 추가(#2865의 `MAX_BOOKMARK_NAME_LEN` 패턴 준용).
  - `nameInput`/`enNameInput`에 `maxLength = MAX_STYLE_NAME_LEN` 설정.
  - `onConfirm()`에 길이 초과 시 알림 후 반환하는 방어 검증 추가(붙여넣기 등으로 `maxLength`를
    우회한 값이 들어와도 저장 직전에 차단).
- `rhwp-studio/tests/style-name-length-guard.test.ts` (신규)
  - `MAX_STYLE_NAME_LEN`이 u16 랩어라운드 지점(65536)보다 작은지 검증하는 소스 가드 테스트
    (수정 전에는 상수 자체가 없으므로 red, 수정 후 green).

`.rs` 파일은 수정하지 않았다(범위: TypeScript 프런트엔드 가드만).

## 검증

- `npm test` (`rhwp-studio`): 500 테스트 중 499 통과, 실패 1건은 기존에 알려진
  `tests/cell-flow-boundary.test.ts` 뿐(무관한 사전 실패).
- `npx tsc --noEmit` (`rhwp-studio`): 사전 존재하는 TS2307 오류 2건(`@wasm/rhwp.js` 모듈 누락)만
  남고 신규 오류 없음.

## 관련

- 이슈 #2866
- 선행 사례: #2851 → PR #2854, #2862 → PR #2865
