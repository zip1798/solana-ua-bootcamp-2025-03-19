export function sleep(ms: number) {
    return new Promise(resolve => setTimeout(resolve, ms));
}

export async function sleepAsync(ms: number) {
    return new Promise(resolve => setTimeout(resolve, ms));
}