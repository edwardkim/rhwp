/**
 * 첫 실행에는 선택 창 없이 기본 클래식 스킨을 저장한다.
 *
 * 저장된 스킨이 있으면 그대로 유지한다. 이후 변경은 보기 > 테마에서 한다.
 * 기존 시작 경로와 호환되도록 함수명과 표시 여부 반환값은 유지한다.
 */
import { userSettings } from '../core/user-settings';

/** 현재 스킨을 확정하되 대화상자는 표시하지 않는다. */
export function maybeShowSkinOnboarding(): boolean {
  try {
    // 새 설정의 skin은 'default'(클래식)이며 기존 선택은 덮어쓰지 않는다.
    userSettings.markSkinChosen();
  } catch {
    // 저장소가 차단되거나 가득 차도 현재 스킨으로 앱을 계속 시작한다.
  }
  return false;
}
