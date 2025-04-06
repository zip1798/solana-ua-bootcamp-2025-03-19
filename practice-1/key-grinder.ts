import yargs from 'yargs';
import { hideBin } from 'yargs/helpers';
import logUpdate from 'log-update';
import { randomBytes } from 'crypto';
import base58 from "bs58";
import { Keypair } from "@solana/web3.js";

const argv = yargs(hideBin(process.argv)).argv;
if (!argv['starts'] && !argv['ends']) {
    throw new Error('Have to set --starts or --ends parameter');
}

const startsWith = argv['starts'] ? String(argv['starts']).toLowerCase().split(',') : [];
const endsWith = argv['ends'] ? String(argv['ends']).toLowerCase().split(',') : [];

const frames = ['-', '\\', '|', '/'];
const result: Keypair[] = [];
const startAt = Date.now();

let iterations = 0;

setInterval(() => {
    const now = Date.now();
    const elapsed = now - startAt;
    const seconds = Math.floor((elapsed / 1000) % 60).toString().padStart(2, '0');
    const minutes = Math.floor((elapsed / 1000 / 60) % 60).toString().padStart(2, '0');
    const hours = Math.floor(elapsed / 1000 / 3600).toString();
    const frame = frames[Math.floor(elapsed / 250) % frames.length]; // кожні 250мс

    for (let i = 0; i < 100; i++) {  // генеруємо 100 ключів за раз
        const keypair = generateKeyPair();
        iterations++;
        if (checkKeyPair(keypair)) {
            result.push(keypair);
        }
    }

    logUpdate(`${frame} ${hours}:${minutes}:${seconds} | Total Checked: ${iterations} | Found: ${result.length}\n${getResultString(result)}`);
}, 100);

function generateKeyPair(): Keypair {
    const seed = randomBytes(32);
    return Keypair.fromSeed(seed);
}

function checkKeyPair(keypair: Keypair): boolean {
    const publicKey = keypair.publicKey.toBase58().toLowerCase();
    const startMatch = startsWith.length === 0 || startsWith.some(prefix => publicKey.startsWith(prefix));
    const endMatch = endsWith.length === 0 || endsWith.some(suffix => publicKey.endsWith(suffix));
    return startMatch && endMatch;
}

function getResultString(result: Keypair[]): string {
    return result.map(kp => `[${kp.publicKey.toBase58()}, ${base58.encode(kp.secretKey)}]`).join('\n');
}
