import { createRootRoute, Outlet, useLocation } from "@tanstack/react-router";
import { AppSidebar } from "@/components/AppSidebar";
import { SidebarProvider } from "@/components/ui/sidebar";

const NO_SIDEBAR_ROUTES = ["/login", "/reset_password"];

export const Route = createRootRoute({
  component: RootLayout,
});

function RootLayout() {
  const { pathname } = useLocation();
  const showSidebar = !NO_SIDEBAR_ROUTES.some((r) => pathname.startsWith(r));

  if (!showSidebar) {
    return <Outlet />;
  }

  return (
    <SidebarProvider>
      <AppSidebar />
      <div className="flex flex-col flex-1 min-h-screen">
        <main className="flex-1 py-8 px-6">
          <Outlet />
        </main>
      </div>
    </SidebarProvider>
  );
}
