const SPLIT_ITEM_SELECTOR = '.tb-split-item:not([disabled])';

export function toolbarSplitItems(split: Element): HTMLButtonElement[] {
  return Array.from(split.querySelectorAll<HTMLButtonElement>(SPLIT_ITEM_SELECTOR));
}

export function setToolbarSplitOpen(
  split: Element,
  open: boolean,
  options: { focus?: 'first' | 'last'; returnFocus?: boolean } = {},
): void {
  const arrow = split.querySelector<HTMLButtonElement>('.tb-split-arrow');
  const menu = split.querySelector<HTMLElement>('.tb-split-menu');
  split.classList.toggle('open', open);
  arrow?.setAttribute('aria-expanded', String(open));

  if (!open) {
    if (options.returnFocus) arrow?.focus({ preventScroll: true });
    return;
  }
  if (!menu) return;

  const anchorRect = split.getBoundingClientRect();
  const menuRect = menu.getBoundingClientRect();
  const gutter = 4;
  const left = Math.min(
    Math.max(gutter, anchorRect.left),
    Math.max(gutter, window.innerWidth - menuRect.width - gutter),
  );
  const below = anchorRect.bottom + 2;
  const top = below + menuRect.height <= window.innerHeight - gutter
    ? below
    : Math.max(gutter, anchorRect.top - menuRect.height - 2);
  menu.style.left = `${left}px`;
  menu.style.top = `${top}px`;

  if (options.focus) {
    const items = toolbarSplitItems(split);
    const target = options.focus === 'first' ? items[0] : items.at(-1);
    requestAnimationFrame(() => target?.focus({ preventScroll: true }));
  }
}

export function closeToolbarSplitMenus(root: ParentNode, except?: Element): void {
  root.querySelectorAll('.tb-split.open').forEach((split) => {
    if (split !== except) setToolbarSplitOpen(split, false);
  });
}

export function moveToolbarSplitFocus(
  split: Element,
  current: Element,
  direction: -1 | 1,
): void {
  const items = toolbarSplitItems(split);
  const index = items.indexOf(current as HTMLButtonElement);
  if (index < 0 || items.length === 0) return;
  items[(index + direction + items.length) % items.length]?.focus({ preventScroll: true });
}
