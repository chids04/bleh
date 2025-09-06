export interface Song {
    id: string,
    title: string,
    artist: [string, string],
    album: [string, string],
    features?: [string | null, string][]
    track_num: number
    disc_num: number
    cover?: Image
    path: string
    duration: number
}

export interface Album {
    id: string
    title: string
    artists: [string | null, string][],
    cover?: Image
    songs: string[]
}

export interface Image {
    data: number[]
    extension: string
}
