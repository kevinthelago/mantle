import { useCallback } from "react";
import { SearchInput } from "./components/SearchInput";
import { ResultsList } from "./components/ResultsList";
import { useKeyboardNav } from "./hooks/useKeyboardNav";
import { useLauncher } from "./hooks/useLauncher";
import styles from "./LauncherApp.module.css";

/**
 * Root component of the Mantle app launcher.
 *
 * Keyboard contract:
 *  ↑ / ↓     — navigate results
 *  Enter      — launch selected app or run command
 *  Escape     — close launcher
 *  >          — switch to run mode (arbitrary shell command)
 */
export function LauncherApp() {
  const launcher = useLauncher();

  useKeyboardNav({
    onUp: launcher.selectPrev,
    onDown: launcher.selectNext,
    onConfirm: launcher.confirm,
    onDismiss: launcher.dismiss,
  });

  const handleSelect = useCallback(
    (index: number) => {
      // Single click selects; double click / Enter launches.
      const current = launcher.selectedIndex;
      if (index === current) {
        void launcher.confirm();
      } else {
        // Update selection to the clicked row. Since useLauncher exposes
        // only selectNext/Prev, we invoke them the required number of times.
        // For a richer UX this would be a direct set — acceptable here.
        const delta = index - current;
        for (let i = 0; i < Math.abs(delta); i++) {
          delta > 0 ? launcher.selectNext() : launcher.selectPrev();
        }
      }
    },
    [launcher]
  );

  return (
    <div className={styles.launcher} role="dialog" aria-label="App launcher">
      <SearchInput
        value={launcher.query}
        mode={launcher.mode}
        onChange={launcher.setQuery}
      />
      {launcher.error && (
        <p className={styles.error} role="alert">
          {launcher.error}
        </p>
      )}
      <ResultsList
        results={launcher.results}
        selectedIndex={launcher.selectedIndex}
        onSelect={handleSelect}
      />
    </div>
  );
}
