import {
  Table,
  TableBody,
  TableCaption,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"

import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuTrigger,
} from "@/components/ui/context-menu"


import { useState, useEffect } from "react"
import { invoke } from "@tauri-apps/api/core"

export function DirectoryTable() {
    const [dirs, setDirs] = useState<{ id: number; path: string }[]>([])

    const handleDelete = async (id: number) => {
        await invoke("delete_directory", { id }) 
        loadDirs();
    }

    const loadDirs = async () => {
        const folders = await invoke<Record<number, string>>("get_directories")

        const sortedDirs = Object.entries(folders)
            .map(([id, path]) => ({ id: parseInt(id), path }))
            .sort((a, b) => a.id - b.id)

        setDirs(sortedDirs)

    }

    useEffect(() => {
        loadDirs()
    }, [])

    return (
        <>
        <ContextMenu>
            <Table>
                <TableHeader>
                    <TableRow>
                        <TableHead>path</TableHead>
                    </TableRow>
                </TableHeader>
                <TableBody>
                {dirs.map((dir) => (
                    <ContextMenu key={dir.id}>
                        <ContextMenuTrigger asChild>
                            <TableRow>
                                <TableCell className="font-medium">{dir.path}</TableCell>
                            </TableRow>
                        </ContextMenuTrigger>

                        <ContextMenuContent className="w-52">
                            <ContextMenuItem className="text-red-500" onSelect={(e) => handleDelete(dir.id)}>
                                delete
                            </ContextMenuItem>
                        </ContextMenuContent>
                    </ContextMenu>
                ))}
                </TableBody>
            </Table>
        </ContextMenu>
        
        </>
    )
}