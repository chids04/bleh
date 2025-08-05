import { invoke } from "@tauri-apps/api/core"
import { useEffect, useState } from "react"
import { ImageDisplay } from "@/components/ImageDisplay"

import type { Song, Image } from "@/types"


type CoversMap = Record<string, Image>;

export default function SongsDisplay() {
    const [songs, setSongs] = useState<Song[]>([])
    const [nowPlaying, setNowPlaying] = useState<Song>()
    const [covers, setCovers] = useState<CoversMap>({});

    useEffect(() => {
        const load_songs = async () => {
            console.log("getting songs")
            const songs = await invoke<Song[]>("get_songs");

            setSongs(songs)

            const covers = await invoke<CoversMap>("get_covers");
            setCovers(covers)
        }
        

        load_songs()

    }, [])

    const playSong = async (id: string) => {
        try {
            await invoke("play_song", { id })
            
        }
        catch (error) {
            console.log(String(error))
        }
    }
        
    return (
        <div className="flex flex-col w-full">
            <div className="flex flex-col gap-2">
                {songs.map((song) => (
                    <div onClick={() => playSong(song.id)} className="flex flex-row bg-zinc-900 rounded-lg gap-2">
                        {/* <div className="w-[100px] h-[100px] bg-zinc-700 flex items-center justify-center">
                            <span className="text-3xl font-bold text-zinc-400">
                                {song.title.charAt(0)}
                            </span>
                        </div> */}
                        <ImageDisplay
                        image={
                            covers[song.album[0]] ?? undefined
                        }
                        className="w-[100px] h-[100px]"
                        />
                        <div className="flex flex-col text-white">
                            <p>{song.title}</p>
                            <p>{song.artist[1]}</p>
                            <p>{song.album[1]}</p>
                        </div>
                    </div>
                ))}
            </div>
            
        </div>
    )
}