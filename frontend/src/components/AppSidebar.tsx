import { Link, useLocation, useNavigate } from "@tanstack/react-router";
import { Building2, Info, LogOut, Users } from "lucide-react";
import { useTranslation } from "react-i18next";
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
} from "@/components/ui/sidebar";
import { logout } from "@/lib/authUtils";
import { H2 } from "./ui/typography/h2";

export function AppSidebar() {
  const navigate = useNavigate();
  const { pathname } = useLocation();
  const { t } = useTranslation();

  const navItems = [
    { label: t("navigation.patients"), icon: Users, to: "/patients" },
    { label: t("navigation.myInformation"), icon: Info, to: "/my_information" },
    { label: t("navigation.myOffices"), icon: Building2, to: "/offices" },
  ];

  return (
    <Sidebar>
      <SidebarHeader className="mb-3 mt-1">
        <div
          className="flex items-center ml-2 gap-2 hover:cursor-pointer"
          onClick={() => navigate({ to: "/" })}
        >
          <img src="/favicon/favicon.svg" width={30} />
          <H2 className="text-primary">OpenCab</H2>
        </div>
      </SidebarHeader>

      <SidebarContent className="mx-2">
        <SidebarMenu>
          {navItems.map(({ label, icon: Icon, to }) => (
            <SidebarMenuItem key={to}>
              <SidebarMenuButton asChild isActive={pathname.startsWith(to)}>
                <Link to={to}>
                  <Icon />
                  {label}
                </Link>
              </SidebarMenuButton>
            </SidebarMenuItem>
          ))}
        </SidebarMenu>
      </SidebarContent>

      <SidebarFooter className="ml-2">
        <SidebarMenu>
          <SidebarMenuItem>
            <SidebarMenuButton onClick={logout} className="text-destructive">
              <LogOut />
              {t("auth.logout")}
            </SidebarMenuButton>
          </SidebarMenuItem>
        </SidebarMenu>
      </SidebarFooter>
    </Sidebar>
  );
}
