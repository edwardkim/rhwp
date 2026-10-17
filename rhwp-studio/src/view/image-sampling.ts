/** Interpolate controls enlarged source samples, not minification coverage. */
export function magnificationFiltering(
  declared: 'nearest' | 'linear' | undefined,
  sourceWidth: number,
  sourceHeight: number,
  deviceWidth: number,
  deviceHeight: number,
): 'nearest' | 'linear' | undefined {
  if (declared !== 'nearest' && declared !== 'linear') return undefined;
  if (![sourceWidth, sourceHeight, deviceWidth, deviceHeight]
    .every(value => Number.isFinite(value) && value > 0)
    || deviceWidth < sourceWidth || deviceHeight < sourceHeight) return undefined;
  return declared;
}
