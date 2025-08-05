import { useEffect, useState } from "react"
import type { Image } from "@/types"

interface ImageDisplayProps {
    image?: Image;
    altText?: string;
    className?: string;
}

export function ImageDisplay({ image, altText = "Image", className = "" }: ImageDisplayProps) {
    const [imageUrl, setImageUrl] = useState<string | null>(null);

    useEffect(() => {
        if (!image || image.data.length === 0) {
            return
        }

        console.log(image)

        const uint8Data = new Uint8Array(image.data);

        const blob = new Blob([uint8Data], { type: image.extension });


        const url = URL.createObjectURL(blob);
        setImageUrl(url);

        console.log(url)

        return () => {
            URL.revokeObjectURL(url);
        };

    }, [image]); 



    return (
        <img
            src={imageUrl ? imageUrl : "null"}
            alt={altText}
            className={className}
        />
    );
}
