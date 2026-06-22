import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { useCallback, useEffect, useState } from 'react';

export type SniStatus = 'passive' | 'active' | 'needsAttention';

export type MenuItemType = 'standard' | 'separator';
export type MenuToggleType = 'none' | 'checkmark' | 'radio';
export type ToggleState = 'off' | 'on' | 'indeterminate';

export interface MenuItem {
  id: number;
  label: string;
  enabled: boolean;
  visible: boolean;
  iconName: string | null;
  itemType: MenuItemType;
  toggleType: MenuToggleType;
  toggleState: ToggleState;
  children: MenuItem[];
}

export interface TrayMenu {
  revision: number;
  root: MenuItem;
}

export interface TrayItem {
  key: string;
  id: string;
  title: string;
  status: SniStatus;
  category: string;
  icon: string | null;
  tooltip: string | null;
  menuPath: string | null;
  menu: TrayMenu | null;
}

export interface TraySnapshot {
  items: TrayItem[];
}

export function useTray(): {
  items: TrayItem[];
  activate: (key: string, x: number, y: number) => Promise<void>;
  sendMenuEvent: (key: string, itemId: number, event: string) => Promise<void>;
  refreshMenu: (key: string) => Promise<TrayMenu | null>;
} {
  const [items, setItems] = useState<TrayItem[]>([]);

  useEffect(() => {
    let unlistenUpdate: UnlistenFn | undefined;
    let unlistenRemoved: UnlistenFn | undefined;

    invoke<TraySnapshot>('get_tray_items')
      .then((snap) => setItems(snap.items))
      .catch(() => setItems([]));

    listen<TraySnapshot>('tray_update', (e) => {
      setItems(e.payload.items);
    }).then((fn) => { unlistenUpdate = fn; });

    listen<string>('tray_item_removed', (e) => {
      setItems((prev) => prev.filter((i) => i.key !== e.payload));
    }).then((fn) => { unlistenRemoved = fn; });

    return () => {
      unlistenUpdate?.();
      unlistenRemoved?.();
    };
  }, []);

  const activate = useCallback(async (key: string, x: number, y: number) => {
    await invoke('tray_item_activate', { key, x, y });
  }, []);

  const sendMenuEvent = useCallback(async (key: string, itemId: number, event: string) => {
    await invoke('tray_menu_event', { key, itemId, event });
  }, []);

  const refreshMenu = useCallback(async (key: string): Promise<TrayMenu | null> => {
    return invoke<TrayMenu | null>('tray_refresh_menu', { key });
  }, []);

  return { items, activate, sendMenuEvent, refreshMenu };
}
