import React, {
  useCallback,
  useEffect,
  useRef,
  useState,
} from 'react';

import type { MenuItem, TrayItem } from './useTray';
import styles from './Tray.module.css';

interface TrayItemProps {
  item: TrayItem;
  onActivate: (key: string, x: number, y: number) => Promise<void>;
  onMenuEvent: (key: string, itemId: number, event: string) => Promise<void>;
  onRefreshMenu: (key: string) => Promise<import('./useTray').TrayMenu | null>;
}

interface MenuState {
  x: number;
  y: number;
  items: MenuItem[];
}

export function TrayItemView({
  item,
  onActivate,
  onMenuEvent,
  onRefreshMenu,
}: TrayItemProps): React.ReactElement {
  const [menu, setMenu] = useState<MenuState | null>(null);
  const ref = useRef<HTMLDivElement>(null);

  // Close on outside click
  useEffect(() => {
    if (!menu) return;
    const handler = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) {
        setMenu(null);
      }
    };
    document.addEventListener('mousedown', handler);
    return () => document.removeEventListener('mousedown', handler);
  }, [menu]);

  const handleLeftClick = useCallback(
    async (e: React.MouseEvent) => {
      e.preventDefault();
      setMenu(null);
      await onActivate(item.key, e.clientX, e.clientY);
    },
    [item.key, onActivate],
  );

  const handleRightClick = useCallback(
    async (e: React.MouseEvent) => {
      e.preventDefault();
      const fresh = await onRefreshMenu(item.key);
      const menuItems = (fresh ?? item.menu)?.root?.children ?? [];
      setMenu({ x: e.clientX, y: e.clientY, items: menuItems });
    },
    [item, onRefreshMenu],
  );

  return (
    <div ref={ref} className={styles.item} data-status={item.status}>
      <button
        type="button"
        style={{ all: 'unset', display: 'contents' }}
        title={item.tooltip ?? item.title}
        onClick={handleLeftClick}
        onContextMenu={handleRightClick}
        aria-label={item.title || item.id}
      >
        <ItemIcon item={item} />
      </button>

      {menu && (
        <MenuPopover
          menu={menu}
          itemKey={item.key}
          onEvent={onMenuEvent}
          onClose={() => setMenu(null)}
        />
      )}
    </div>
  );
}

function ItemIcon({ item }: { item: TrayItem }): React.ReactElement {
  if (item.icon?.startsWith('data:') || item.icon?.startsWith('file://')) {
    return (
      <img
        className={styles.icon}
        src={item.icon}
        alt={item.title}
        draggable={false}
      />
    );
  }
  // Fallback: icon name as text (design-system Icon component takes over at merge)
  return (
    <span className={styles.icon} aria-hidden>
      {item.icon ?? item.id.slice(0, 1).toUpperCase()}
    </span>
  );
}

function MenuPopover({
  menu,
  itemKey,
  onEvent,
  onClose,
}: {
  menu: MenuState;
  itemKey: string;
  onEvent: (key: string, itemId: number, event: string) => Promise<void>;
  onClose: () => void;
}): React.ReactElement {
  // Clamp to viewport
  const style: React.CSSProperties = {
    top: Math.min(menu.y, window.innerHeight - 200),
    left: Math.min(menu.x, window.innerWidth - 200),
  };

  return (
    <div className={styles.menu} style={style} role="menu">
      <MenuItemList
        items={menu.items}
        itemKey={itemKey}
        onEvent={onEvent}
        onClose={onClose}
      />
    </div>
  );
}

function MenuItemList({
  items,
  itemKey,
  onEvent,
  onClose,
}: {
  items: MenuItem[];
  itemKey: string;
  onEvent: (key: string, itemId: number, event: string) => Promise<void>;
  onClose: () => void;
}): React.ReactElement {
  return (
    <>
      {items
        .filter((i) => i.visible)
        .map((i) => (
          <MenuItemView
            key={i.id}
            item={i}
            itemKey={itemKey}
            onEvent={onEvent}
            onClose={onClose}
          />
        ))}
    </>
  );
}

function MenuItemView({
  item,
  itemKey,
  onEvent,
  onClose,
}: {
  item: MenuItem;
  itemKey: string;
  onEvent: (key: string, itemId: number, event: string) => Promise<void>;
  onClose: () => void;
}): React.ReactElement {
  if (item.itemType === 'separator') {
    return <div className={styles.menuSeparator} role="separator" />;
  }

  const handleClick = async () => {
    if (!item.enabled) return;
    await onEvent(itemKey, item.id, 'clicked');
    onClose();
  };

  const checkMark =
    item.toggleType === 'checkmark'
      ? item.toggleState === 'on'
        ? '✓'
        : ' '
      : item.toggleType === 'radio'
      ? item.toggleState === 'on'
        ? '●'
        : ' '
      : null;

  return (
    <div
      className={styles.menuItem}
      role="menuitem"
      aria-disabled={String(!item.enabled)}
      onClick={handleClick}
      tabIndex={item.enabled ? 0 : -1}
      onKeyDown={(e) => {
        if (e.key === 'Enter' || e.key === ' ') handleClick();
      }}
    >
      {checkMark !== null && (
        <span className={styles.menuCheck}>{checkMark}</span>
      )}
      <span>{item.label}</span>
      {item.children.length > 0 && <span style={{ marginLeft: 'auto' }}>▶</span>}
    </div>
  );
}
