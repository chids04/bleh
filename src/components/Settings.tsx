import { useEffect, useState } from "react"
import { Button } from "@/components/ui/button"
import { invoke } from '@tauri-apps/api/core';
import { open } from "@tauri-apps/plugin-dialog"
import { DirectoryTable } from "@/components/settings/DirectoryTable";
import { Separator } from "@/components/ui/separator";


export function SettingsView() {
    const [error, setError] = useState("")
    

    const handleClick = async () => {
        const selected = await open({
            directory: true,
            multiple: false, 
        })

        if(selected) {
            console.log(selected)
            try {
                await invoke("read_directory", {path: selected})
            }
            catch(error) {
                setError(String(error))
            }
        }
        
    }

    return (
        <div>
            <Button 
            className="border-muted-foreground border-2 hover:bg-muted-foregrounds mb-3" 
            onClick={handleClick}>scan directory</Button>
            
            {error && (
                <div
                    className="mt-4 p-3 rounded-md text-sm bg-red-900/30 text-red-400">
                    {error}
                </div>
            )}

            <Separator className="mb-3"/>

            <DirectoryTable />
        </div>


    )
}
