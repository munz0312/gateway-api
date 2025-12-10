import { clsx, type ClassValue } from "clsx"
import { twMerge } from "tailwind-merge"
import { Duration } from "./types"

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

export function parseDuration(duration: Duration): number {
  if (typeof duration === "string") {
    // Fallback if received as string (e.g. ISO 8601)
    // For now return 0 to avoid crash
    return 0
  }
  return duration.secs * 1000 + duration.nanos / 1_000_000
}
