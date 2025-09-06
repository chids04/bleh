import { SidebarProvider, SidebarTrigger } from "@/components/ui/sidebar"
import  CSidebar  from "@/components/CSidebar"
 
export default function Layout({ children, className }: { children: React.ReactNode, className?: string }) {
  
  return (
    <SidebarProvider className="border-green-600 border-2 h-full">
      <div className="flex border-3 border-amber-400 h-full w-full">
        <CSidebar/>
        <main className= "flex-1 overflow-auto p-5">
            {children}
        </main>
      </div>
    </SidebarProvider>
  )
}