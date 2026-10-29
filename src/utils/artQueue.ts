import { ART_MAX_CONCURRENT } from "../config/app";

/** Limit concurrent VLC/ffmpeg thumbnail jobs so opening a list doesn't spawn dozens. */
const artQueue: Array<() => void> = [];
let artRunning = 0;

export function enqueueArt<T>(job: () => Promise<T>): Promise<T> {
  return new Promise((resolve, reject) => {
    const run = () => {
      artRunning++;
      job()
        .then(resolve, reject)
        .finally(() => {
          artRunning--;
          const next = artQueue.shift();
          if (next) next();
        });
    };
    if (artRunning < ART_MAX_CONCURRENT) run();
    else artQueue.push(run);
  });
}
