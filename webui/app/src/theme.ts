export type ThemePreference = "automatic" | "light" | "dark";
export type EffectiveTheme = Exclude<ThemePreference, "automatic">;

export const themeStorageKey = "theme-preference";

function parseThemePreference(value: string | null): ThemePreference {
  return value === "light" || value === "dark" || value === "automatic"
    ? value
    : "automatic";
}

function loadThemePreference(): ThemePreference {
  try {
    return parseThemePreference(localStorage.getItem(themeStorageKey));
  } catch {
    return "automatic";
  }
}

function saveThemePreference(preference: ThemePreference) {
  try {
    localStorage.setItem(themeStorageKey, preference);
  } catch {
    // The preference still applies for this page when storage is unavailable.
  }
}

function resolveTheme(preference: ThemePreference, systemPrefersDark: boolean): EffectiveTheme {
  if (preference === "automatic") {
    return systemPrefersDark ? "dark" : "light";
  }
  return preference;
}

export function initializeTheme(onEffectiveThemeChange: (theme: EffectiveTheme) => void): EffectiveTheme {
  const systemTheme = window.matchMedia("(prefers-color-scheme: dark)");
  const controls = Array.from(
    document.querySelectorAll<HTMLInputElement>('input[name="theme-preference"]'),
  );
  let preference = loadThemePreference();

  const syncControls = () => {
    for (const control of controls) {
      control.checked = control.value === preference;
    }
  };

  const applyTheme = () => {
    const effectiveTheme = resolveTheme(preference, systemTheme.matches);
    document.documentElement.classList.toggle("dark", effectiveTheme === "dark");
    onEffectiveThemeChange(effectiveTheme);
    return effectiveTheme;
  };

  for (const control of controls) {
    control.addEventListener("change", () => {
      if (!control.checked) {
        return;
      }
      preference = parseThemePreference(control.value);
      saveThemePreference(preference);
      syncControls();
      applyTheme();
    });
  }

  systemTheme.addEventListener("change", () => {
    if (preference === "automatic") {
      applyTheme();
    }
  });

  window.addEventListener("storage", (event) => {
    if (event.key !== themeStorageKey) {
      return;
    }
    preference = parseThemePreference(event.newValue);
    syncControls();
    applyTheme();
  });

  syncControls();
  return applyTheme();
}
