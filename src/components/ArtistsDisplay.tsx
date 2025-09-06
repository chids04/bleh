import { useState, useEffect } from "react"
import { invoke } from "@tauri-apps/api/core"

interface DisplayArtist {
    uuid: string
    name: string
}

export function ArtistsDisplay() {
    const [artists, setArtists] = useState<DisplayArtist[]>([])


    useEffect(() => {
        const loadArtists = async () => {
            const artists = await invoke<DisplayArtist[]>("get_artists")
            setArtists(artists)
        }

        loadArtists()
    }, [])
    return (
        <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-5">
            {artists.map((artist) => (
                <div
                    key={artist.uuid}
                    className="flex flex-col bg-gray-900 rounded-lg shadow-lg overflow-hidden w-full"
                >
                    <div className="aspect-square w-full bg-zinc-700 flex items-center justify-center rounded-t-lg">
                        <span className="text-3xl font-bold text-zinc-400">
                            {artist.name.charAt(0)}
                        </span>
                    </div>
                    <div className="p-3 text-white">
                        <h3 className="text-sm font-semibold mb-1 h-[2.8em] overflow-hidden leading-tight line-clamp-2">
                            {artist.name}
                        </h3>
                    </div>
                </div>
            ))}
        </div>
    );
}