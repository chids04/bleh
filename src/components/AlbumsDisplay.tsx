import { useState, useEffect } from "react";
import type { Album } from "@/types"
import { invoke } from "@tauri-apps/api/core"

export default function AlbumsDisplay() {
  const [albums, setAlbums] = useState<Album[]>([])

  useEffect(() => {
    const loadAlbums = async () => {
      const albums = await invoke<Album[]>("get_albums");
      console.log(albums)
      setAlbums(albums)
    }

    loadAlbums()

  }, [])

  return (
    <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-5">
      {albums.map((album) => (
        <div
          key={album.id}
          className="flex flex-col bg-gray-900 rounded-lg shadow-lg overflow-hidden w-full"
        >
          <div className="aspect-square w-full bg-zinc-700 flex items-center justify-center rounded-t-lg">
            <span className="text-3xl font-bold text-zinc-400">
              {album.title.charAt(0)}
            </span>
          </div>
          <div className="p-3 text-white">
            <h3 className="text-sm font-semibold mb-1 h-[2.8em] overflow-hidden leading-tight line-clamp-2">
              {album.title}
            </h3>
            <p className="text-xs text-gray-400 whitespace-nowrap overflow-hidden text-ellipsis">
              {album.artists.map(artist => artist[1]).join(', ')}
            </p>
          </div>
        </div>
      ))}
    </div>
  );
}