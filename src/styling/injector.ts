const STYLE_ID = "mantle-css";

/**
 * Insert or replace the managed `<style id="mantle-css">` element.
 *
 * Safe to call repeatedly — subsequent calls replace the content in place
 * rather than appending a new element.
 */
export function injectCss(css: string): void {
  let el = document.getElementById(STYLE_ID) as HTMLStyleElement | null;
  if (!el) {
    el = document.createElement("style");
    el.id = STYLE_ID;
    document.head.appendChild(el);
  }
  el.textContent = css;
}

/** Remove the managed style element. Useful for teardown and testing. */
export function removeManagedStyle(): void {
  document.getElementById(STYLE_ID)?.remove();
}
