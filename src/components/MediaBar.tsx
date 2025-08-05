import { Slider } from "@/components/ui/slider"
import { useState, useEffect } from "react"
import { Pause, Play } from "lucide-react"
import { SkipForward } from "lucide-react";
import { SkipBack } from "lucide-react"
import type { Image } from "@/types";

import { listen } from '@tauri-apps/api/event';
import { invoke } from "@tauri-apps/api/core";

type PlayingSong = {
    title: String,
    artists: String,
    cover?: Image
}

export default function MediaBar() {
    const [sliderValue, setSliderValue] = useState([33]);
    const [play, setPlay] = useState(false)

    useEffect(() => {

        const unlisten = listen("playing-song", (e) => {
            setPlay(true)
        })

        return () => {
            unlisten.then(f => f());
        }
    }, [])

    const togglePlay = async() => {
        console.log(play)
        setPlay(!play)

        await invoke("toggle_play")
    }
    
    return (
        <div className="flex flex-row h-full w-full">
            <div className="w-1/4 border-pink-400 border-2">

            </div>

            <div className="w-2/4 h-full border-pink-400 border-2">
            
            <div className="flex justify-center items-center gap-4 my-2">
                <SkipBack
                    size={24}  
                    color="currentColor"
                    strokeWidth={2}
                    className="text-muted-foreground hover:text-muted" 
                />

                {
                    play ? 
                        <Pause 
                            size={24}  
                            color="currentColor"
                            strokeWidth={2}
                            className="text-muted-foreground hover:text-muted" 
                            onClick={togglePlay}
                        />
                        :
                        <Play
                            size={24}  
                            color="currentColor"
                            strokeWidth={2}
                            className="text-muted-foreground hover:text-muted" 
                            onClick={togglePlay}
                        />

                }

                {/* <Play 
                    size={24}  
                    color="currentColor"
                    strokeWidth={2}
                    className="text-muted-foreground hover:text-muted" 
                />
                <SkipForward
                    size={24}  
                    color="currentColor"
                    strokeWidth={2}
                    className="text-muted-foreground hover:text-muted" 
                /> */}
            </div>

            <div className="flex flex-row gap-3">
            <p className="text-muted-foreground">0;00</p>
            <Slider/>
            <p className="text-muted-foreground">0;00</p>
            </div>

            </div>
            
            <div className="w-1/4 border-pink-400 border-2">

            </div>

        </div>
    )
}