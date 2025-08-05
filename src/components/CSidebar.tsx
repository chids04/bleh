import { 
  ListMusic, 
  Disc, 
  Mic2, 
  Music, 
  ListOrdered, 
  Search, 
  Settings 
} from "lucide-react"

import {
  Sidebar,
  SidebarContent,
  SidebarGroup,
  SidebarGroupContent,
  SidebarGroupLabel,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
} from "@/components/ui/sidebar"

// Updated items with music-related categories
const items = [
  {
    title: "playlists",
    url: "#",
    icon: ListMusic,
  },
  {
    title: "albums",
    url: "albums",
    icon: Disc,
  },
  {
    title: "artists",
    url: "#",
    icon: Mic2,
  },
  {
    title: "songs",
    url: "songs",
    icon: Music,
  },
  {
    title: "queue",
    url: "#",
    icon: ListOrdered,
  },
  {
    title: "search",
    url: "#",
    icon: Search,
  },
  {
    title: "settings",
    url: "settings",
    icon: Settings,
  },
]

export default function CSidebar() {
  return (
    // Add "bg-black text-white" classes for dark mode with white text
    <Sidebar collapsible="none" className="bg-zinc-800">
      <SidebarContent>
        <SidebarGroup>
          <SidebarGroupContent>
            <SidebarMenu>
              {items.map((item) => (
                <SidebarMenuItem key={item.title} className="mb-3">
                  {/* Add stroke-white to make icons white */}
                  <SidebarMenuButton asChild className="bg-zinc-700 hover:bg-zinc-600">
                    <a href={item.url}>
                      <item.icon className="stroke-white" />
                      <span className="text-white">{item.title}</span>
                    </a>
                  </SidebarMenuButton>
                </SidebarMenuItem>
              ))}
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>
      </SidebarContent>
    </Sidebar>
  )
}