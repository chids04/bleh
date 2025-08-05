const albums = [
  {
    id: 1,
    title: 'good kid, m.A.A.d city (Deluxe Edition)',
    artist: 'Kendrick Lamar',
    image: '/images/good_kid_maad_city.jpg', // Adjust path for Astro static assets
  },
  {
    id: 2,
    title: 'Section.80',
    artist: 'Kendrick Lamar',
    image: '/images/section_80.jpg', // Adjust path for Astro static assets
  },
  {
    id: 3,
    title: 'To Pimp a Butterfly',
    artist: 'Kendrick Lamar',
    image: '/images/to_pimp_a_butterfly.jpg', // Adjust path for Astro static assets
  },
  {
    id: 4,
    title: 'untitled unmastered.',
    artist: 'Kendrick Lamar',
    image: '/images/untitled_unmastered.jpg', // Adjust path for Astro static assets
  },
  {
    id: 5,
    title: 'DAMN.',
    artist: 'Kendrick Lamar',
    image: '/images/damn.jpg', // Adjust path for Astro static assets
  },
  {
    id: 6,
    title: 'Mr. Morale & The Big Steppers',
    artist: 'Kendrick Lamar',
    image: '/images/mr_morale.jpg', // Adjust path for Astro static assets
  },
];

export default function AlbumsDisplay() {
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
              {album.artist}
            </p>
          </div>
        </div>
      ))}
    </div>
  );
}