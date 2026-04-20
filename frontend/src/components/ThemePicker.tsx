import { Check, Moon, Sun, SunMoon } from "lucide-react";
import { useTheme } from "next-themes";
import type { ReactNode } from "react";
import { useTranslation } from "react-i18next";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "./ui/dropdown-menu";
import { SidebarMenuButton } from "./ui/sidebar";

type AppTheme = "system" | "light" | "dark";

const THEMES_ICONS: Record<AppTheme, ReactNode> = {
  system: <SunMoon />,
  light: <Sun />,
  dark: <Moon />,
};

export const ThemePicker = () => {
  const { theme: selectedTheme, setTheme } = useTheme();
  const { t } = useTranslation();
  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <SidebarMenuButton className="w-9 shrink-0 [&>svg]:size-5">
          {THEMES_ICONS[(selectedTheme ?? "system") as AppTheme]}
        </SidebarMenuButton>
      </DropdownMenuTrigger>
      <DropdownMenuContent side="right" align="end" alignOffset={4}>
        {Object.entries(THEMES_ICONS).map((theme) => (
          <DropdownMenuItem
            key={theme[0]}
            onClick={() => setTheme(theme[0])}
            className="flex items-center justify-between"
          >
            <div className="flex items-center gap-2">
              {theme[1]}
              {t(`components.themePicker.${theme[0]}`)}
            </div>
            {selectedTheme === theme[0] && <Check className="text-primary" />}
          </DropdownMenuItem>
        ))}
      </DropdownMenuContent>
    </DropdownMenu>
  );
};
