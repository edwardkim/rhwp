/**
 * `@rhwp/hwpctrl/studio-plugin` 의 타입 표면.
 *
 * 패키지는 순수 ESM(.mjs)이라 타입 선언이 없다. studio 는 이 플러그인을 **동적 import 로만**
 * 가져오고 `StudioPlugin` 모양으로만 다루므로, 여기서는 그 계약만 선언한다.
 */
import type { StudioPlugin } from '@/plugin/types';

export declare const hwpctrlStudioPlugin: StudioPlugin;
export default hwpctrlStudioPlugin;
