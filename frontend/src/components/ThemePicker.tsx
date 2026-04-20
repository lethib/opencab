import { t } from "i18next";
import { Check, Moon, Sun, SunMoon } from "lucide-react";
import { useTheme } from "next-themes";
import type { ReactNode } from "react";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "./ui/dropdown-menu";
import { SidebarMenuButton } from "./ui/sidebar";

type AppTheme = "system" | "light" | "dark";

const THEMES_ICONS_AND_TEXTS: Record<
  AppTheme,
  { icon: ReactNode; text: string }
> = {
  system: {
    icon: <SunMoon />,
    text: t("components.themePicker.system"),
  },
  light: {
    icon: <Sun />,
    text: t("components.themePicker.light"),
  },
  dark: {
    icon: <Moon />,
    text: t("components.themePicker.dark"),
  },
};

export const ThemePicker = () => {
  const { theme: selectedTheme, setTheme } = useTheme();
  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <SidebarMenuButton className="w-9 shrink-0 [&>svg]:size-5">
          {THEMES_ICONS_AND_TEXTS[((selectedTheme ?? "system") as AppTheme)].icon}
        </SidebarMenuButton>
      </DropdownMenuTrigger>
      <DropdownMenuContent side="right" align="end" alignOffset={4}>
        {Object.entries(THEMES_ICONS_AND_TEXTS).map((theme) => (
          <DropdownMenuItem
            key={theme[0]}
            onClick={() => setTheme(theme[0])}
            className="flex items-center justify-between"
          >
            <div className="flex items-center gap-2">
              {theme[1].icon}
              {theme[1].text}
            </div>
            {selectedTheme === theme[0] && <Check className="text-primary" />}
          </DropdownMenuItem>
        ))}
      </DropdownMenuContent>
    </DropdownMenu>
  );
};
