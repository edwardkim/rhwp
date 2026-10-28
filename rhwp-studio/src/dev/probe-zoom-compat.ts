import type { ZoomSessionState } from './zoom-session-observation';

/** 구 기준선의 없는 API는 관찰 불가로 남긴다. 제품에 메서드를 추가하지 않는다. */
export interface ProbeZoomCapabilities {
  getZoomInputState?: () => Partial<ZoomSessionState>;
  isZoomRasterPending?: () => boolean;
}

export function probeZoomInputState(vm: object): Partial<ZoomSessionState> {
  return (vm as ProbeZoomCapabilities).getZoomInputState?.() ?? {};
}

export function probeZoomRasterPending(vm: object): boolean {
  // 구 기준선은 별도 pending 단계 없이 animation 종료 경로에서 동기 raster한다.
  return (vm as ProbeZoomCapabilities).isZoomRasterPending?.() ?? false;
}
