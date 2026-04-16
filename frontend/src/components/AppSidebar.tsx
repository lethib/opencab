import { useLocation, useNavigate } from "@tanstack/react-router";
import { Building2, Info, LogOut, Users } from "lucide-react";
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

const navItems = [
  { label: "Mes patients", icon: Users, to: "/patients" },
  { label: "Mon profil", icon: Info, to: "/my_information" },
  { label: "Mes cabinets", icon: Building2, to: "/offices" },
];

export function AppSidebar() {
  const navigate = useNavigate();
  const { pathname } = useLocation();

  console.log(pathname.startsWith("/patients"));

  return (
    <Sidebar>
      <SidebarHeader className="mb-3 mt-1">
        <div className="flex items-center ml-2 gap-2">
          <img src="/favicon/favicon.svg" width={30} />
          <H2 className="text-primary">OpenCab</H2>
        </div>
      </SidebarHeader>

      <SidebarContent className="mx-2">
        <SidebarMenu>
          {navItems.map(({ label, icon: Icon, to }) => (
            <SidebarMenuItem key={to}>
              <SidebarMenuButton
                isActive={pathname.startsWith(to)}
                onClick={() => navigate({ to })}
              >
                <Icon />
                {label}
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
              Déconnexion
            </SidebarMenuButton>
          </SidebarMenuItem>
        </SidebarMenu>
      </SidebarFooter>
    </Sidebar>
  );
}
