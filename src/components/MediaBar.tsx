import { Slider } from "@/components/ui/slider";
import { useState, useEffect, useRef } from "react";
import { Pause, Play, SkipBack, SkipForward } from "lucide-react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";

type PlayingSong = {
    title: string;
    artist: [string, string];
    features?: [string | null, string][];
    album: [string, string];
    duration: number;
};

export default function MediaBar() {
    const [sliderValue, setSliderValue] = useState<number>(0);
    const [isPlaying, setIsPlaying] = useState<boolean>(false);
    const [currentSong, setCurrentSong] = useState<PlayingSong | null>(null);
    const [isDragging, setIsDragging] = useState<boolean>(false);
    const animationFrameRef = useRef<number | null>(null);
    const startTimeRef = useRef<number | null>(null);

    const formatTime = (seconds: number | undefined): string => {
        if (seconds === undefined || isNaN(seconds)) return "0:00";
        const minutes = Math.floor(seconds / 60);
        const remainingSeconds = Math.floor(seconds % 60);
        return `${minutes}:${remainingSeconds.toString().padStart(2, "0")}`;
    };

    useEffect(() => {
        if (isPlaying && currentSong?.duration && !isDragging) {
            startTimeRef.current = performance.now() / 1000 - sliderValue;

            const updateSlider = () => {
                if (!isPlaying || !currentSong?.duration || isDragging) return;

                const currentTime = performance.now() / 1000;
                const elapsedTime = currentTime - (startTimeRef.current || 0);
                const newValue = Math.min(elapsedTime, currentSong.duration);

                setSliderValue(newValue);

                if (newValue >= currentSong.duration) {
                    setIsPlaying(false);
                    cancelAnimationFrame(animationFrameRef.current!);
                    return;
                }

                animationFrameRef.current = requestAnimationFrame(updateSlider);
            };

            animationFrameRef.current = requestAnimationFrame(updateSlider);
        }

        return () => {
            if (animationFrameRef.current) {
                cancelAnimationFrame(animationFrameRef.current);
            }
        };
    }, [isPlaying, currentSong, isDragging]);

    useEffect(() => {
        const unlisten = listen<PlayingSong>("playing-song", (e) => {
            setCurrentSong(e.payload);
            setIsPlaying(true);
            setSliderValue(0);
            startTimeRef.current = performance.now() / 1000;
        });

        return () => {
            unlisten.then((f) => f());
        };
    }, []);

    const togglePlay = async () => {
        try {
            await invoke("toggle_play");
            setIsPlaying(!isPlaying);
            if (!isPlaying) {
                startTimeRef.current = performance.now() / 1000 - sliderValue;
            }
        } catch (error) {
            console.error("failed to toggle play:", error);
        }
    };

    const handleSliderChange = (value: number[]) => {
        const newValue = value[0];
        setSliderValue(newValue);
    };

    //stops the slider from moving whilst slider is held, better ux
    const handleSliderDragStart = () => {
        setIsDragging(true);
    };

    //this actually seeks the song
    //instead of seeking song at each new position, instead seek song on when mouse released from slider
    //greatly reduces number of messages sent to audio thread
    const handleSliderCommit = async (value: number[]) => {
        setIsDragging(false);
        const newValue = value[0];
        startTimeRef.current = performance.now() / 1000 - newValue;
        try {
            await invoke("seek_to", { pos: newValue });
        } catch (error) {
            console.error("failed to seek:", error);
        }
    };

    return (
        <div className="flex flex-row h-full w-full select-none">
            <div className="w-1/4 border-pink-400 border-2">
            </div>

            <div className="w-2/4 h-full border-pink-400 border-2">
                <div className="flex justify-center items-center gap-4 my-2">
                    <SkipBack
                        size={24}
                        color="currentColor"
                        strokeWidth={2}
                        className="text-muted-foreground hover:text-muted cursor-pointer"
                        onClick={() => invoke("previous_song")}
                    />
                    {isPlaying ? (
                        <Pause
                            size={24}
                            color="currentColor"
                            strokeWidth={2}
                            className="text-muted-foreground hover:text-muted cursor-pointer"
                            onClick={togglePlay}
                        />
                    ) : (
                        <Play
                            size={24}
                            color="currentColor"
                            strokeWidth={2}
                            className="text-muted-foreground hover:text-muted cursor-pointer"
                            onClick={togglePlay}
                        />
                    )}
                    <SkipForward
                        size={24}
                        color="currentColor"
                        strokeWidth={2}
                        className="text-muted-foreground hover:text-muted cursor-pointer"
                        onClick={() => invoke("next_song")}
                    />
                </div>
                <div className="flex flex-row items-center gap-3 px-2">
                    <div className="text-muted-foreground text-sm w-12 text-right font-mono tabular-nums">
                        {formatTime(sliderValue)}
                    </div>
                    <div className="flex-1">
                        <Slider
                            value={[sliderValue]}
                            max={currentSong?.duration ? Math.floor(currentSong.duration) : 100}
                            step={0.1}
                            onValueChange={handleSliderChange}
                            onValueCommit={handleSliderCommit}
                            onPointerDown={handleSliderDragStart}
                            className="w-full"
                        />
                    </div>
                    <div className="text-muted-foreground text-sm w-12 font-mono tabular-nums">
                        {formatTime(currentSong?.duration ? Math.floor(currentSong.duration) : 0)}
                    </div>
                </div>
            </div>

            <div className="w-1/4 border-pink-400 border-2"></div>
        </div>
    );
}